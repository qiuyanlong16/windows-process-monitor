use crate::config::AppConfig;
use crate::stats::SharedStats;
use eframe::egui;

pub struct MonitorApp {
    shared: SharedStats,
    config: AppConfig,
}

impl MonitorApp {
    pub fn new(cc: &eframe::CreationContext<'_>, shared: SharedStats, config: AppConfig) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.visuals.window_corner_radius = egui::CornerRadius::same(12);
        style.visuals.window_fill = egui::Color32::from_rgba_unmultiplied(30, 30, 40, 217);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_unmultiplied(40, 40, 55, 180);
        style.text_styles.get_mut(&egui::TextStyle::Body).unwrap().size = 13.0;
        style.text_styles.get_mut(&egui::TextStyle::Small).unwrap().size = 11.0;
        cc.egui_ctx.set_style(style);

        Self {
            shared,
            config,
        }
    }
}

impl eframe::App for MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::Frame::NONE
            .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 40, 217))
            .corner_radius(12)
            .inner_margin(12);

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

        ctx.request_repaint_after(std::time::Duration::from_secs_f32(1.0 / 30.0));
    }
}

impl MonitorApp {
    fn render(&mut self, ui: &mut egui::Ui) {
        let stats = self.shared.read().unwrap();

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("\u{2500}\u{2500}\u{2500}\u{2500}").size(8.0).color(egui::Color32::GRAY));
            ui.allocate_space(egui::vec2(ui.available_width(), 0.0));
        });

        ui.add_space(4.0);

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
        let filled = filled.min(10);
        let bar: String = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(10 - filled);
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
