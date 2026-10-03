use std::time::{Duration, Instant};
use eframe::egui::{self, Align, Color32, Layout, Margin, RichText, Rounding, Stroke, Ui};

use crate::config::GuiConfig;
use crate::theme::Theme;
use crate::ui::components;

/// Renders top navigation bar for the Settings screen.
pub fn show_settings_header(
    ui: &mut Ui,
    theme: &Theme,
    go_back: &mut bool,
) {
    ui.horizontal(|ui| {
        let back_btn = ui.add(
            egui::Button::new(
                RichText::new("< Back to Service")
                    .size(12.0)
                    .strong()
                    .color(theme.text_primary),
            )
            .fill(theme.bg_surface)
            .stroke(Stroke::new(1.0_f32, theme.border))
            .rounding(Rounding::same(6.0)),
        );
        if back_btn.clicked() {
            *go_back = true;
        }

        ui.add_space(8.0);

        ui.label(
            RichText::new("Preferences & Configuration")
                .color(theme.text_primary)
                .strong()
                .size(15.0),
        );
    });

    ui.add_space(10.0);
    // Header separator line removed as per user request
}

/// Renders settings cards: Server & Network, Hardware Acceleration, Storage Directories, Behavior, and Save button.
pub fn show_settings_screen(
    ui: &mut Ui,
    theme: &Theme,
    config: &mut GuiConfig,
    config_saved_feedback: &mut Option<Instant>,
) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            const CONTROL_H: f32 = 32.0;
            const CARD_PAD: f32 = 16.0;

            // Leave space for the scrollbar on the right (14px gutter) so it stays outside the cards
            let card_w = (ui.available_width() - 14.0).max(100.0);
            let inner_w = (card_w - CARD_PAD * 2.0).max(100.0);

            // Group 1: Server & Network
            egui::Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0_f32, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(CARD_PAD))
                .show(ui, |ui| {
                    ui.set_width(inner_w);
                    ui.set_max_width(inner_w);

                    ui.label(
                        RichText::new("Server & Network")
                            .strong()
                            .size(13.0)
                            .color(theme.text_primary),
                    );
                    ui.add_space(14.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("HTTP Port:").color(theme.text_secondary).size(13.0));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let mut port_str = config.port.to_string();
                            let edit = components::singleline_input(&mut port_str);
                            if ui.add_sized([160.0, CONTROL_H], edit).changed() {
                                if let Ok(p) = port_str.parse::<u16>() {
                                    config.port = p;
                                }
                            }
                        });
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Host Address:").color(theme.text_secondary).size(13.0));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let edit = components::singleline_input(&mut config.host);
                            ui.add_sized([160.0, CONTROL_H], edit);
                        });
                    });
                });

            ui.add_space(16.0);

            // Group 2: Hardware Acceleration
            egui::Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0_f32, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(CARD_PAD))
                .show(ui, |ui| {
                    ui.set_width(inner_w);
                    ui.set_max_width(inner_w);

                    ui.label(
                        RichText::new("Hardware Acceleration")
                            .strong()
                            .size(13.0)
                            .color(theme.text_primary),
                    );
                    ui.add_space(14.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Backend:").color(theme.text_secondary).size(13.0));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.spacing_mut().interact_size.y = CONTROL_H;
                            egui::ComboBox::from_id_salt("hw_combo")
                                .selected_text(&config.hardware)
                                .width(160.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut config.hardware, "auto".to_string(), "Auto (Recommended)");
                                    ui.selectable_value(&mut config.hardware, "cpu".to_string(), "CPU (Portable)");
                                    ui.selectable_value(&mut config.hardware, "cuda".to_string(), "CUDA (NVIDIA GPU)");
                                    ui.selectable_value(&mut config.hardware, "vulkan".to_string(), "Vulkan");
                                    ui.selectable_value(&mut config.hardware, "rocm".to_string(), "ROCm (AMD)");
                                });
                        });
                    });
                });

            ui.add_space(16.0);

            // Group 3: Storage Directories
            egui::Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0_f32, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(CARD_PAD))
                .show(ui, |ui| {
                    ui.set_width(inner_w);
                    ui.set_max_width(inner_w);

                    ui.label(
                        RichText::new("Storage Directories")
                            .strong()
                            .size(13.0)
                            .color(theme.text_primary),
                    );
                    ui.add_space(14.0);

                    let browse_w = 80.0;
                    let item_spacing = ui.spacing().item_spacing.x;
                    let input_w = (inner_w - browse_w - item_spacing).max(60.0);

                    ui.label(RichText::new("Models Directory:").color(theme.text_secondary).size(13.0));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let edit = components::singleline_input(&mut config.models_dir).desired_width(input_w);
                        ui.add_sized([input_w, CONTROL_H], edit);
                        let browse_btn = ui.add_sized([browse_w, CONTROL_H], egui::Button::new("Browse..."));
                        if browse_btn.clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                config.models_dir = folder.to_string_lossy().to_string();
                            }
                        }
                    });

                    ui.add_space(14.0);

                    ui.label(RichText::new("Runtimes Directory:").color(theme.text_secondary).size(13.0));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let edit = components::singleline_input(&mut config.runtimes_dir).desired_width(input_w);
                        ui.add_sized([input_w, CONTROL_H], edit);
                        let browse_btn = ui.add_sized([browse_w, CONTROL_H], egui::Button::new("Browse..."));
                        if browse_btn.clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                config.runtimes_dir = folder.to_string_lossy().to_string();
                            }
                        }
                    });
                });

            ui.add_space(16.0);

            // Group 4: Application Behavior
            egui::Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0_f32, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(CARD_PAD))
                .show(ui, |ui| {
                    ui.set_width(inner_w);
                    ui.set_max_width(inner_w);

                    ui.label(
                        RichText::new("Application Behavior")
                            .strong()
                            .size(13.0)
                            .color(theme.text_primary),
                    );
                    ui.add_space(14.0);

                    ui.checkbox(
                        &mut config.autostart_app,
                        "Start Monolai Desktop on system startup",
                    );
                    ui.add_space(8.0);
                    ui.checkbox(
                        &mut config.autostart_server,
                        "Start service automatically on launch",
                    );
                    ui.add_space(8.0);
                    ui.checkbox(
                        &mut config.minimize_on_start,
                        "Minimize to system tray when service starts",
                    );
                });

            ui.add_space(18.0);

            // Save Preferences Button
            let saved = config_saved_feedback
                .map(|t| t.elapsed() < Duration::from_secs(2))
                .unwrap_or(false);

            let save_text = if saved { "Settings Saved Successfully!" } else { "Save Preferences" };
            let save_color = if saved { theme.emerald } else { theme.primary };

            if ui
                .add_sized(
                    [card_w, 42.0],
                    egui::Button::new(RichText::new(save_text).strong().size(13.0).color(Color32::WHITE))
                        .fill(save_color)
                        .rounding(Rounding::same(6.0)),
                )
                .clicked()
            {
                let _ = crate::config::set_autostart_app(config.autostart_app);
                let _ = config.save();

                // If backend is running, notify it immediately via HTTP API so it reloads in real-time
                let payload = serde_json::json!({
                    "models": config.models_dir.trim(),
                    "runtimes": config.runtimes_dir.trim(),
                    "hardware": config.hardware.trim(),
                });
                if let Ok(body) = serde_json::to_string(&payload) {
                    let _ = ureq::post(&format!("http://127.0.0.1:{}/api/config/setup", config.port))
                        .set("Content-Type", "application/json")
                        .timeout(Duration::from_millis(500))
                        .send_string(&body);
                }

                *config_saved_feedback = Some(Instant::now());
            }

            ui.add_space(16.0);
        });
}
