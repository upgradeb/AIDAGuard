# AIDAGuard 检测规则系统优化计划

## Context

用户希望全面优化 AIDAGuard 的检测规则系统，涵盖五个方面：

1. **统一规则系统** - 将 YAML 规则和 PatternRecognizer 合并为单一架构
2. **精简规则文件** - 合并重复规则、移除低价值规则、优化目录结构
3. **优化规则管理** - 改进规则热加载、版本管理、规则测试体验
4. **增强规则功能** - YAML 规则支持校验器声明、改进正则精度、增强上下文评分
5. **区域规则配置** - 设置中选择国家/地区，只显示相关规则，保留自定义规则能力

当前存在两套独立的检测系统，增加了维护复杂性和功能冗余。

---

## Phase 1: 统一规则架构

### 1.1 设计目标

将 YAML 规则和 PatternRecognizer 统一为声明式规则系统：

- YAML 规则支持 `validator` 字段声明校验器
- YAML 规则支持 `context_words` 字段声明上下文词
- PatternRecognizer 从 YAML 规则动态生成
- 保留 `AnalyzerEngine` 作为统一检测管线

### 1.2 新规则格式

```yaml
rules:
  - id: credit_card
    name: Credit Card
    pattern: '(?-u:\b)(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13})(?-u:\b)'
    validator: luhn                      # 新增：校验器名称
    context_words: [credit, card, visa, mastercard]  # 新增：上下文词
    base_confidence: 0.4                 # 新增：基础置信度
    enabled: true
    strategy: placeholder
    mode: filter
    priority: 100
    compliance: [PCI_DSS]
```

### 1.3 关键修改

**文件: `crates/aidaguard-core/src/detector/mod.rs`**

```rust
// RuleDef 新增字段
pub struct RuleDef {
    // ... 现有字段 ...
    pub validator: Option<String>,        // 校验器名称
    pub context_words: Vec<String>,       // 上下文词列表
    pub base_confidence: Option<f64>,     // 基础置信度
}

// CompiledRule 新增字段
pub struct CompiledRule {
    pub def: RuleDef,
    pub regex: Regex,
    pub exclude_regex: Option<Regex>,
    pub validator_fn: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,  // 新增
}
```

**文件: `crates/aidaguard-detector/src/recognizers/pattern/yaml_recognizer.rs` (新建)**

```rust
/// 从 YAML 规则动态生成的识别器
pub struct YamlRecognizer {
    rule: CompiledRule,
    context_enhancer: LemmaContextAwareEnhancer,
}

impl Recognizer for YamlRecognizer {
    fn entity_type(&self) -> EntityType {
        EntityType::Custom(self.rule.def.id.clone())
    }

    fn analyze(&self, text: &str) -> Vec<RecognizerResult> {
        // 正则匹配 -> 校验器过滤 -> 上下文增强
    }
}
```

**文件: `crates/aidaguard-detector/src/validation/registry.rs` (新建)**

```rust
/// 校验器注册表
pub struct ValidatorRegistry {
    validators: HashMap<String, Arc<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl ValidatorRegistry {
    pub fn new() -> Self {
        let mut registry = Self { validators: HashMap::new() };
        registry.register("luhn", Arc::new(luhn::luhn_check));
        registry.register("id_card_cn", Arc::new(id_card_cn::validate_id_card_cn));
        registry.register("iban", Arc::new(iban::validate_iban));
        registry.register("us_ssn", Arc::new(us_ssn::validate_us_ssn));
        registry
    }
}
```

### 1.4 实现步骤

1. 扩展 `RuleDef` 和 `CompiledRule` 结构
2. 创建 `ValidatorRegistry` 校验器注册表
3. 创建 `YamlRecognizer` 从规则生成识别器
4. 修改 `RecognizerRegistry::load_from_rules()` 加载 YAML 规则
5. 移除硬编码的 PatternRecognizer（保留校验函数）

---

## Phase 2: 精简规则文件

### 2.1 当前问题

- 存在大量 `enabled: false` 的规则
- 部分规则重复定义（如 `credit_card` 在多处定义）
- 目录结构冗余（空文件如 `eu/general.yaml`）

### 2.2 优化方案

**合并后的目录结构：**

```
rules/
├── core.yaml           # 核心规则（始终加载）
│   ├── credentials     # API Key, AWS Key, GitHub Token, JWT, Private Key
│   ├── identifiers     # Email, IP, MAC, URL
│   └── financial       # Credit Card, IBAN, SWIFT, Crypto
│
├── cn.yaml             # 中国地区规则
│   ├── phone_cn, id_card_cn, passport_cn
│   ├── car_plate_cn, bank_card_cn
│   └── personal (姓名, 地址等)
│
├── us.yaml             # 美国地区规则
│   └── us_ssn, us_passport
│
├── eu.yaml             # 欧盟地区规则
│   └── eu_vat, gdpr_specific
│
└── gb.yaml             # 英国地区规则
    └── uk_nino, uk_passport
```

### 2.3 规则清理

