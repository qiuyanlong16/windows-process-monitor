mod app;
mod config;
mod stats;

use app::MonitorApp;
use eframe::egui;
use stats::create_shared_stats;

fn main() -> eframe::Result {
    let config_path = config::default_config_path();
    let config = config::read_config(&config_path).unwrap_or_else(|_| config::default_config());

    let shared = create_shared_stats();
    stats::spawn_stats_collector(shared.clone(), config.watch_process.clone());

    let position = match (config.window_x, config.window_y) {
        (Some(x), Some(y)) => egui::Pos2::new(x as f32, y as f32),
        _ => egui::Pos2::new(950.0, 50.0),
    };

    let viewport = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("")
            .with_always_on_top()
            .with_decorations(false)
            .with_resizable(false)
            .with_inner_size([240.0, 220.0])
            .with_position(position)
            .with_active(true)
            .with_taskbar(false),
        ..Default::default()
    };

    eframe::run_native(
        "Monitor Widget",
        viewport,
        Box::new(|cc| Ok(Box::new(MonitorApp::new(cc, shared, config)))),
    )
}
