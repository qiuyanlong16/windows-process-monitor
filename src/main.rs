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
        _ => egui::Pos2::new(1700.0, 10.0),
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
        Box::new(|cc| Ok(Box::new(MonitorApp::new(cc, shared, config)))),
    )
}
