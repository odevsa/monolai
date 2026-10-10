use eframe::egui::{self, Color32, Ui, Vec2};

/// Draws a clean circular status dot (pure GPU vector paint).
pub fn draw_status_dot(ui: &mut Ui, color: Color32, radius: f32) {
    let size = Vec2::splat((radius + 1.0) * 2.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), radius, color);
}
