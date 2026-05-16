# 插件动态加载规划

**工作项：** 3.4  
**优先级：** P2  
**工作量：** 5-7 天  
**依赖：** 无

---

## 一、目标

支持运行时加载工具适配器插件，无需重新编译：
- 动态加载 `.dylib` / `.so` / `.dll`
- C ABI 兼容，跨语言支持
- 插件签名验证（可选）
- 插件热更新

---

## 二、当前状态

### 2.1 现有插件系统

```rust
// aidaguard-plugins/src/adapters/*.rs

// 硬编码的适配器
pub struct Cursor;
impl ToolAdapter for Cursor { ... }

pub struct ClaudeCode;
impl ToolAdapter for ClaudeCode { ... }

// 每个适配器都编译在 binary 中
```

### 2.2 问题

1. **新增适配器需要重新编译**
2. **无法第三方扩展**
3. **不支持热更新**

---

## 三、架构设计

### 3.1 插件生命周期

```
┌─────────────────────────────────────────────────────────────────┐
│                      插件生命周期                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  扫描目录 → 加载库 → 验证签名 → 获取元数据 → 注册到 Registry    │
│     │                                                   │      │
│     ▼                                                   ▼      │
│  ~/.aidaguard/plugins/                              可用状态   │
│  ├── cursor.json                                              │
│  ├── cursor.dylib                                             │
│  └── ...                                                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 目录结构

```
~/.aidaguard/plugins/
├── cursor/
│   ├── manifest.json      # 元数据
│   ├── cursor.dylib       # macOS
│   ├── cursor.so          # Linux
│   └── cursor.dll         # Windows
├── claude_code/
│   ├── manifest.json
│   └── ...
└── custom/
    └── my_adapter.dylib
```

### 3.3 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                    PluginRegistry                                │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 插件管理                                                 │   │
│  │ - load_from_dir() 扫描加载                              │   │
│  │ - get(id) 获取插件                                      │   │
│  │ - list() 列出所有                                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           │                                     │
│                           ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ PluginLoader                                             │   │
│  │ - 扫描目录                                               │   │
│  │ - 加载动态库                                             │   │
│  │ - 解析符号表                                             │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           │                                     │
│                           ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ DynamicPlugin                                            │   │
│  │ - library: Library                                       │   │
│  │ - vtable: PluginVTable                                   │   │
│  │ - manifest: PluginManifest                               │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 四、实现方案

### 4.1 插件 ABI 定义

```rust
// aidaguard-plugins/src/abi.rs

use std::ffi::c_int;

/// 插件 ABI 版本
pub const ABI_VERSION: u32 = 1;

/// 插件元数据 (C ABI)
#[repr(C)]
pub struct PluginMeta {
    /// 插件 ID
    pub id: *const i8,
    /// 插件名称
    pub name: *const i8,
    /// 版本
    pub version: *const i8,
    /// 描述
    pub description: *const i8,
    /// 作者
    pub author: *const i8,
}

/// 插件虚函数表 (C ABI)
#[repr(C)]
pub struct PluginVTable {
    // ── 元信息 ──
    
    /// 获取插件 ID
    pub id: unsafe extern "C" fn() -> *const i8,
    
    /// 获取插件名称
    pub name: unsafe extern "C" fn() -> *const i8,
    
    /// 获取 ABI 版本
    pub abi_version: unsafe extern "C" fn() -> u32,
    
    // ── 生命周期 ──
    
    /// 初始化插件
    /// 返回 0 表示成功
    pub init: unsafe extern "C" fn() -> c_int,
    
    /// 清理插件
    pub cleanup: unsafe extern "C" fn(),
    
    // ── 检测 ──
    
    /// 检测工具是否安装
    pub detect: unsafe extern "C" fn() -> bool,
    
    // ── 配置 ──
    
    /// 获取配置文件路径
    pub config_path: unsafe extern "C" fn() -> *const i8,
    
    /// 获取当前端点
    pub current_endpoint: unsafe extern "C" fn() -> *const i8,
    
    /// 配置代理
    /// 返回 0 表示成功
    pub configure: unsafe extern "C" fn(proxy_url: *const i8) -> c_int,
    
    /// 恢复配置
    /// 返回 0 表示成功
    pub restore: unsafe extern "C" fn() -> c_int,
    
    // ── 状态 ──
    
    /// 检查是否已配置代理
    pub is_configured: unsafe extern "C" fn() -> bool,
    
    /// 获取当前模型
    pub current_model: unsafe extern "C" fn() -> *const i8,
}
```

### 4.2 插件加载器

```rust
// aidaguard-plugins/src/loader.rs

use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// 动态加载的插件
pub struct DynamicPlugin {
    /// 动态库句柄
    library: Library,
    /// 虚函数表
    vtable: PluginVTable,
    /// 元数据（已解析）
    manifest: PluginManifest,
}

