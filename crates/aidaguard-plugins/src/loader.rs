//! Dynamic plugin loader.
//!
//! Loads plugins from `.dylib`/`.so`/`.dll` files at runtime.
//! This is an optional feature that must be explicitly enabled.

use crate::abi::{PluginVTable, ABI_VERSION};
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Plugin loading error
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Library not found: {0}")]
    LibraryNotFound(String),

    #[error("Failed to load library: {0}")]
    LoadFailed(#[from] libloading::Error),

    #[error("ABI version mismatch: expected {expected}, got {actual}")]
    AbiMismatch { expected: u32, actual: u32 },

    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),

    #[error("Configuration failed for plugin {0}")]
    ConfigureFailed(String),

    #[error("Restore failed for plugin {0}")]
    RestoreFailed(String),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Plugin manifest loaded from JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub min_aidaguard_version: String,
    pub categories: Vec<String>,
}

/// A dynamically loaded plugin
pub struct DynamicPlugin {
    /// Library handle (kept alive for the plugin's lifetime)
    #[allow(dead_code)]
    library: Library,
    /// Virtual function table
    vtable: PluginVTable,
    /// Parsed manifest
    manifest: DynamicManifest,
}

impl DynamicPlugin {
    /// Get plugin ID
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    /// Get plugin name
    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    /// Get manifest
    pub fn manifest(&self) -> &DynamicManifest {
        &self.manifest
    }

    /// Check if the tool is installed
    pub fn detect(&self) -> bool {
        unsafe { (self.vtable.detect)() }
    }

    /// Configure proxy
    pub fn configure(&self, proxy_url: &str) -> Result<(), PluginError> {
        let url = std::ffi::CString::new(proxy_url)
            .map_err(|_| PluginError::ConfigureFailed(self.manifest.id.clone()))?;
        let result = unsafe { (self.vtable.configure)(url.as_ptr()) };
        if result == 0 {
            Ok(())
        } else {
            Err(PluginError::ConfigureFailed(self.manifest.id.clone()))
        }
    }

    /// Restore original configuration
    pub fn restore(&self) -> Result<(), PluginError> {
        let result = unsafe { (self.vtable.restore)() };
        if result == 0 {
            Ok(())
        } else {
            Err(PluginError::RestoreFailed(self.manifest.id.clone()))
        }
    }

    /// Check if proxy is configured
    pub fn is_configured(&self) -> bool {
        unsafe { (self.vtable.is_configured)() }
    }

    /// Get current endpoint
    pub fn current_endpoint(&self) -> Option<String> {
        unsafe {
            let ptr = (self.vtable.current_endpoint)();
            if ptr.is_null() {
                return None;
            }
            Some(
                std::ffi::CStr::from_ptr(ptr)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }

    /// Get current model
    pub fn current_model(&self) -> Option<String> {
        unsafe {
            let ptr = (self.vtable.current_model)();
            if ptr.is_null() {
                return None;
            }
            Some(
                std::ffi::CStr::from_ptr(ptr)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }

    /// Get config path
    pub fn config_path(&self) -> Option<String> {
        unsafe {
            let ptr = (self.vtable.config_path)();
            if ptr.is_null() {
                return None;
            }
            Some(
                std::ffi::CStr::from_ptr(ptr)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }
}

/// Plugin loader for dynamic libraries
pub struct PluginLoader {
    /// Plugin directory
    plugin_dir: PathBuf,
    /// Loaded plugins
    plugins: HashMap<String, DynamicPlugin>,
}

impl PluginLoader {
    /// Create a new loader
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugin_dir,
            plugins: HashMap::new(),
        }
    }

    /// Scan and load all plugins from directory
    pub fn scan_and_load(&mut self) -> Result<Vec<String>, PluginError> {
        let mut loaded = Vec::new();

        if !self.plugin_dir.exists() {
            std::fs::create_dir_all(&self.plugin_dir)?;
            return Ok(loaded);
        }

        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let path = entry?.path();

            if path.is_dir() {
                if let Ok(id) = self.load_from_dir(&path) {
                    loaded.push(id);
                }
            } else if Self::is_library(&path) {
                if let Ok(id) = self.load(&path) {
                    loaded.push(id);
                }
            }
        }

        Ok(loaded)
    }

    /// Load plugin from directory (expects manifest.json + library file)
    fn load_from_dir(&mut self, dir: &Path) -> Result<String, PluginError> {
        let lib_path = self.find_library(dir)?;
        let manifest_path = dir.join("manifest.json");

        let manifest = if manifest_path.exists() {
            let content = std::fs::read_to_string(&manifest_path)?;
            serde_json::from_str(&content)
                .map_err(|e| PluginError::InvalidManifest(e.to_string()))?
        } else {
            // Generate manifest from library metadata
            self.load_manifest_from_library(&lib_path)?
        };

        self.load_with_manifest(&lib_path, manifest)
    }

    /// Load a single library file
    fn load(&mut self, path: &Path) -> Result<String, PluginError> {
        let manifest = self.load_manifest_from_library(path)?;
        self.load_with_manifest(path, manifest)
    }

    /// Load library with manifest
    fn load_with_manifest(
        &mut self,
        lib_path: &Path,
        manifest: DynamicManifest,
    ) -> Result<String, PluginError> {
        unsafe {
            let library = Library::new(lib_path)?;

            let get_vtable: Symbol<unsafe extern "C" fn() -> PluginVTable> =
                library.get(b"plugin_vtable")?;

            let vtable = get_vtable();

            // Verify ABI
            let abi = (vtable.abi_version)();
            if abi != ABI_VERSION {
                return Err(PluginError::AbiMismatch {
                    expected: ABI_VERSION,
                    actual: abi,
                });
            }

            // Initialize
            let init_result = (vtable.init)();
            if init_result != 0 {
                return Err(PluginError::InitFailed(manifest.id.clone()));
            }

            let id = manifest.id.clone();
            let plugin = DynamicPlugin {
                library,
                vtable,
                manifest,
            };

            self.plugins.insert(id.clone(), plugin);
            Ok(id)
        }
    }

    /// Extract manifest from library metadata
    fn load_manifest_from_library(&self, lib_path: &Path) -> Result<DynamicManifest, PluginError> {
        unsafe {
            let library = Library::new(lib_path)?;

            let get_vtable: Symbol<unsafe extern "C" fn() -> PluginVTable> =
                library.get(b"plugin_vtable")?;

            let vtable = get_vtable();

            let id = std::ffi::CStr::from_ptr((vtable.id)())
                .to_string_lossy()
                .into_owned();
            let name = std::ffi::CStr::from_ptr((vtable.name)())
                .to_string_lossy()
                .into_owned();

            Ok(DynamicManifest {
                id,
                name: name.clone(),
                version: "0.1.0".to_string(),
                description: format!("{} plugin", name),
                author: "Unknown".to_string(),
                min_aidaguard_version: "0.4.0".to_string(),
                categories: vec![],
            })
        }
    }

    /// Check if path is a dynamic library
    fn is_library(path: &Path) -> bool {
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

    /// Find library file in directory
    fn find_library(&self, dir: &Path) -> Result<PathBuf, PluginError> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if Self::is_library(&path) {
                return Ok(path);
            }
        }
        Err(PluginError::LibraryNotFound(dir.display().to_string()))
    }

    /// Get loaded plugin by ID
    pub fn get(&self, id: &str) -> Option<&DynamicPlugin> {
        self.plugins.get(id)
    }

    /// List all loaded plugin IDs
    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }

    /// Get number of loaded plugins
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}