**移除的规则：**
- `enabled: false` 且无实际使用价值的规则
- 重复定义的规则（保留精度最高的版本）

**保留但禁用的规则：**
- 有潜在价值但误报率高的规则（如 `amount_usd`）

### 2.4 实现步骤

1. 分析所有规则文件，识别重复和低价值规则
2. 设计新的目录结构
3. 合并规则文件
4. 更新 `Config::rule_presets()` 加载逻辑
5. 更新测试用例

---

## Phase 3: 优化规则管理

### 3.1 规则热加载改进

**文件: `crates/aidaguard-core/src/detector/mod.rs`**

当前问题：
- 热加载仅支持整个目录重新加载
- 无增量更新能力

改进方案：
```rust
/// 增量更新规则
pub fn update_rule(&mut self, id: &str, def: RuleDef) -> Result<()>;

/// 删除规则
pub fn remove_rule(&mut self, id: &str) -> Result<()>;
```

### 3.2 版本管理增强

**文件: `crates/aidaguard-core/src/detector/versioned.rs`**

新增功能：
```rust
impl VersionedDetector {
    /// 获取规则变更历史
    pub fn history(&self) -> &[Arc<RuleSnapshot>];

    /// 比较两个版本的差异
    pub fn diff(&self, v1: u64, v2: u64) -> RuleDiff;

    /// 导出当前规则快照
    pub fn export_snapshot(&self) -> Vec<u8>;

    /// 导入规则快照
    pub fn import_snapshot(&self, data: &[u8]) -> Result<()>;
}
```

### 3.3 规则测试改进

**API 增强: `crates/aidaguard-tauri/src-tauri/src/commands/rules.rs`**

```rust
/// 测试规则并返回详细结果
#[tauri::command]
pub async fn test_rule_detailed(
    pattern: String,
    text: String,
    validator: Option<String>,
) -> Result<TestResult, String> {
    // 返回: matches, confidence_scores, validation_results
}

pub struct TestResult {
    pub matches: Vec<MatchDetail>,
    pub execution_time_ms: u64,
    pub pattern_valid: bool,
    pub validator_passed: usize,
    pub validator_failed: usize,
}
```

**UI 改进: `crates/aidaguard-tauri/src/src/components/RuleTestPanel.tsx`**
- 显示每个匹配的置信度分数
- 显示校验器通过/失败状态
- 显示执行时间
- 支持多个测试用例

---

## Phase 4: 增强规则功能

### 4.1 校验器声明支持

**内置校验器列表：**

| 名称 | 函数 | 适用规则 |
|------|------|----------|
| `luhn` | Luhn mod-10 | 信用卡 |
| `id_card_cn` | GB 11643-1999 mod-11 | 中国身份证 |
| `iban` | ISO 13616 mod-97 | IBAN |
| `us_ssn` | SSN 规则校验 | 美国 SSN |
| `uk_nino` | NINO 格式校验 | 英国国民保险号 |

### 4.2 正则精度改进

**中国手机号：**
```yaml
# 当前
pattern: '1[3-9]\d{9}'

# 改进：添加边界检测
pattern: '(?-u:\b)1[3-9]\d{9}(?-u:\b)'
exclude: '1[3-9]\d{9}@'  # 排除邮箱中的数字
```

**中国身份证：**
```yaml
# 当前
pattern: '\d{17}[\dXx]'

# 改进：完整格式校验
pattern: '(?-u:\b)[1-9]\d{5}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dXx](?-u:\b)'
validator: id_card_cn
```

### 4.3 上下文评分增强

**文件: `crates/aidaguard-detector/src/validation/context.rs`**

改进方向：
- 支持加权上下文词（不同词不同权重）
- 支持负向上下文词（降低置信度）
- 支持多语言上下文词

```rust
pub struct ContextWord {
    pub word: String,
    pub weight: f64,      // 正数加分，负数减分
    pub languages: Vec<String>,  // 适用语言
}

impl LemmaContextAwareEnhancer {
    pub fn enhance_weighted(
        &self,
        result: &RecognizerResult,
        text: &str,
        context_words: &[ContextWord],
    ) -> f64;
}
```

---

## Phase 5: 区域规则配置

### 5.1 需求说明

用户需要在设置中选择国家/地区，系统只显示和加载该地区的相关检测规则：

- **预设区域规则**：根据选择的地区自动加载对应的检测规则
- **自定义规则**：用户可添加自己的规则，不受地区限制
- **规则分类显示**：UI 中区分"系统规则"和"自定义规则"

### 5.2 区域配置设计

**配置结构 (`config.rs`)：**

```rust
pub struct Config {
    // ... 现有字段 ...
    
    /// 检测区域设置
    pub detection_region: DetectionRegion,
    
    /// 自定义规则目录
    pub custom_rules_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRegion {
    /// 主要地区 (cn, us, eu, gb, sg, jp, kr, global)
    pub primary: String,
    
    /// 附加地区 (可多选)
    pub additional: Vec<String>,
    
    /// 是否加载全局规则
    pub include_global: bool,
}

impl Default for DetectionRegion {
    fn default() -> Self {
        Self {
            primary: "global".to_string(),
            additional: vec![],
            include_global: true,
        }
    }
}
```

