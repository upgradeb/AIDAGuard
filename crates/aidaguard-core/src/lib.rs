pub mod proxy;
pub mod detector;
pub mod replacer;
pub mod storage;

/// Aidaguard core version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
