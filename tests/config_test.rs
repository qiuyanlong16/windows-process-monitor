use monitor_widget::config::{AppConfig, read_config, write_config, default_config};
use std::fs;
use std::path::PathBuf;

fn temp_config_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("monitor-test-{}.json", name));
    path
}

#[test]
fn test_default_config_has_no_watch_process() {
    let config = default_config();
    assert!(config.watch_process.is_none());
    assert!(config.window_x.is_none());
    assert!(config.window_y.is_none());
}

#[test]
fn test_default_config_serializes_to_valid_json() {
    let config = default_config();
    let json = serde_json::to_string(&config).unwrap();
    let parsed: AppConfig = serde_json::from_str(&json).unwrap();
    assert!(parsed.watch_process.is_none());
}

#[test]
fn test_config_with_watch_process_roundtrip() {
    let mut config = default_config();
    config.watch_process = Some("chrome.exe".to_string());
    config.window_x = Some(1700.0);
    config.window_y = Some(10.0);

    let path = temp_config_path("roundtrip");
    write_config(&config, &path).unwrap();

    let loaded = read_config(&path).unwrap();
    assert_eq!(loaded.watch_process, Some("chrome.exe".to_string()));
    assert_eq!(loaded.window_x, Some(1700.0));
    assert_eq!(loaded.window_y, Some(10.0));

    fs::remove_file(&path).ok();
}

#[test]
fn test_read_config_missing_file_returns_default() {
    let path = temp_config_path("nonexistent");
    let config = read_config(&path).unwrap();
    assert_eq!(config, default_config());
}

#[test]
fn test_read_config_invalid_json_returns_default() {
    let path = temp_config_path("invalid");
    fs::write(&path, "not json").unwrap();
    let config = read_config(&path).unwrap();
    assert_eq!(config, default_config());
    fs::remove_file(&path).ok();
}
