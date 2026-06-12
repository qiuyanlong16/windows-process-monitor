# Windows Performance Monitor Widget Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a compact, always-on-top Windows desktop widget that displays real-time CPU and memory usage, with optional specific-process monitoring.

**Architecture:** Two-layer Rust app — a background thread polls system stats via `sysinfo` every 1 second and writes to `Arc<RwLock<Stats>>`, while the main thread runs an `egui` window that reads the shared state and renders stats at ~30fps. Config is persisted as JSON in the user's home directory.

**Tech Stack:** Rust, egui + eframe, sysinfo, serde + serde_json

---

### File Structure

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Project manifest with dependencies |
| `src/main.rs` | Entry point — initializes egui app, passes shared state |
| `src/lib.rs` | Module declarations |
| `src/config.rs` | `AppConfig` struct, JSON read/write, default config, window position persistence |
| `src/stats.rs` | `Stats` and `ProcessStats` structs, `StatsCollector` background thread, sysinfo polling |
| `src/app.rs` | `MonitorApp` egui struct — window setup, rendering logic, drag handling |
| `tests/config_test.rs` | Config serialization/deserialization tests |

---

### Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

- [ ] **Step 1: Initialize the Rust project**

Run: `cd d:/workspace/monitor-mem-special-sw && cargo init --name monitor-widget`

- [ ] **Step 2: Write `Cargo.toml` with dependencies**

```toml
[package]
name = "monitor-widget"
version = "0.1.0"
edition = "2021"

[dependencies]
egui = "0.31"
eframe = "0.31"
sysinfo = "0.33"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 3: Write `src/lib.rs` with module declarations**

```rust
pub mod config;
pub mod stats;
pub mod app;
```

- [ ] **Step 4: Write minimal `src/main.rs`**

```rust
mod app;
mod config;
mod stats;

fn main() {
    println!("monitor-widget starting up");
}
```

- [ ] **Step 5: Verify project compiles**

Run: `cargo check`
Expected: PASS (no errors)

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs src/lib.rs
git commit -m "chore: scaffold Rust project with egui/sysinfo/serde dependencies"
```

---

### Task 2: Config Module with Tests (TDD)

**Files:**
- Create: `src/config.rs`
- Create: `tests/config_test.rs`

- [ ] **Step 1: Write config tests first**

Create `tests/config_test.rs`:

```rust
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
    // Should parse back
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test config_test`
Expected: FAIL — `config` module doesn't exist yet

- [ ] **Step 3: Implement `src/config.rs`**

```rust
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
```

- [ ] **Step 4: Export config from `src/lib.rs`**

Update `src/lib.rs`:

```rust
pub mod config;
pub mod stats;
pub mod app;
```

(Already declared, no change needed if Task 1's lib.rs already has this.)

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test config_test`
Expected: PASS (all 5 tests)

- [ ] **Step 6: Commit**

```bash
git add src/config.rs tests/config_test.rs
git commit -m "feat: add config module with JSON persistence and TDD tests"
```

---

### Task 3: Stats Module

**Files:**
- Create: `src/stats.rs`

- [ ] **Step 1: Write `src/stats.rs` with Stats structs and collector**

```rust
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStats {
    pub name: String,
    pub pid: u32,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub cpu_percent: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub watched_process: Option<ProcessStats>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            cpu_percent: 0.0,
            mem_used: 0,
            mem_total: 0,
            watched_process: None,
        }
    }
}

pub type SharedStats = Arc<RwLock<Stats>>;

pub fn create_shared_stats() -> SharedStats {
    Arc::new(RwLock::new(Stats::new()))
}

