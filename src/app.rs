use crate::config::AppConfig;
use crate::stats::SharedStats;
use eframe::egui;

const BG: egui::Color32 = egui::Color32::from_rgb(30, 30, 40);
const TITLE_BAR_BG: egui::Color32 = egui::Color32::from_rgb(40, 40, 55);
const TITLE_BAR_HEIGHT: f32 = 22.0;

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
        style.visuals.widgets.noninteractive.bg_fill =
            egui::Color32::from_rgba_unmultiplied(40, 40, 55, 180);
        style.text_styles.get_mut(&egui::TextStyle::Body).unwrap().size = 13.0;
        style.text_styles.get_mut(&egui::TextStyle::Small).unwrap().size = 11.0;
        cc.egui_ctx.set_style(style);

        Self { shared, config }
    }
}

impl eframe::App for MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("title_bar")
            .exact_height(TITLE_BAR_HEIGHT)
            .frame(
                egui::Frame::NONE
                    .fill(TITLE_BAR_BG)
                    .inner_margin(egui::Margin::symmetric(8, 0)),
            )
            .show(ctx, |ui| {
                self.render_title_bar(ui, ctx);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(BG)
                    .inner_margin(egui::Margin::symmetric(12, 8)),
            )
            .show(ctx, |ui| {
                self.render(ui);
            });

        ctx.request_repaint_after(std::time::Duration::from_secs_f32(1.0 / 30.0));
    }
}

impl MonitorApp {
    fn render_title_bar(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let bar_rect = ui.max_rect();
        let drag_rect = egui::Rect::from_min_max(
            bar_rect.min,
            egui::pos2(bar_rect.right() - 28.0, bar_rect.max.y),
        );

        let drag_response = ui.interact(
            drag_rect,
            ui.make_persistent_id("title_drag"),
            egui::Sense::drag(),
        );
        if drag_response.drag_started() {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if drag_response.hovered() || drag_response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }

        paint_drag_grip(
            ui.painter(),
            drag_rect.left_center() + egui::vec2(12.0, 0.0),
            egui::Color32::from_gray(100),
        );

        let close_rect = egui::Rect::from_min_size(
            egui::pos2(bar_rect.right() - 20.0, bar_rect.center().y - 10.0),
            egui::vec2(20.0, 20.0),
        );
        let close_response = ui.interact(close_rect, ui.make_persistent_id("close_btn"), egui::Sense::click());
        let close_color = if close_response.hovered() {
            egui::Color32::from_rgb(220, 80, 80)
        } else {
            egui::Color32::from_gray(160)
        };
        ui.painter().line_segment(
            [
                close_rect.center() + egui::vec2(-4.0, -4.0),
                close_rect.center() + egui::vec2(4.0, 4.0),
            ],
            egui::Stroke::new(1.5, close_color),
        );
        ui.painter().line_segment(
            [
                close_rect.center() + egui::vec2(4.0, -4.0),
                close_rect.center() + egui::vec2(-4.0, 4.0),
            ],
            egui::Stroke::new(1.5, close_color),
        );
        if close_response.clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        let stats = self.shared.read().unwrap();

        ui.separator();
        ui.add_space(4.0);

        render_bar_row(ui, "CPU", stats.cpu_percent);
        render_bar_row(ui, "MEM", mem_percent(&stats));

        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(format!(
                "{} / {}",
                format_bytes(stats.mem_used),
                format_bytes(stats.mem_total)
            ))
            .size(11.0)
            .color(egui::Color32::from_gray(130)),
        );

        if stats.watched_process.is_some() {
            ui.add_space(4.0);
            ui.separator();
        }
        if let Some(ref proc_stats) = stats.watched_process {
            ui.label(
                egui::RichText::new(&proc_stats.name)
                    .size(12.0)
                    .color(egui::Color32::from_rgb(150, 180, 255)),
            );
            render_bar_row_precise(ui, "CPU", proc_stats.cpu_percent);
            ui.label(
                egui::RichText::new(format!(
                    "WS   {}",
                    format_bytes(proc_stats.working_set_bytes)
                ))
                .size(12.0),
            );
            ui.label(
                egui::RichText::new(format!(
                    "PWS  {}",
                    format_bytes(proc_stats.private_working_set_bytes)
                ))
                .size(12.0)
                .color(egui::Color32::from_rgb(150, 180, 255)),
            );
        } else if self.config.watch_process.is_some() {
            ui.label(
                egui::RichText::new("Not running")
                    .size(11.0)
                    .color(egui::Color32::from_gray(130)),
            );
        }
    }
}

fn paint_drag_grip(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let bar_w = 16.0;
    let bar_h = 2.0;
    let gap = 3.0;
    let left = center.x - bar_w / 2.0;
    let top = center.y - (bar_h * 3.0 + gap * 2.0) / 2.0;
    for i in 0..3 {
        let y = top + i as f32 * (bar_h + gap);
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(left, y), egui::vec2(bar_w, bar_h)),
            egui::CornerRadius::same(1),
            color,
        );
    }
}

fn render_bar_row(ui: &mut egui::Ui, label: &str, percent: f32) {
    render_bar_row_inner(ui, label, percent, |pct| format!("{:.0}%", pct));
}

fn render_bar_row_precise(ui: &mut egui::Ui, label: &str, percent: f32) {
    render_bar_row_inner(ui, label, percent, |pct| format!("{:.1}%", pct));
}

fn render_bar_row_inner(
    ui: &mut egui::Ui,
    label: &str,
    percent: f32,
    format_pct: impl Fn(f32) -> String,
) {
    let pct = percent.clamp(0.0, 100.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).size(13.0).color(egui::Color32::from_gray(130)));
        ui.label(egui::RichText::new(format_pct(pct)).size(13.0));
        ui.add_space(8.0);
        ui.add(
            egui::ProgressBar::new(pct / 100.0)
                .desired_width(110.0)
                .desired_height(14.0)
                .fill(egui::Color32::from_rgb(100, 160, 255))
                .corner_radius(egui::CornerRadius::same(4))
                .animate(false),
        );
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
