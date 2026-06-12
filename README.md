# Monitor Widget

A compact, always-on-top Windows desktop widget that shows real-time CPU and memory usage. Optionally monitors a specific process by executable name.

Built with Rust, [egui](https://github.com/emilk/egui), and [sysinfo](https://github.com/GuillaumeGomez/sysinfo).

## Features

- System-wide CPU and memory usage with progress bars
- Optional per-process monitoring (e.g. `chrome.exe`, `by-claw.exe`)
- Borderless dark-themed floating window
- Draggable custom title bar with close button
- Always on top, hidden from taskbar
- Window position persisted to config file

## Requirements

- Windows 10/11
- [Rust](https://rustup.rs/) (edition 2021)

## Build & Run

```bash
cargo build --release
cargo run
```

Release binary: `target/release/monitor-widget.exe`

## Configuration

Config file: `%USERPROFILE%\.monitor-config.json`

```json
{
  "watch_process": "by-claw.exe",
  "window_x": 950.0,
  "window_y": 50.0
}
```

| Field | Description |
|-------|-------------|
| `watch_process` | Executable name to monitor (case-insensitive). `null` = system stats only |
| `window_x` | Last window X position |
| `window_y` | Last window Y position |

If the config file is missing or invalid, defaults are used.

## Project Structure

```
src/
  main.rs    # Entry point, viewport setup
  app.rs     # egui UI, title bar, stats rendering
  stats.rs   # Background sysinfo polling thread
  config.rs  # JSON config read/write
tests/
  config_test.rs
```

## License

Internal use — Lenovo Huishang / Baiying AI.