impl DynamicPlugin {
    /// 获取插件 ID
    pub fn id(&self) -> &str {
        &self.manifest.id
    }
    
    /// 获取插件名称
    pub fn name(&self) -> &str {
        &self.manifest.name
    }
    
    /// 检测工具是否安装
    pub fn detect(&self) -> bool {
        unsafe { (self.vtable.detect)() }
    }
    
    /// 配置代理
    pub fn configure(&self, proxy_url: &str) -> Result<(), PluginError> {
        let url = std::ffi::CString::new(proxy_url)?;
        let result = unsafe { (self.vtable.configure)(url.as_ptr()) };
        if result == 0 {
            Ok(())
        } else {
            Err(PluginError::ConfigureFailed(self.manifest.id.clone()))
        }
    }
    
    /// 恢复配置
    pub fn restore(&self) -> Result<(), PluginError> {
        let result = unsafe { (self.vtable.restore)() };
        if result == 0 {
            Ok(())
        } else {
            Err(PluginError::RestoreFailed(self.manifest.id.clone()))
        }
    }
}

/// 插件加载器
pub struct PluginLoader {
    /// 插件目录
    plugin_dir: PathBuf,
    /// 已加载的插件
    loaded: HashMap<String, DynamicPlugin>,
}

impl PluginLoader {
    /// 创建加载器
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugin_dir,
            loaded: HashMap::new(),
        }
    }
    
    /// 扫描并加载所有插件
    pub fn scan_and_load(&mut self) -> Result<Vec<String>, PluginError> {
        let mut loaded_ids = Vec::new();
        
        if !self.plugin_dir.exists() {
            return Ok(loaded_ids);
        }
        
        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let path = entry?.path();
            
            if path.is_dir() {
                // 尝试加载目录中的插件
                if let Ok(id) = self.load_from_dir(&path) {
                    loaded_ids.push(id);
                }
            } else if self.is_library(&path) {
                // 直接加载库文件
                if let Ok(id) = self.load(&path) {
                    loaded_ids.push(id);
                }
            }
        }
        
        Ok(loaded_ids)
    }
    
    /// 从目录加载插件
    fn load_from_dir(&mut self, dir: &Path) -> Result<String, PluginError> {
        // 查找库文件
        let lib_path = self.find_library(dir)?;
        
        // 加载 manifest
        let manifest_path = dir.join("manifest.json");
        let manifest = self.load_manifest(&manifest_path)?;
        
        // 加载库
        self.load_with_manifest(&lib_path, manifest)
    }
    
    /// 加载单个库文件
    fn load(&mut self, path: &Path) -> Result<String, PluginError> {
        // 加载库
        let library = unsafe { Library::new(path)? };
        
        // 获取 vtable
        let get_vtable: Symbol<unsafe extern "C" fn() -> PluginVTable> = 
            unsafe { library.get(b"plugin_vtable")? };
        
        let vtable = unsafe { get_vtable() };
        
        // 验证 ABI 版本
        let abi_version = unsafe { (vtable.abi_version)() };
        if abi_version != ABI_VERSION {
            return Err(PluginError::AbiMismatch {
                expected: ABI_VERSION,
                actual: abi_version,
            });
        }
        
        // 解析 ID
        let id = unsafe {
            let ptr = (vtable.id)();
            std::ffi::CStr::from_ptr(ptr)
                .to_string_lossy()
                .into_owned()
        };
        
        // 解析 manifest（从库元数据）
        let manifest = self.parse_manifest_from_vtable(&vtable)?;
        
        let plugin = DynamicPlugin {
            library,
            vtable,
            manifest,
        };
        
        self.loaded.insert(id.clone(), plugin);
        Ok(id)
    }
    
    /// 检查是否是库文件
    fn is_library(&self, path: &Path) -> bool {
        let ext = path.extension().and_then(|s| s.to_str());
        match ext {
            #[cfg(target_os = "macos")]
            Some("dylib") => true,
            #[cfg(target_os = "linux")]
            Some("so") => true,
            #[cfg(target_os = "windows")]
            Some("dll") => true,
            _ => false,
        }
    }
    
    /// 在目录中查找库文件
    fn find_library(&self, dir: &Path) -> Result<PathBuf, PluginError> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if self.is_library(&path) {
                return Ok(path);
            }
        }
        Err(PluginError::LibraryNotFound(dir.display().to_string()))
    }
    
    /// 获取已加载的插件
    pub fn get(&self, id: &str) -> Option<&DynamicPlugin> {
        self.loaded.get(id)
    }
    
    /// 列出所有已加载插件
    pub fn list(&self) -> Vec<&str> {
        self.loaded.keys().map(|s| s.as_str()).collect()
    }
}
```

### 4.3 插件注册表集成

```rust
// aidaguard-plugins/src/registry.rs

