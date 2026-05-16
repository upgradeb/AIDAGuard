//! Plugin ABI definitions for dynamic loading.
//!
//! Defines C ABI compatible structures for cross-language plugin support.
//! Plugins must implement the PluginVTable interface.

use std::ffi::c_int;

/// Plugin ABI version. Must match between host and plugin.
pub const ABI_VERSION: u32 = 1;

/// Plugin metadata (C ABI compatible).
#[repr(C)]
pub struct PluginMeta {
    /// Plugin ID (null-terminated C string)
    pub id: *const i8,
    /// Plugin name
    pub name: *const i8,
    /// Version string
    pub version: *const i8,
    /// Description
    pub description: *const i8,
    /// Author
    pub author: *const i8,
}

/// Plugin virtual function table (C ABI).
///
/// All plugins must export a function `plugin_vtable()` returning this struct.
#[repr(C)]
pub struct PluginVTable {
    // ── Metadata ──

    /// Get plugin ID
    pub id: unsafe extern "C" fn() -> *const i8,

    /// Get plugin name
    pub name: unsafe extern "C" fn() -> *const i8,

    /// Get ABI version
    pub abi_version: unsafe extern "C" fn() -> u32,

    // ── Lifecycle ──

    /// Initialize plugin. Returns 0 on success.
    pub init: unsafe extern "C" fn() -> c_int,

    /// Cleanup plugin resources
    pub cleanup: unsafe extern "C" fn(),

    // ── Detection ──

    /// Check if the tool is installed
    pub detect: unsafe extern "C" fn() -> bool,

    // ── Configuration ──

    /// Get config file path
    pub config_path: unsafe extern "C" fn() -> *const i8,

    /// Get current API endpoint
    pub current_endpoint: unsafe extern "C" fn() -> *const i8,

    /// Configure proxy URL. Returns 0 on success.
    pub configure: unsafe extern "C" fn(proxy_url: *const i8) -> c_int,

    /// Restore original config. Returns 0 on success.
    pub restore: unsafe extern "C" fn() -> c_int,

    // ── State ──

    /// Check if proxy is configured
    pub is_configured: unsafe extern "C" fn() -> bool,

    /// Get current model name
    pub current_model: unsafe extern "C" fn() -> *const i8,
}

impl PluginVTable {
    /// Check if this vtable is compatible with current ABI.
    pub fn check_abi(&self) -> bool {
        unsafe { (self.abi_version)() == ABI_VERSION }
    }
}