pub fn spawn_stats_collector(shared: SharedStats, watch_process: Option<String>) {
    std::thread::spawn(move || {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_memory(sysinfo::MemoryRefreshKind::new().with_ram())
                .with_cpu(sysinfo::CpuRefreshKind::new().with_cpu_usage()),
        );

        // First refresh to seed data (CPU usage needs a delta measurement)
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        std::thread::sleep(std::time::Duration::from_millis(500));

        loop {
            sys.refresh_cpu_usage();
            sys.refresh_memory();

            if watch_process.is_some() {
                sys.refresh_processes_specifics(
                    ProcessRefreshKind::new().with_cpu().with_memory(),
                );
            }

            let mut stats = shared.write().unwrap();
            stats.cpu_percent = sys.cpu_usage();
            stats.mem_used = sys.used_memory();
            stats.mem_total = sys.total_memory();

            if let Some(ref proc_name) = watch_process {
                stats.watched_process = find_process(&sys, proc_name);
            }

            drop(stats);
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

fn find_process(sys: &System, name: &str) -> Option<ProcessStats> {
    let name_lower = name.to_lowercase();
    sys.processes()
        .values()
        .filter(|p| {
            p.name()
                .to_string_lossy()
                .to_lowercase()
                == name_lower
        })
        .max_by_key(|p| p.memory())
        .map(|p| ProcessStats {
            name: name.to_string(),
            pid: p.pid().as_u32(),
            cpu_percent: p.cpu_usage(),
            memory_bytes: p.memory(),
        })
}
```

- [ ] **Step 2: Verify project compiles**

Run: `cargo check`
Expected: PASS (no errors)

- [ ] **Step 3: Commit**

```bash
git add src/stats.rs
git commit -m "feat: add stats module with sysinfo polling and process lookup"
```

---

### Task 4: egui App Module

**Files:**
- Create: `src/app.rs`

- [ ] **Step 1: Write `src/app.rs` with the egui app**

```rust
use crate::config::{self, AppConfig};
use crate::stats::SharedStats;
use eframe::egui;

pub struct MonitorApp {
    shared: SharedStats,
    config_path: std::path::PathBuf,
    config: AppConfig,
    dragging_offset: Option<egui::Vec2>,
}

impl MonitorApp {
    pub fn new(cc: &eframe::CreationContext<'_>, shared: SharedStats) -> Self {
        // Style setup
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.visuals.window_rounding = egui::Rounding::same(12.0);
        style.visuals.window_fill = egui::Color32::from_rgba_unmultiplied(30, 30, 40, 217); // ~0.85 alpha
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_unmultiplied(40, 40, 55, 180);
        style.text_styles.get_mut(&egui::TextStyle::Body).unwrap().size = 13.0;
        style.text_styles.get_mut(&egui::TextStyle::Small).unwrap().size = 11.0;
        cc.egui_ctx.set_style(style);

        let config_path = config::default_config_path();
        let config = config::read_config(&config_path).unwrap_or_else(|_| config::default_config());

        Self {
            shared,
            config_path,
            config,
            dragging_offset: None,
        }
    }
}

impl eframe::App for MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::Frame::none()
            .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 40, 217))
            .rounding(12.0)
            .inner_margin(12.0);

        egui::Window::new("")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .frame(frame)
            .default_width(220.0)
            .fixed_size([220.0, 100.0])
            .show(ctx, |ui| {
                self.render(ui);
            });

        ctx.request_repaint_after_secs(1.0 / 30.0);
    }
}

impl MonitorApp {
    fn render(&mut self, ui: &mut egui::Ui) {
        let stats = self.shared.read().unwrap();

        // Drag handle area
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("────").size(8.0).color(egui::Color32::GRAY));
            ui.allocate_space(ui.available_width());
        });

        ui.add_space(4.0);

        // System-wide stats
        render_bar_row(ui, "CPU", stats.cpu_percent);
        render_bar_row(ui, "MEM", mem_percent(&stats));

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "{} / {}",
                format_bytes(stats.mem_used),
                format_bytes(stats.mem_total)
            ))
            .size(11.0)
            .color(egui::Color32::GRAY),
        );

        // Process-specific stats
        if stats.watched_process.is_some() {
            ui.separator();
        }
        if let Some(ref proc_stats) = stats.watched_process {
            ui.label(egui::RichText::new(&proc_stats.name).size(12.0).color(egui::Color32::from_rgb(150, 180, 255)));
            render_bar_row(ui, "CPU", proc_stats.cpu_percent);
            ui.label(
                egui::RichText::new(format!("MEM  {}", format_bytes(proc_stats.memory_bytes)))
                    .size(13.0),
            );
        } else if self.config.watch_process.is_some() {
            ui.label(egui::RichText::new("Not running").size(11.0).color(egui::Color32::GRAY));
        }
    }
}

