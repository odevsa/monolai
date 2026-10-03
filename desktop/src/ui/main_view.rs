use crate::config::GuiConfig;
use crate::supervisor::{ProcessSupervisor, ServerStatus};
use crate::theme::Theme;
use crate::ui::components;
use eframe::egui::{self, Align, Color32, Layout, Margin, RichText, Rounding, Stroke, Ui, Vec2};
use std::time::{Duration, Instant};

pub fn show_main_header(
    ui: &mut Ui,
    _ctx: &egui::Context,
    theme: &Theme,
    status: &ServerStatus,
    on_settings: impl FnOnce(),
) {
    ui.horizontal(|ui| {
        // App title and small icon on header
        ui.label(
            RichText::new("Monolai")
                .color(theme.text_primary)
                .strong()
                .size(16.0),
        );

        // Header controls (right-aligned)
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Settings Navigation Button
            let settings_btn = ui.add(
                egui::Button::new(
                    RichText::new("Settings")
                        .size(11.0)
                        .strong()
                        .color(theme.text_primary),
                )
                .fill(theme.bg_surface)
                .stroke(Stroke::new(1.0_f32, theme.border))
                .rounding(Rounding::same(6.0)),
            );
            if settings_btn.on_hover_text("Configure server and directories").clicked() {
                on_settings();
            }

            ui.add_space(8.0);

            // Status Badge with vector dot
            let (badge_bg, badge_border, dot_color, text_color, text) = match status {
                ServerStatus::Running { .. } => (
                    theme.emerald_bg,
                    theme.emerald,
                    theme.emerald,
                    theme.emerald,
                    "ONLINE",
                ),
                ServerStatus::Starting => (
                    theme.amber_bg,
                    theme.amber,
                    theme.amber,
                    theme.amber,
                    "STARTING",
                ),
                ServerStatus::Stopping => (
                    theme.amber_bg,
                    theme.amber,
                    theme.amber,
                    theme.amber,
                    "STOPPING",
                ),
                ServerStatus::Error(_) | ServerStatus::UnexpectedExit { .. } => (
                    theme.red_bg,
                    theme.red,
                    theme.red,
                    theme.red,
                    "ERROR",
                ),
                ServerStatus::Stopped => (
                    theme.bg_surface,
                    theme.border,
                    theme.text_muted,
                    theme.text_muted,
                    "OFFLINE",
                ),
            };

            egui::Frame::none()
                .fill(badge_bg)
                .stroke(Stroke::new(1.0_f32, badge_border))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(8.0, 3.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        components::draw_status_dot(ui, dot_color, 3.5);
                        ui.add_space(3.0);
                        ui.label(
                            RichText::new(text)
                                .color(text_color)
                                .size(10.0)
                                .strong(),
                        );
                    });
                });
        });
    });

    // Clean spacing without separator line
    ui.add_space(10.0);
}