### 5.3 规则文件组织

**新的规则目录结构：**

```
rules/
├── global/                    # 全局规则（始终加载）
│   ├── credentials.yaml       # API Key, Token, Private Key
│   ├── identifiers.yaml       # Email, IP, MAC, URL
│   └── financial.yaml         # Credit Card, IBAN, SWIFT
│
├── regions/                   # 按地区组织
│   ├── cn/                    # 中国
│   │   ├── id_cards.yaml      # 身份证、护照、军官证
│   │   ├── phone.yaml         # 手机号
│   │   ├── vehicle.yaml       # 车牌号
│   │   ├── financial.yaml     # 银行卡、统一信用代码
│   │   └── personal.yaml      # 姓名、地址（NLP）
│   │
│   ├── us/                    # 美国
│   │   ├── id_cards.yaml      # SSN, ITIN, 护照
│   │   ├── driver_license.yaml # 各州驾照
│   │   └── financial.yaml     # EIN, ABA路由号
│   │
│   ├── eu/                    # 欧盟
│   │   ├── vat.yaml           # 各国 VAT 号
│   │   ├── id_cards.yaml      # 各国身份证
│   │   └── gdpr.yaml          # GDPR 敏感数据
│   │
│   ├── sg/                    # 新加坡
│   │   ├── nric.yaml          # NRIC, FIN
│   │   └── uen.yaml           # 企业注册号
│   │
│   ├── jp/                    # 日本
│   │   ├── my_number.yaml     # My Number
│   │   └── id_cards.yaml      # 住基卡、健康保险证
│   │
│   ├── gb/                    # 英国
│   │   ├── nino.yaml          # NINO
│   │   ├── nhs.yaml           # NHS 号码
│   │   └── driver_license.yaml # 驾照
│   │
│   └── kr/                    # 韩国
│       └── rrn.yaml           # 居民登记号
│
└── custom/                    # 用户自定义规则
    └── *.yaml                 # 用户创建的规则文件
```

### 5.4 规则元数据

**YAML 规则新增 `region` 字段：**

```yaml
rules:
  - id: id_card_cn
    name: 中国身份证
    pattern: '[1-9]\d{5}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dXx]'
    validator: id_card_cn
    region: cn                    # 所属地区
    source: system                # system / custom
    enabled: true
    strategy: placeholder
    mode: filter
    priority: 100
    compliance: [PIPL]
```

### 5.5 规则加载逻辑

**文件: `aidaguard-core/src/config.rs`**

```rust
impl Config {
    /// 根据区域配置计算要加载的规则目录
    pub fn rule_presets(&self) -> Vec<PathBuf> {
        let mut presets = Vec::new();
        
        // 1. 全局规则（可选）
        if self.detection_region.include_global {
            presets.push(PathBuf::from("rules/global"));
        }
        
        // 2. 主要地区规则
        if self.detection_region.primary != "global" {
            presets.push(PathBuf::from(format!(
                "rules/regions/{}", 
                self.detection_region.primary
            )));
        }
        
        // 3. 附加地区规则
        for region in &self.detection_region.additional {
            presets.push(PathBuf::from(format!("rules/regions/{}", region)));
        }
        
        // 4. 自定义规则
        if let Some(ref custom_dir) = self.custom_rules_dir {
            presets.push(custom_dir.clone());
        }
        
        presets
    }
}
```

### 5.6 设置界面设计

**文件: `crates/aidaguard-tauri/src/src/pages/Settings.tsx`**

```
┌─────────────────────────────────────────────────────────────┐
│  检测规则设置                                                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  地区设置                                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 主要地区: [中国 ▼]                                    │   │
│  │                                                     │   │
│  │ 附加地区: (可多选)                                    │   │
│  │ ☐ 美国    ☐ 欧盟    ☐ 英国                          │   │
│  │ ☐ 新加坡  ☐ 日本    ☐ 韩国                          │   │
│  │                                                     │   │
│  │ ☑ 包含全局规则                                       │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  规则统计                                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 已加载规则: 45 条                                     │   │
│  │ ├─ 全局规则: 15 条                                   │   │
│  │ ├─ 中国规则: 25 条                                   │   │
│  │ └─ 自定义规则: 5 条                                  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  自定义规则目录                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 路径: [./custom_rules          ] [浏览...]           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 5.7 规则管理界面

**文件: `crates/aidaguard-tauri/src/src/pages/Rules.tsx`**

```
┌─────────────────────────────────────────────────────────────┐
│  规则管理                                                    │
├─────────────────────────────────────────────────────────────┤
│  筛选: [全部 ▼] [系统规则 ▼]                                │
│                                                             │
│  ┌─ 全局规则 (15) ─────────────────────────────────────┐   │
│  │ ☑ credit_card    信用卡        Luhn校验    启用      │   │
│  │ ☑ email          邮箱地址      -           启用      │   │
│  │ ...                                                 │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─ 中国规则 (25) ──────────────────────────────────────┐   │
│  │ ☑ id_card_cn     身份证        mod-11校验  启用      │   │
│  │ ☑ phone_cn       手机号        -           启用      │   │
│  │ ☐ passport_cn    护照          -           禁用      │   │
│  │ ...                                                 │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─ 自定义规则 (5) ─────────────────────────────────────┐   │
│  │ ☑ my_custom_1    自定义规则1   -           启用      │   │
│  │ ☑ my_custom_2    自定义规则2   -           启用      │   │
│  │                          [+ 添加自定义规则]           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 5.8 API 设计

