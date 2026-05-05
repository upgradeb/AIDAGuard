use std::io::Read;

fn main() {
    // Sync version from Cargo.toml into tauri.conf.json at build time
    let version = env!("CARGO_PKG_VERSION");
    let config_path = std::path::Path::new("tauri.conf.json");
    if config_path.exists() {
        let mut content = String::new();
        std::fs::File::open(config_path)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        let mut config: serde_json::Value =
            serde_json::from_str(&content).unwrap();
        let current = config.get("version").and_then(|v| v.as_str()).unwrap_or("");
        if current != version {
            if let Some(obj) = config.as_object_mut() {
                obj.insert(
                    "version".to_string(),
                    serde_json::Value::String(version.to_string()),
                );
            }
            let out = serde_json::to_string_pretty(&config).unwrap();
            std::fs::write(config_path, out + "\n").unwrap();
        }
    }

    tauri_build::build()
}