impl PluginRegistry {
    /// 从目录加载动态插件
    pub fn load_dynamic_plugins(&mut self, plugin_dir: &Path) -> Result<Vec<String>, PluginError> {
        let mut loader = PluginLoader::new(plugin_dir.to_path_buf());
        let loaded_ids = loader.scan_and_load()?;
        
        // 注册到 registry
        for id in &loaded_ids {
            if let Some(plugin) = loader.get(id) {
                self.register_dynamic_plugin(plugin);
            }
        }
        
        // 保存 loader
        self.loader = Some(loader);
        
        Ok(loaded_ids)
    }
    
    /// 注册动态插件
    fn register_dynamic_plugin(&mut self, plugin: &DynamicPlugin) {
        let wrapper = DynamicPluginAdapter(plugin);
        self.plugins.insert(plugin.id().to_string(), Box::new(wrapper));
    }
}

/// 动态插件适配器（实现 ToolAdapter trait）
struct DynamicPluginAdapter(&DynamicPlugin);

impl ToolAdapter for DynamicPluginAdapter {
    fn id(&self) -> &str { self.0.id() }
    fn name(&self) -> &str { self.0.name() }
    fn detect(&self) -> bool { self.0.detect() }
    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        self.0.configure(proxy_url).map_err(|e| e.to_string())
    }
    fn restore(&self) -> Result<(), String> {
        self.0.restore().map_err(|e| e.to_string())
    }
    // ... 其他方法
}
```

---

## 五、插件开发指南

### 5.1 创建插件（Rust）

```rust
// plugins/cursor/src/lib.rs

use aidaguard_plugins::abi::*;
use std::ffi::{CStr, CString};

static ID: &[u8] = b"cursor\0";
static NAME: &[u8] = b"Cursor\0";

#[no_mangle]
pub unsafe extern "C" fn plugin_vtable() -> PluginVTable {
    PluginVTable {
        id: || ID.as_ptr() as *const i8,
        name: || NAME.as_ptr() as *const i8,
        abi_version: || ABI_VERSION,
        init: || 0,
        cleanup: || {},
        detect: || {
            // 检测 Cursor 是否安装
            std::path::Path::new(
                "~/Library/Application Support/Cursor"
            ).exists()
        },
        config_path: || {
            // 返回配置文件路径
            static PATH: &[u8] = b"~/Library/Application Support/Cursor/User/settings.json\0";
            PATH.as_ptr() as *const i8
        },
        current_endpoint: || std::ptr::null(),
        configure: |proxy_url| {
            // 配置代理逻辑
            let url = CStr::from_ptr(proxy_url);
            // ... 写入配置文件
            0
        },
        restore: || {
            // 恢复配置
            0
        },
        is_configured: || false,
        current_model: || std::ptr::null(),
    }
}
```

### 5.2 编译插件

```toml
# plugins/cursor/Cargo.toml

[package]
name = "aidaguard-plugin-cursor"
version = "0.1.0"

[lib]
crate-type = ["cdylib"]  # 编译为动态库

[dependencies]
aidaguard-plugins = { path = "../../crates/aidaguard-plugins" }
```

```bash
# 编译
cargo build --release

# 输出
# target/release/libaidaguard_plugin_cursor.dylib  (macOS)
# target/release/libaidaguard_plugin_cursor.so     (Linux)
# target/release/aidaguard_plugin_cursor.dll       (Windows)
```

### 5.3 创建 Manifest

```json
// plugins/cursor/manifest.json

{
  "id": "cursor",
  "name": "Cursor",
  "version": "0.1.0",
  "description": "AI-first code editor",
  "author": "Cursor",
  "min_aidaguard_version": "0.4.0",
  "categories": ["editor", "openai-compatible"]
}
```

---

## 六、文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-plugins/src/abi.rs` | 新增 | ABI 定义 |
| `aidaguard-plugins/src/loader.rs` | 新增 | 加载器实现 |
| `aidaguard-plugins/src/registry.rs` | 修改 | 集成动态加载 |
| `aidaguard-plugins/src/lib.rs` | 修改 | 导出新模块 |
| `aidaguard-plugins/Cargo.toml` | 修改 | 添加 libloading 依赖 |

---

## 七、验收标准

- [ ] ABI 定义完整
- [ ] PluginLoader 实现
- [ ] 集成到 PluginRegistry
- [ ] 示例插件可加载
- [ ] 插件签名验证（可选）
- [ ] 文档完整

---

## 八、风险与缓解

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| ABI 不兼容 | 高 | 版本检查 + 文档 |
| 插件安全 | 高 | 签名验证 + 沙箱 |
| 内存安全 | 中 | unsafe 审查 |
| 平台差异 | 中 | 条件编译 |