**Tauri 命令 (`commands/rules.rs`)：**

```rust
/// 获取按区域分组的规则列表
#[tauri::command]
pub async fn get_rules_grouped(
    state: State<'_, AppState>,
) -> Result<RuleGroups, String> {
    // 返回: { global: [...], region: [...], custom: [...] }
}

/// 获取支持的地区列表
#[tauri::command]
pub async fn get_supported_regions() -> Vec<RegionInfo> {
    vec![
        RegionInfo { id: "cn", name: "中国", rule_count: 25 },
        RegionInfo { id: "us", name: "美国", rule_count: 15 },
        RegionInfo { id: "eu", name: "欧盟", rule_count: 20 },
        RegionInfo { id: "gb", name: "英国", rule_count: 10 },
        RegionInfo { id: "sg", name: "新加坡", rule_count: 8 },
        RegionInfo { id: "jp", name: "日本", rule_count: 12 },
        RegionInfo { id: "kr", name: "韩国", rule_count: 5 },
    ]
}

/// 更新检测区域设置
#[tauri::command]
pub async fn update_detection_region(
    region: DetectionRegion,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 1. 保存配置
    // 2. 重新加载规则
    // 3. 返回新的规则统计
}
```

### 5.9 实现步骤

| 步骤 | 任务 | 预估时间 |
|------|------|----------|
| 5.1 | 定义 `DetectionRegion` 配置结构 | 0.5 天 |
| 5.2 | 重构规则目录结构 | 0.5 天 |
| 5.3 | 更新规则加载逻辑 | 0.5 天 |
| 5.4 | 添加 `region` 和 `source` 元数据 | 0.5 天 |
| 5.5 | 实现设置界面地区选择 | 1 天 |
| 5.6 | 实现规则分组显示 | 1 天 |
| 5.7 | 实现自定义规则管理 | 1 天 |

**总计：约 5 天**

---

## 实现顺序

| 阶段 | 任务 | 优先级 | 预估时间 |
|------|------|--------|----------|
| 1.1 | 扩展 RuleDef 结构 | P0 | 1 天 |
| 1.2 | 创建 ValidatorRegistry | P0 | 1 天 |
| 1.3 | 创建 YamlRecognizer | P0 | 2 天 |
| 1.4 | 修改 RecognizerRegistry | P0 | 1 天 |
| 2.1 | 分析并精简规则文件 | P1 | 1 天 |
| 2.2 | 更新规则目录结构 | P1 | 0.5 天 |
| 3.1 | 规则热加载改进 | P2 | 1 天 |
| 3.2 | 版本管理增强 | P2 | 1 天 |
| 3.3 | 规则测试 UI 改进 | P2 | 2 天 |
| 4.1 | 内置校验器完善 | P1 | 1 天 |
| 4.2 | 正则精度改进 | P1 | 1 天 |
| 4.3 | 上下文评分增强 | P2 | 1 天 |
| 5.1 | 定义 DetectionRegion 配置结构 | P1 | 0.5 天 |
| 5.2 | 重构规则目录结构（按地区） | P1 | 0.5 天 |
| 5.3 | 更新规则加载逻辑 | P1 | 0.5 天 |
| 5.4 | 添加 region/source 元数据 | P1 | 0.5 天 |
| 5.5 | 实现设置界面地区选择 | P1 | 1 天 |
| 5.6 | 实现规则分组显示 | P1 | 1 天 |
| 5.7 | 实现自定义规则管理 | P2 | 1 天 |

**总计：约 18.5 天**

---

## 关键文件清单

### 需要修改的文件

| 文件 | 修改内容 |
|------|----------|
| `aidaguard-core/src/detector/mod.rs` | RuleDef 扩展、校验器集成、region/source 字段 |
| `aidaguard-core/src/detector/versioned.rs` | 版本管理增强 |
| `aidaguard-core/src/config.rs` | DetectionRegion 配置、规则加载逻辑 |
| `aidaguard-detector/src/pipeline.rs` | 规则加载逻辑 |
| `aidaguard-detector/src/core/recognizer_registry.rs` | 从 YAML 生成识别器 |
| `aidaguard-detector/src/validation/context.rs` | 加权上下文词 |
| `aidaguard-tauri/src-tauri/src/commands/rules.rs` | 测试 API、区域规则 API |
| `aidaguard-tauri/src/src/pages/Settings.tsx` | 地区设置界面 |
| `aidaguard-tauri/src/src/pages/Rules.tsx` | 规则分组显示 |
| `aidaguard-tauri/src/src/components/RuleTestPanel.tsx` | 测试 UI 改进 |

