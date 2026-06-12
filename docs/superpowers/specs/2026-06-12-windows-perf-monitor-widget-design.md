---
name: Windows Performance Monitor Widget Design
description: Design for a compact desktop widget showing real-time CPU/memory stats
type: design
---

# Windows Performance Monitor Widget

## Overview

A small, always-on-top floating window for Windows that displays real-time CPU and memory usage, with optional monitoring of a specific process by name. Built in Rust using egui for the GUI and sysinfo for system stats.

## Architecture

### High-Level Structure

Two-layer architecture running in a single process:

1. **StatsCollector (background thread)** — Polls Windows system counters every 1 second using the `sysinfo` crate. Writes results to shared state.
2. **egui Window (main thread)** — Borderless, always-on-top, draggable window that reads shared state and renders stats at ~30fps.

Communication is through `Arc<RwLock<Stats>>` — simple shared state, no message passing.

### Data Flow

```
sysinfo → StatsCollector (every 1s) → Arc<RwLock<Stats>> → egui update() → render
```

1. App starts, spawns StatsCollector thread
2. Collector initializes `sysinfo::System`, seeds data
3. Each 1-second tick: refreshes stats, writes to `Arc<RwLock<Stats>>`
4. egui reads current stats each frame, renders bars and labels
5. Window drag position saved to config file on each frame change

## GUI Layer

### Window Properties

- Borderless, no title bar, no system chrome
- Semi-transparent dark background (alpha ~0.85)
- Rounded corners
- Fixed size: ~220×100px (minimum), scales if process monitoring is active
- Always on top
- Entire window is draggable
- Position persisted to `~/.monitor-config.json`

### Visual Layout

**System-wide only (no process watch):**

```
┌──────────────────────────────┐
│  ────                        │
│                              │
│  CPU  23%  ████░░░░░░░░░░   │
│  MEM  45%  ██████░░░░░░░░   │
│                              │
│  12.4 GB / 32.0 GB          │
└──────────────────────────────┘
```

**With process monitoring active:**

```
┌──────────────────────────────────┐
│  ────                            │
│                                  │
│  CPU  23%  ████░░░░░░░░░░       │
│  MEM  45%  ██████░░░░░░░░       │
│                                  │
│  chrome.exe                      │
│    CPU  8%  ██░░░░░░░░░░        │
│    MEM  312 MB                   │
└──────────────────────────────────┘
```

### Styling

- Dark theme matching Windows 11 Fluent Design
- Segoe UI Variable font (system default)
- Progress bars with subtle blue accent color
- Semi-transparent dark background with slight blur effect

## Data Models

```rust
struct Stats {
    // System-wide
    cpu_percent: f32,
    mem_used: u64,
    mem_total: u64,

    // Specific process (from config)
    watched_process: Option<ProcessStats>,
}

struct ProcessStats {
    name: String,
    pid: u32,
    cpu_percent: f32,
    memory_bytes: u64,
}
```

```rust
struct AppConfig {
    watch_process: Option<String>,  // e.g. "chrome.exe"
    window_x: Option<f64>,
    window_y: Option<f64>,
}
```

## Configuration

Config file at `%USERPROFILE%/.monitor-config.json`:

```json
{
  "watch_process": null,
  "window_x": 1700.0,
  "window_y": 10.0
}
```

- `watch_process`: Executable name to monitor (case-insensitive). Null/missing = system-wide only.
- `window_x`, `window_y`: Last window position. Falls back to top-right corner on first run.
- Config is read at startup, written on window position changes.

## Error Handling

- If sysinfo cannot read CPU/memory info: display "N/A" for that stat
- If config file read fails: use defaults, no error shown
- If config file write fails: silently ignore
- If watched process is not running: display "Not running" in process section
- No user-facing error dialogs — passive monitoring tool, never interrupts

## Dependencies

| Crate | Purpose |
|-------|---------|
| `egui` + `eframe` | GUI framework |
| `sysinfo` | System stats collection (CPU, memory, processes) |
| `serde` + `serde_json` | Config file persistence |

~4 direct dependencies.

## Non-Goals (for this version)

- Per-process graph history / sparklines
- Alert thresholds or notifications
- Network or disk I/O monitoring
- Interactive controls (refresh rate, colors, etc.)
- Multiple process monitoring