fn render_bar_row(ui: &mut egui::Ui, label: &str, percent: f32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).size(13.0).color(egui::Color32::GRAY));
        ui.label(egui::RichText::new(format!("{:.0}%", percent.clamp(0.0, 100.0))).size(13.0));
        ui.allocate_space(egui::vec2(8.0, 0.0));
        let filled = (percent.clamp(0.0, 100.0) / 100.0 * 10.0).ceil() as usize;
        let bar: String = "█".repeat(filled) + &"░".repeat(10 - filled);
        ui.label(egui::RichText::new(bar).color(egui::Color32::from_rgb(100, 160, 255)).size(13.0));
    });
}

fn mem_percent(stats: &crate::stats::Stats) -> f32 {
    if stats.mem_total == 0 {
        0.0
    } else {
        (stats.mem_used as f64 / stats.mem_total as f64 * 100.0) as f32
    }
}

fn format_bytes(bytes: u64) -> String {
    let gb = bytes as f64 / 1_073_741_824.0;
    if gb >= 1.0 {
        format!("{:.1} GB", gb)
    } else {
        format!("{:.0} MB", bytes as f64 / 1_048_576.0)
    }
}
```

- [ ] **Step 2: Update `src/main.rs` to wire everything together**

```rust
mod app;
mod config;
mod stats;

use app::MonitorApp;
use config;
use eframe::egui;
use stats::create_shared_stats;

fn main() -> eframe::Result {
    let config_path = config::default_config_path();
    let config = config::read_config(&config_path).unwrap_or_else(|_| config::default_config());

    let shared = create_shared_stats();
    stats::spawn_stats_collector(shared.clone(), config.watch_process.clone());

    let position = match (config.window_x, config.window_y) {
        (Some(x), Some(y)) => egui::Pos2::new(x as f32, y as f32),
        _ => egui::Pos2::new(1700.0, 10.0), // Default: top-right
    };

    let viewport = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_position(position),
        ..Default::default()
    };

    eframe::run_native(
        "Monitor Widget",
        viewport,
        Box::new(|cc| Ok(Box::new(MonitorApp::new(cc, shared)))),
    )
}
```

- [ ] **Step 3: Verify project compiles**

Run: `cargo check`
Expected: PASS (warnings are OK, no errors)

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: add egui app with styled widget rendering CPU/MEM stats"
```

---

### Task 5: Polish and Run

**Files:**
- Modify: `src/main.rs` (if needed)
- Modify: `src/app.rs` (if needed)

- [ ] **Step 1: Full build**

Run: `cargo build --release`
Expected: PASS, produces `target/release/monitor-widget.exe`

- [ ] **Step 2: Run all tests**

Run: `cargo test`
Expected: PASS (all config tests)

- [ ] **Step 3: Smoke test — launch the widget**

Run: `cargo run --release`
Expected: A small window appears in top-right corner showing CPU% and MEM%, updates every second

- [ ] **Step 4: Test process monitoring**

Create `%USERPROFILE%/.monitor-config.json` with:
```json
{
  "watch_process": "explorer.exe"
}
```

Run: `cargo run --release`
Expected: Window also shows explorer.exe CPU and MEM stats

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: final polish, verify build and tests pass"
```