### 需要新建的文件

| 文件 | 内容 |
|------|------|
| `aidaguard-detector/src/validation/registry.rs` | 校验器注册表 |
| `aidaguard-detector/src/recognizers/pattern/yaml_recognizer.rs` | YAML 规则识别器 |
| `rules/regions/` 目录下各地区的 YAML 文件 | 按地区组织的检测规则 |

### 需要重构的文件

| 文件 | 内容 |
|------|------|
| `rules/` 目录下所有 YAML 文件 | 合并、精简、增强、添加元数据 |

---

## 验证方案

### 单元测试

```bash
# 测试校验器注册表
cargo test -p aidaguard-detector validator_registry

# 测试 YAML 规则识别器
cargo test -p aidaguard-detector yaml_recognizer

# 测试规则加载
cargo test -p aidaguard-core rule_loading

# 测试检测精度
cargo test -p aidaguard-detector detection_accuracy
```

### 集成测试

```bash
# 启动桌面应用
cd crates/aidaguard-tauri && cargo tauri dev

# 测试场景：
# 1. 规则热加载：修改 YAML 文件，验证自动重载
# 2. 规则测试：使用测试面板验证规则效果
# 3. 检测准确性：使用测试数据验证检测率
```

### 回归测试

确保现有功能不受影响：
- 所有现有 YAML 规则仍可加载
- 所有现有 Tauri 命令仍可执行
- 检测性能无明显下降

---

## 附录 A: 各国家/地区敏感数据类型法规参考

### A.1 中国 (PIPL - 个人信息保护法)

**法规依据：** 《个人信息保护法》第二十八条

**敏感个人信息类型：**

| 类别 | 具体数据类型 | 检测规则建议 |
|------|-------------|-------------|
| **生物识别** | 人脸、指纹、虹膜、声纹、基因 | 需 NLP/图像处理 |
| **宗教信仰** | 宗教团体成员、宗教活动记录 | NLP 上下文检测 |
| **特定身份** | 残疾证、军官证、党员身份 | 特定证件格式 |
| **医疗健康** | 病历、诊断、用药、基因检测、体检报告 | `medical_record_no`, ICD 诊断码 |
| **金融账户** | 银行账户、信用卡、贷款、征信 | `bank_card`, `credit_card`, `bank_account` |
| **行踪轨迹** | GPS 定位、出行记录、住宿记录 | GPS 坐标格式 |
| **不满十四周岁未成年人** | 儿童个人信息 | 需上下文判断 |

**中国特有标识符：**

| 数据类型 | 格式/示例 | 正则模式 |
|----------|----------|----------|
| 居民身份证 | 18位，含校验码 | `[1-9]\d{5}(?:19\|20)\d{2}(?:0[1-9]\|1[0-2])(?:0[1-9]\|[12]\d\|3[01])\d{3}[\dXx]` |
| 护照号码 | E/G + 8位数字 | `[EGeg]\d{8}` |
| 军官证 | 军字第XXXXXX号 | `军字第\d{6,8}号` |
| 港澳通行证 | C/H/M + 8位数字 | `[CHMchm]\d{8}` |
| 台湾通行证 | 8位数字或字母数字组合 | `[A-Z]\d{7,8}` |
| 统一社会信用代码 | 18位 | `[0-9A-HJ-NPQRTUWXY]{2}\d{6}[0-9A-HJ-NPQRTUWXY]{10}` |
| 手机号码 | 11位，1开头 | `1[3-9]\d{9}` |
| 车牌号 | 省份简称+字母+5位 | `[京津沪渝冀豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤川青藏琼宁使领][A-Z][A-Z0-9]{4,5}[A-Z0-9挂学警港澳]` |
| 银行卡号 | 16-19位 | `\d{16,19}` (需 Luhn 校验) |

---

### A.2 欧盟 (GDPR - 通用数据保护条例)

**法规依据：** GDPR Article 9 (特殊类别个人数据)

**特殊类别数据：**

| 类别 | 具体数据类型 | 检测规则建议 |
|------|-------------|-------------|
| **种族或民族出身** | 种族背景、民族身份 | NLP 上下文检测 |
| **政治观点** | 政党成员、政治倾向 | NLP 上下文检测 |
| **宗教或哲学信仰** | 宗教团体成员、信仰声明 | NLP 上下文检测 |
| **工会会员资格** | 工会成员身份 | NLP 上下文检测 |
| **基因数据** | DNA 序列、基因检测报告 | 特定格式 |
| **生物识别数据** | 用于身份识别的人脸、指纹等 | 需图像处理 |
| **健康数据** | 病历、诊断、用药、残疾 | 医疗编码 (ICD, SNOMED) |
| **性生活或性取向** | 相关个人数据 | NLP 上下文检测 |

