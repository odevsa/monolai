use std::time::Instant;
use eframe::egui::{self, Align, Color32, Layout, Margin, RichText, Rounding, Stroke, Ui};

use crate::core::config::GuiConfig;
use crate::platform::autostart;
use crate::supervisor::SupervisorHandle;
use crate::ui::app::{FeedbackState, MaintenanceUiState};
use crate::ui::components;
use crate::ui::theme::Theme;

pub fn show_settings_header(ui: &mut Ui, theme: &Theme, on_back: impl FnOnce()) {
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
            on_back();
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
}

pub fn show_settings_screen(
    ui: &mut Ui,
    theme: &Theme,
    config: &mut GuiConfig,
    supervisor: &SupervisorHandle,
    feedback: &mut FeedbackState,
    maintenance: &mut MaintenanceUiState,
) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            const CONTROL_H: f32 = 32.0;
            const CARD_PAD: f32 = 16.0;

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
                        "Start on system startup",
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
            let saved = feedback.is_saved_active();
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
                let _ = autostart::set_autostart_app(config.autostart_app, config.minimize_on_start);
                let _ = config.save();
                supervisor.notify_config(config.clone());
                feedback.saved = Some(Instant::now());
            }

            ui.add_space(20.0);

            // Group 5: Data Management & Maintenance
            egui::Frame::none()
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0_f32, theme.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(CARD_PAD))
                .show(ui, |ui| {
                    ui.set_width(inner_w);
                    ui.set_max_width(inner_w);

                    ui.label(
                        RichText::new("Data Management & Maintenance")
                            .strong()
                            .size(13.0)
                            .color(theme.text_primary),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Manage database storage, model tracking, and application configuration.")
                            .size(11.0)
                            .color(theme.text_muted),
                    );
                    ui.add_space(14.0);

                    // Row 1: Clean Storage
                    ui.label(RichText::new("Clean Storage").strong().size(12.0).color(theme.text_primary));
                    ui.add_space(3.0);
                    ui.label(
                        RichText::new("Deletes the SQLite database (chat histories, messages, and model cache) and active process cache. Preserves your settings and downloaded models.")
                            .size(11.0)
                            .color(theme.text_secondary),
                    );
                    ui.add_space(8.0);

                    let in_progress = maintenance.in_progress;
                    let clean_confirming = maintenance.is_clean_confirming();

                    let (clean_btn_text, clean_btn_fill, clean_btn_color) = if in_progress {
                        ("Maintenance in progress...", theme.bg_primary, theme.text_muted)
                    } else if clean_confirming {
                        ("Click to Confirm: Wipe Storage Database", Color32::from_rgb(220, 38, 38), Color32::WHITE)
                    } else {
                        ("Clean Storage", theme.bg_primary, theme.text_primary)
                    };

                    let clean_btn = ui.add_enabled(
                        !in_progress,
                        egui::Button::new(RichText::new(clean_btn_text).strong().size(12.0).color(clean_btn_color))
                            .fill(clean_btn_fill)
                            .stroke(Stroke::new(1.0_f32, if clean_confirming { Color32::from_rgb(220, 38, 38) } else { theme.border }))
                            .rounding(Rounding::same(6.0))
                            .min_size(egui::Vec2::new(inner_w, CONTROL_H)),
                    );

                    if clean_btn.clicked() {
                        if clean_confirming {
                            maintenance.confirm_clean = None;
                            maintenance.in_progress = true;
                            supervisor.run_maintenance("--clean-storage", Some(config.clone()));
                        } else {
                            maintenance.confirm_clean = Some(Instant::now());
                            maintenance.confirm_reset = None;
                        }
                    }

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(14.0);

                    // Row 2: Factory Reset
                    ui.label(RichText::new("Factory Reset").strong().size(12.0).color(theme.text_primary));
                    ui.add_space(3.0);
                    ui.label(
                        RichText::new("Performs a complete factory reset. Deletes the database, process cache, and resets your configuration file (config.yaml) to defaults.")
                            .size(11.0)
                            .color(theme.text_secondary),
                    );
                    ui.add_space(8.0);

                    let reset_confirming = maintenance.is_reset_confirming();

                    let (reset_btn_text, reset_btn_fill, reset_btn_color) = if in_progress {
                        ("Maintenance in progress...", theme.bg_primary, theme.text_muted)
                    } else if reset_confirming {
                        ("Click to Confirm: Factory Reset Everything", Color32::from_rgb(185, 28, 28), Color32::WHITE)
                    } else {
                        ("Factory Reset", theme.bg_primary, theme.red)
                    };

                    let reset_btn = ui.add_enabled(
                        !in_progress,
                        egui::Button::new(RichText::new(reset_btn_text).strong().size(12.0).color(reset_btn_color))
                            .fill(reset_btn_fill)
                            .stroke(Stroke::new(1.0_f32, if reset_confirming { Color32::from_rgb(185, 28, 28) } else { theme.red }))
                            .rounding(Rounding::same(6.0))
                            .min_size(egui::Vec2::new(inner_w, CONTROL_H)),
                    );

                    if reset_btn.clicked() {
                        if reset_confirming {
                            maintenance.confirm_reset = None;
                            maintenance.in_progress = true;
                            supervisor.run_maintenance("--factory-reset", Some(config.clone()));
                        } else {
                            maintenance.confirm_reset = Some(Instant::now());
                            maintenance.confirm_clean = None;
                        }
                    }

                    if let Some((msg, is_ok)) = maintenance.current_status() {
                        ui.add_space(12.0);
                        let banner_color = if is_ok { theme.emerald } else { theme.red };
                        ui.label(
                            RichText::new(msg)
                                .color(banner_color)
                                .strong()
                                .size(12.0),
                        );
                    }
                });

            ui.add_space(16.0);
        });
}