pub fn show_main_screen(
    ui: &mut Ui,
    ctx: &egui::Context,
    theme: &Theme,
    config: &GuiConfig,
    supervisor: &ProcessSupervisor,
    logo_dark: Option<&egui::TextureHandle>,
    logo_light: Option<&egui::TextureHandle>,
    copied_feedback: &mut Option<Instant>,
    alert_message: &Option<(String, Instant)>,
    on_start_error: impl FnOnce(String),
) {
    let status = supervisor.get_status();

    // Prominent Centered Hero Section with Larger Logo
    ui.vertical_centered(|ui| {
        let logo = if theme.is_dark { logo_dark } else { logo_light };
        if let Some(logo_tex) = logo {
            ui.add(egui::Image::new(logo_tex).fit_to_exact_size(Vec2::new(64.0, 64.0)));
        }

        ui.add_space(8.0);

        ui.label(
            RichText::new("Monolai")
                .color(theme.text_primary)
                .strong()
                .size(22.0),
        );

        ui.add_space(2.0);

        ui.label(
            RichText::new("Local AI Runtime & Model Hub")
                .color(theme.text_secondary)
                .size(12.0),
        );
    });

    ui.add_space(14.0);

    // 1. Hero Status Card
    egui::Frame::none()
        .fill(theme.bg_surface)
        .stroke(Stroke::new(1.0_f32, theme.border))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(16.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            match &status {
                ServerStatus::Running { url, port, started_at } => {
                    let uptime = started_at.elapsed().as_secs();
                    let mins = uptime / 60;
                    let secs = uptime % 60;

                    ui.horizontal(|ui| {
                        components::draw_status_dot(ui, theme.emerald, 5.0);
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("Service is Active & Ready")
                                    .color(theme.text_primary)
                                    .strong()
                                    .size(15.0),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "Listening on port {} | Uptime: {:02}m {:02}s",
                                    port, mins, secs
                                ))
                                .color(theme.text_secondary)
                                .size(12.0),
                            );
                        });
                    });

                    ui.add_space(16.0);

                    // URL Display Box
                    egui::Frame::none()
                        .fill(theme.bg_input)
                        .stroke(Stroke::new(1.0_f32, theme.border))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(url)
                                        .monospace()
                                        .color(theme.text_primary)
                                        .size(13.0),
                                );

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let copied = copied_feedback
                                        .map(|t| t.elapsed() < Duration::from_secs(2))
                                        .unwrap_or(false);

                                    let copy_btn_text = if copied { "Copied!" } else { "Copy" };
                                    if ui.add_sized([65.0, 26.0], egui::Button::new(RichText::new(copy_btn_text).size(11.0))).clicked() {
                                        ctx.output_mut(|o| o.copied_text = url.clone());
                                        *copied_feedback = Some(Instant::now());
                                    }
                                });
                            });
                        });

                    ui.add_space(12.0);

                    // Open Web Interface Button
                    let open_btn = ui.add_sized(
                        [ui.available_width(), 38.0],
                        egui::Button::new(
                            RichText::new("Open Web Interface")
                                .color(Color32::WHITE)
                                .strong()
                                .size(13.0),
                        )
                        .fill(theme.primary)
                        .rounding(Rounding::same(6.0)),
                    );

                    if open_btn.clicked() {
                        let _ = open::that(url);
                    }
                }
                ServerStatus::Starting => {
                    ui.horizontal(|ui| {
                        components::draw_status_dot(ui, theme.amber, 5.0);
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("Starting Service...")
                                    .color(theme.amber)
                                    .strong()
                                    .size(15.0),
                            );
                            ui.label(
                                RichText::new("Waiting for HTTP server to respond...")
                                    .color(theme.text_secondary)
                                    .size(12.0),
                            );
                        });
                    });
                }
                ServerStatus::Stopping => {
                    ui.horizontal(|ui| {
                        components::draw_status_dot(ui, theme.amber, 5.0);
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("Stopping Service...")
                                    .color(theme.amber)
                                    .strong()
                                    .size(15.0),
                            );
                            ui.label(
                                RichText::new("Terminating background runtime processes...")
                                    .color(theme.text_secondary)
                                    .size(12.0),
                            );
                        });
                    });
                }
                ServerStatus::Stopped => {
                    ui.horizontal(|ui| {
                        components::draw_status_dot(ui, theme.text_muted, 5.0);
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("Service is Inactive")
                                    .color(theme.text_primary)
                                    .strong()
                                    .size(15.0),
                            );
                            ui.label(
                                RichText::new("Click Start Service below to launch the local AI backend.")
                                    .color(theme.text_secondary)
                                    .size(12.0),
                            );
                        });
                    });
                }
                ServerStatus::UnexpectedExit { exit_code } => {
                    ui.horizontal(|ui| {
                        components::draw_status_dot(ui, theme.red, 5.0);
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("Service Stopped Unexpectedly")
                                    .color(theme.red)
                                    .strong()
                                    .size(15.0),
                            );
                            let code_str = exit_code.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string());
                            ui.label(
                                RichText::new(format!("Exit status code: {}", code_str))
                                    .color(theme.text_secondary)
                                    .size(12.0),
                            );
                        });
                    });
                }
                ServerStatus::Error(err) => {
                    ui.horizontal(|ui| {
                        components::draw_status_dot(ui, theme.red, 5.0);
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("Failed to Launch Service")
                                    .color(theme.red)
                                    .strong()
                                    .size(15.0),
                            );
                            ui.label(
                                RichText::new(err)
                                    .color(theme.text_secondary)
                                    .size(12.0),
                            );
                        });
                    });
                }
            }
        });

    ui.add_space(14.0);

    // 2. Action Control Buttons
    match status {
        ServerStatus::Running { .. } => {
            ui.horizontal(|ui| {
                let total_w = ui.available_width();
                let spacing = ui.spacing().item_spacing.x;
                let btn_w = (total_w - spacing) * 0.5;

                let stop_btn = ui.add_sized(
                    [btn_w, 42.0],
                    egui::Button::new(
                        RichText::new("Stop Service")
                            .color(Color32::WHITE)
                            .strong()
                            .size(13.0),
                    )
                    .fill(theme.red)
                    .rounding(Rounding::same(6.0)),
                );
                if stop_btn.clicked() {
                    let _ = supervisor.stop();
                }

                let restart_btn = ui.add_sized(
                    [btn_w, 42.0],
                    egui::Button::new(
                        RichText::new("Restart")
                            .color(theme.text_primary)
                            .strong()
                            .size(13.0),
                    )
                    .fill(theme.bg_surface)
                    .stroke(Stroke::new(1.0_f32, theme.border))
                    .rounding(Rounding::same(6.0)),
                );
                if restart_btn.clicked() {
                    let _ = supervisor.restart(config);
                }
            });
        }
        ServerStatus::Stopped | ServerStatus::UnexpectedExit { .. } | ServerStatus::Error(_) => {
            let start_btn = ui.add_sized(
                [ui.available_width(), 44.0],
                egui::Button::new(
                    RichText::new("Start Service")
                        .color(Color32::WHITE)
                        .strong()
                        .size(14.0),
                )
                .fill(theme.emerald)
                .rounding(Rounding::same(6.0)),
            );
            if start_btn.clicked() {
                if let Err(e) = supervisor.start(config) {
                    on_start_error(e);
                }
            }
        }
        ServerStatus::Starting | ServerStatus::Stopping => {
            ui.add_enabled(
                false,
                egui::Button::new(RichText::new("Processing...").size(14.0))
                    .min_size(Vec2::new(ui.available_width(), 44.0))
                    .rounding(Rounding::same(6.0)),
            );
        }
    }

    // 3. Error Banner Alert Message if present
    if let Some((msg, created_at)) = alert_message {
        if created_at.elapsed() < Duration::from_secs(6) {
            ui.add_space(12.0);
            egui::Frame::none()
                .fill(theme.red_bg)
                .stroke(Stroke::new(1.0_f32, theme.red))
                .rounding(Rounding::same(6.0))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(msg).color(theme.red).size(12.0));
                });
        }
    }

    ui.add_space(16.0);

}