**欧盟通用标识符：**

| 数据类型 | 格式/示例 | 正则模式 |
|----------|----------|----------|
| IBAN | 国家代码+校验码+账号 | `[A-Z]{2}\d{2}[A-Z0-9]{11,30}` |
| SWIFT/BIC | 8或11位 | `[A-Z]{6}[A-Z0-9]{2}([A-Z0-9]{3})?` |
| VAT 号 (德国) | DE + 9位数字 | `DE\d{9}` |
| VAT 号 (法国) | 2字母+11位 | `[A-Z]{2}\d{11}` |
| VAT 号 (意大利) | IT + 11位 | `IT\d{11}` |
| VAT 号 (西班牙) | ES + 9位 | `ES[A-Z]?\d{7,8}[A-Z]?` |
| 欧盟身份证 | 各国格式不同 | 按国家分别处理 |
| 德国身份证 | 9位 | `[1-9]\d{8}` |
| 法国身份证 | 12位 | `\d{12}` |
| 意大利税号 | 16位字母数字 | `[A-Z]{6}\d{2}[A-Z]\d{2}[A-Z]\d{3}[A-Z]` |
| 西班牙 DNI | 8位数字+校验字母 | `\d{8}[A-HJ-NP-TV-Z]` |

---

### A.3 美国 (CCPA/CPRA - 加州消费者隐私法)

**法规依据：** CCPA §1798.140(ae), CPRA

**敏感个人信息类别：**

| 类别 | 具体数据类型 | 检测规则建议 |
|------|-------------|-------------|
| **社会保险号 SSN** | 9位数字，XXX-XX-XXXX 格式 | `\d{3}-\d{2}-\d{4}` |
| **驾照号码** | 各州格式不同 | 按州分别处理 |
| **护照号码** | 9位数字 | `\d{9}` |
| **精确地理位置** | GPS 坐标 | `-?\d{1,3}\.\d+,-?\d{1,3}\.\d+` |
| **种族或族裔** | 相关身份信息 | NLP 上下文检测 |
| **宗教或哲学信仰** | 相关信仰信息 | NLP 上下文检测 |
| **工会会员资格** | 工会成员身份 | NLP 上下文检测 |
| **邮件内容** | 电子邮件通信内容 | 需上下文判断 |
| **基因数据** | DNA 相关数据 | 特定格式 |
| **生物识别信息** | 指纹、人脸、视网膜等 | 需图像处理 |
| **健康信息** | 医疗记录、诊断 | HIPAA 相关 |
| **性生活或性取向** | 相关个人数据 | NLP 上下文检测 |
| **金融账户信息** | 银行账户、信用卡 | `credit_card`, `bank_account` |
| **精确地理位置数据** | 半径 ≤ 1850 英尺 | GPS 坐标精度 |

**美国特有标识符：**

| 数据类型 | 格式/示例 | 正则模式 |
|----------|----------|----------|
| SSN | XXX-XX-XXXX | `\d{3}-\d{2}-\d{4}` |
| ITIN (个人纳税人识别号) | 9XX-XX-XXXX | `9\d{2}-\d{2}-\d{4}` |
| EIN (雇主识别号) | XX-XXXXXXX | `\d{2}-\d{7}` |
| 护照号码 | 9位数字 | `\d{9}` |
| 驾照 (加州) | 1字母+7数字 | `[A-Z]\d{7}` |
| 驾照 (纽约) | 1字母+8数字或9数字 | `[A-Z]?\d{8,9}` |
| 驾照 (德州) | 7-8数字 | `\d{7,8}` |
| 银行路由号 | 9位数字 | `\d{9}` (需校验) |
| ABA 路由号 | 9位，带校验 | `\d{9}` |

---

### A.4 新加坡 (PDPA - 个人数据保护法)

**法规依据：** Personal Data Protection Act 2012

**个人数据定义：** 无论是否真实，能识别个人的数据

**新加坡特有标识符：**

| 数据类型 | 格式/示例 | 正则模式 |
|----------|----------|----------|
| NRIC (身份证) | S/T + 7位数字 + 校验字母 | `[STFG]\d{7}[A-Z]` |
| FIN (外国人身份证) | F/G + 7位数字 + 校验字母 | `[FG]\d{7}[A-Z]` |
| UEN (企业注册号) | 9-10位字母数字 | `\d{8,9}[A-Z]` 或 `[A-Z]\d{8}` |
| 护照号码 | 8位字母数字 | `[A-Z]\d{7,8}` |
| 手机号码 | 8位数字 | `[89]\d{7}` |
| 邮政编码 | 6位数字 | `\d{6}` |

**NRIC/FIN 校验算法：**
```
权重: [2, 7, 6, 5, 4, 3, 2]
前缀 S/T/F/G 对应不同的校验字母表
```

---

### A.5 日本 (APPI - 个人信息保护法)

**法规依据：** Act on the Protection of Personal Information

**要配慮個人情報 (需特别注意的个人信息)：**

