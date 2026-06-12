use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub watch_process: Option<String>,
    pub window_x: Option<f64>,
    pub window_y: Option<f64>,
}

pub fn default_config() -> AppConfig {
    AppConfig {
        watch_process: None,
        window_x: None,
        window_y: None,
    }
}

pub fn read_config(path: &Path) -> Result<AppConfig, String> {
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(config) => Ok(config),
            Err(_) => Ok(default_config()),
        },
        Err(_) => Ok(default_config()),
    }
}

#[allow(dead_code)]
pub fn write_config(config: &AppConfig, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(path, json)
        .map_err(|e| format!("Failed to write config: {}", e))
}

pub fn default_config_path() -> std::path::PathBuf {
    let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home).join(".monitor-config.json")
}