| 类别 | 具体数据类型 | 检测规则建议 |
|------|-------------|-------------|
| **种族** | 种族背景 | NLP 上下文检测 |
| **信仰** | 宗教、政治观点 | NLP 上下文检测 |
| **社会身份** | 犯罪记录、受害经历 | NLP 上下文检测 |
| **医疗记录** | 病历、诊断 | ICD 编码 |
| **健康检查** | 体检报告 | 医疗格式 |
| **犯罪记录** | 前科信息 | NLP 上下文检测 |

**日本特有标识符：**

| 数据类型 | 格式/示例 | 正则模式 |
|----------|----------|----------|
| My Number (个人编号) | 12位数字 | `\d{12}` (带校验) |
| 住基卡编号 | 11位数字 | `\d{11}` |
| 护照号码 | 2字母+7数字 | `[A-Z]{2}\d{7}` |
| 驾照号码 | 12位 | `\d{12}` |
| 健康保险证号 | 都道府县代码+编号 | `\d{2}-\d{4,8}` |
| 银行账户 | 银行代码+支店代码+账号 | `\d{4}-\d{3}-\d{7,8}` |
| 邮政编码 | 3-4位 | `\d{3}-\d{4}` |
| 手机号码 | 070/080/090 + 8位 | `0[78]0\d{8}` |

**My Number 校验算法：**
```
权重: [6, 5, 4, 3, 2, 7, 6, 5, 4, 3, 2]
校验位计算: 11 - (加权和 mod 11)
```

---

### A.6 英国 (UK GDPR / DPA 2018)

**法规依据：** UK General Data Protection Regulation, Data Protection Act 2018

**特殊类别数据：** 与 EU GDPR 基本一致

**英国特有标识符：**

| 数据类型 | 格式/示例 | 正则模式 |
|----------|----------|----------|
| NINO (国民保险号) | 2字母+6数字+1字母 | `[A-Z]{2}\d{6}[A-D]` |
| 护照号码 | 9位数字 | `\d{9}` |
| 驾照号码 | 5部分 | `[A-Z]{5}\d{6}[A-Z]{2}\d[A-Z]{2}` |
| NHS 号码 | 10位数字 | `\d{10}` (带校验) |
| 邮政编码 | AA9A 9AA | `[A-Z]{1,2}\d[A-Z\d]?\s?\d[A-Z]{2}` |
| 银行账户 | 8位排序码 + 账号 | `\d{6}-\d{8}` |
| National Insurance Number | AA123456A | `[A-Z]{2}\d{6}[A-D]` |

---

### A.7 韩国 (PIPA - 个人信息保护法)

**法规依据：** Personal Information Protection Act

**韩国特有标识符：**

| 数据类型 | 格式/示例 | 正则模式 |
|----------|----------|----------|
| 주민등록번호 (居民登记号) | 13位数字 | `\d{6}-\d{7}` |
| 外国人登记号 | 13位 | `\d{6}-\d{7}` |
| 护照号码 | 字母+数字 | `[A-Z]\d{8}` |
| 手机号码 | 010-XXXX-XXXX | `010\d{8}` |
| 银行账户 | 各银行格式 | `\d{10,14}` |

**居民登记号校验：**
```
格式: YYMMDD-GHIJKLX
前6位: 出生日期
后7位: 性别+地区码+顺序码+校验位
```

---

### A.8 数据类型检测优先级矩阵

| 数据类型 | 全球通用 | 中国 | 欧盟 | 美国 | 新加坡 | 日本 | 英国 | 韩国 |
|----------|---------|------|------|------|--------|------|------|------|
| 信用卡 | ✅ Luhn | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| IBAN | ✅ | - | ✅ | - | - | - | ✅ | - |
| SWIFT | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Email | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 手机号 | 按地区 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 身份证 | - | ✅ mod11 | 按国 | ✅ SSN | ✅ NRIC | ✅ MyNumber | ✅ NINO | ✅ 13位 |
| 护照 | 按国 | ✅ | 按国 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 驾照 | 按国 | - | 按国 | 按州 | - | ✅ | ✅ | - |
| 税号 | - | 统一信用代码 | VAT | EIN/ITIN | UEN | - | - | - |
| 银行账号 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| GPS 坐标 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| IP 地址 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

---

### A.9 建议新增的校验器

| 校验器名称 | 适用国家/地区 | 算法 |
|-----------|--------------|------|
| `nric_sg` | 新加坡 | NRIC/FIN mod-11 校验 |
| `my_number_jp` | 日本 | My Number mod-11 校验 |
| `nino_uk` | 英国 | NINO 格式校验 |
| `rrn_kr` | 韩国 | 居民登记号 mod-11 校验 |
| `vat_eu` | 欧盟 | 各国 VAT 校验 |
| `aba_us` | 美国 | ABA 路由号校验 |
| `sort_code_uk` | 英国 | 银行排序码校验 |

---

## 附录 B: NLP 系统性能需求分析

### B.1 当前实现概览

**NLP 特性开关：** `cargo build --features nlp`

**依赖库：**
- `candle-core`, `candle-nn`, `candle-transformers` — 纯 Rust ML 框架
- `tokenizers` — HuggingFace 分词器
- `hf-hub` — 模型下载

**支持的模型：**

| 语言 | 模型 | 标签 |
|------|------|------|
| 英文 (en) | `dslim/bert-base-NER` | PER, LOC, ORG, MISC |
| 中文 (zh) | `ckiplab/bert-base-chinese-ner` | PER, LOC, ORG |

### B.2 资源需求

#### 内存需求

| 组件 | 内存占用 | 说明 |
|------|---------|------|
| BERT 模型参数 | ~420 MB | bert-base 模型 |
| 分词器 | ~10 MB | vocab.txt + tokenizer.json |
| 推理中间张量 | ~50-200 MB | 取决于文本长度 |
| **总计** | **~480-630 MB** | 首次加载后常驻内存 |

#### 存储需求

| 组件 | 文件大小 | 缓存位置 |
|------|---------|----------|
| model.safetensors | ~420 MB | `~/.cache/huggingface/hub/` |
| config.json | ~1 KB | 同上 |
| tokenizer.json | ~500 KB | 同上 |
| **总计** | **~420 MB** | 首次运行时下载 |

#### CPU 需求

| 场景 | 推理时间 | 说明 |
|------|---------|------|
| 100 字符文本 | ~50-100 ms | 单次推理 |
| 1KB 文本 | ~100-200 ms | 单次推理 |
| 10KB 文本 | ~1-2 s | 分块并行处理 |
| 100KB 文本 | ~10-20 s | 分块并行处理 |

**注意：** 以上为 CPU 推理时间，无 GPU 加速。

### B.3 性能优化策略

#### 当前已实现的优化

1. **惰性加载**
   - 模型在首次调用时加载
   - 全局 `ModelRegistry` 单例避免重复加载

2. **推理缓存**
   - `InferenceCache` 按文本哈希缓存结果
   - 最大 512 条缓存，超出时清空

3. **分块并行处理**
   - 长文本自动分块（每块 ~512 tokens）
   - 使用 `rayon` 并行推理
   - 块间重叠 50 tokens 避免边界遗漏

4. **智能跳过策略** (pipeline.rs)
   - 文本 < 200 字符：跳过 NLP
   - 无 PII 信号特征：跳过 NLP

#### 建议的额外优化

| 优化项 | 预期收益 | 实现复杂度 |
|--------|---------|-----------|
| GPU 加速 (CUDA/Metal) | 5-10x 加速 | 高 |
| 模型量化 (INT8) | 内存减半，2x 加速 | 中 |
| ONNX Runtime 后端 | 更快的推理 | 中 |
| 更小的模型 (DistilBERT) | 更低延迟 | 低 |

### B.4 最小运行配置建议

**最低配置：**
- CPU: 4 核心
- 内存: 2 GB 可用
- 存储: 500 MB 可用
- 网络: 首次运行需下载模型

**推荐配置：**
- CPU: 8 核心
- 内存: 4 GB 可用
- 存储: 1 GB 可用

### B.5 检测实体类型映射

**NLP 可检测的非结构化实体：**

| EntityType | NER 标签 | 说明 |
|------------|----------|------|
| `PersonName` | B-PER, I-PER | 人名 |
| `Address` | B-LOC, I-LOC | 地址/地点 |
| `Organization` | B-ORG, I-ORG | 组织/机构 |

**正则可检测的结构化实体：**
- 信用卡、身份证、护照、SSN、IBAN、SWIFT 等
- 邮箱、IP、MAC、URL、API Key 等

### B.6 与正则检测的对比

| 维度 | 正则检测 | NLP NER |
|------|---------|---------|
| 速度 | 毫秒级 | 秒级 |
| 内存 | ~10 MB | ~500 MB |
| 准确率 | 高（格式固定） | 中等（依赖模型） |
| 覆盖范围 | 格式化数据 | 自然语言文本 |
| 误报率 | 低 | 中 |
| 适用场景 | 身份证、卡号、邮箱 | 人名、地址、机构名 |

### B.7 启用/禁用建议

**应启用 NLP 的场景：**
- 处理长篇自然语言文本
- 需要检测人名、地址、机构名
- 服务器部署（资源充足）

**应禁用 NLP 的场景：**
- 桌面应用（资源受限）
- 仅处理 API 请求（格式化数据）
- 对延迟敏感的场景

### B.8 配置示例

```toml
# config.toml

[nlp]
enabled = false              # 默认禁用
default_language = "en"      # 语言：en, zh
min_text_length = 200        # 最小文本长度触发 NLP
min_pii_signals = 2          # 最少 PII 信号数触发 NLP
```

---

## 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 向后兼容性破坏 | 高 | 保留旧格式支持，新字段设为可选 |
| 性能下降 | 中 | 基准测试，优化热点路径 |
| 检测精度下降 | 高 | 对比测试，保留原有正则 |
| 规则文件迁移困难 | 中 | 提供迁移脚本，保留旧目录支持 |
