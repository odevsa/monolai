use std::time::{Duration, Instant};
use eframe::egui::{self, Margin};

use crate::config::GuiConfig;
use crate::supervisor::{ProcessSupervisor, ServerStatus};
use crate::theme::{self, Theme};
use crate::tray::TrayManager;
use crate::ui::{components, main_view, settings_view};

#[derive(PartialEq, Clone, Copy)]
pub enum Screen {
    Main,
    Settings,
}

pub struct DesktopApp {
    config: GuiConfig,
    supervisor: ProcessSupervisor,
    tray: Option<TrayManager>,
    window_visible: bool,
    is_quitting: bool,
    minimize_after: Option<Instant>,
    current_screen: Screen,
    previous_status: ServerStatus,
    alert_message: Option<(String, Instant)>,
    copied_feedback: Option<Instant>,
    config_saved_feedback: Option<Instant>,
    confirm_clean_storage: Option<Instant>,
    confirm_factory_reset: Option<Instant>,
    maintenance_status: Option<(String, bool, Instant)>,
    logo_dark: Option<egui::TextureHandle>,
    logo_light: Option<egui::TextureHandle>,
    theme: Theme,
    last_theme_check: Instant,
}

impl Drop for DesktopApp {
    fn drop(&mut self) {
        // Ensure child process is killed when app exits
        let _ = self.supervisor.stop();
    }
}

impl DesktopApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let mut config = GuiConfig::load();
        if !GuiConfig::config_path().exists() {
            let _ = crate::config::set_autostart_app(true, config.minimize_on_start);
            config.autostart_app = true;
            let _ = config.save();
        } else if crate::config::is_autostart_app_enabled() {
            config.autostart_app = true;
        }
        let supervisor = ProcessSupervisor::new();
        let tray = match TrayManager::new(cc.egui_ctx.clone(), supervisor.clone()) {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("Tray manager initialization error: {}", e);
                None
            }
        };

        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(handle) = cc.window_handle() {
                if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                    components::set_windows_hwnd(win32_handle.hwnd.get() as isize);
                }
            }
        }
        let theme = theme::get_system_theme();

        let logo_dark = components::load_png_texture(
            &cc.egui_ctx,
            "logo_dark",
            include_bytes!("../assets/icons/logo-dark-128.png"),
        );
        let logo_light = components::load_png_texture(
            &cc.egui_ctx,
            "logo_light",
            include_bytes!("../assets/icons/logo-light-128.png"),
        );

        let start_minimized = std::env::args().any(|a| a == "--minimized");
        let window_visible = !start_minimized;

        let mut app = Self {
            config,
            supervisor,
            tray,
            window_visible,
            is_quitting: false,
            minimize_after: None,
            current_screen: Screen::Main,
            previous_status: ServerStatus::Stopped,
            alert_message: None,
            copied_feedback: None,
            config_saved_feedback: None,
            confirm_clean_storage: None,
            confirm_factory_reset: None,
            maintenance_status: None,
            logo_dark,
            logo_light,
            theme,
            last_theme_check: Instant::now(),
        };

        if app.config.autostart_server {
            let _ = app.supervisor.start(&app.config);
        }

        // Initialize tray state with proper active/muted items
        if let Some(ref mut tray) = app.tray {
            let is_running = matches!(app.supervisor.get_status(), ServerStatus::Running { .. });
            tray.update_status(is_running, app.config.port);
        }

        if start_minimized {
            components::minimize_window(&cc.egui_ctx);
        }

        app
    }
}

impl eframe::App for DesktopApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Stop background backend server process when window is closed
        let _ = self.supervisor.stop();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(target_os = "windows")]
        if components::get_windows_hwnd() == 0 {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(handle) = _frame.window_handle() {
                if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                    components::set_windows_hwnd(win32_handle.hwnd.get() as isize);
                }
            }
        }

        ctx.request_repaint_after(Duration::from_millis(500));

        // Periodic OS Theme Auto-detection
        if self.last_theme_check.elapsed() > Duration::from_secs(2) {
            self.last_theme_check = Instant::now();
            let current_os_theme = theme::get_system_theme();
            if current_os_theme.is_dark != self.theme.is_dark {
                self.theme = current_os_theme;
                theme::apply_theme(ctx, &self.theme);
            }
        }

        let current_status = self.supervisor.get_status();

        // Check state transitions
        if current_status != self.previous_status {
            // Update Tray
            if let Some(ref mut tray) = self.tray {
                let is_running = matches!(current_status, ServerStatus::Running { .. });
                let port = match &current_status {
                    ServerStatus::Running { port, .. } => *port,
                    _ => self.config.port,
                };
                tray.update_status(is_running, port);
            }

            // Auto minimize when successfully transitioned to Running if configured (with 1s delay for visual feedback)
            if let ServerStatus::Running { .. } = current_status {
                if self.config.minimize_on_start {
                    self.minimize_after = Some(Instant::now() + Duration::from_secs(1));
                    ctx.request_repaint_after(Duration::from_secs(1));
                }
            } else {
                self.minimize_after = None;
            }

            // Auto restore window if service exited unexpectedly
            if let ServerStatus::UnexpectedExit { exit_code } = current_status {
                self.minimize_after = None;
                self.window_visible = true;
                components::restore_window(ctx);
                let code_msg = exit_code.map(|c| format!(" (exit code: {})", c)).unwrap_or_default();
                self.alert_message = Some((
                    format!("The Monolai service stopped unexpectedly{}!", code_msg),
                    Instant::now(),
                ));
            }

            self.previous_status = current_status.clone();
        }

        // Trigger delayed minimize to tray after 1-second visual feedback
        if let Some(target) = self.minimize_after {
            if Instant::now() >= target {
                self.minimize_after = None;
                self.window_visible = false;
                components::minimize_window(ctx);
            } else {
                ctx.request_repaint_after(target.saturating_duration_since(Instant::now()));
            }
        }

        // Intercept window close button ("X") to minimize to system tray instead of closing
        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.is_quitting {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.window_visible = false;
                components::minimize_window(ctx);
            }
        }

        // Handle Tray Actions
        let mut tray_actions = Vec::new();
        if let Some(ref tray) = self.tray {
            while let Some(action) = tray.poll_action() {
                tray_actions.push(action);
            }
        }

        for action in tray_actions {
            match action {
                crate::tray::TrayAction::OpenDashboard => {
                    self.minimize_after = None;
                    self.window_visible = true;
                    components::restore_window(ctx);
                }
                crate::tray::TrayAction::Quit => {
                    self.minimize_after = None;
                    self.is_quitting = true;
                    let _ = self.supervisor.stop();
                    self.tray = None;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    std::process::exit(0);
                }
            }
        }

        // Dynamic margins based on active screen:
        // Main screen has symmetric 24px padding.
        // Settings screen has 6px right margin so the scrollbar aligns neatly to the window edge,
        // with the cards padded inside the scroll area.
        let panel_margin = match self.current_screen {
            Screen::Main => Margin::symmetric(24.0, 16.0),
            Screen::Settings => Margin {
                left: 20.0,
                right: 6.0,
                top: 16.0,
                bottom: 12.0,
            },
        };

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(self.theme.bg_primary)
                    .inner_margin(panel_margin),
            )
            .show(ctx, |ui| {
                match self.current_screen {
                    Screen::Main => {
                        let mut go_to_settings = false;
                        main_view::show_main_header(
                            ui,
                            ctx,
                            &self.theme,
                            &self.supervisor.get_status(),
                            || go_to_settings = true,
                        );
                        if go_to_settings {
                            self.current_screen = Screen::Settings;
                        }

                        let mut start_err = None;
                        main_view::show_main_screen(
                            ui,
                            ctx,
                            &self.theme,
                            &self.config,
                            &self.supervisor,
                            self.logo_dark.as_ref(),
                            self.logo_light.as_ref(),
                            &mut self.copied_feedback,
                            &self.alert_message,
                            |e| start_err = Some(e),
                        );
                        if let Some(err) = start_err {
                            self.alert_message = Some((err, Instant::now()));
                        }
                    }
                    Screen::Settings => {
                        let mut go_back = false;
                        settings_view::show_settings_header(ui, &self.theme, &mut go_back);
                        if go_back {
                            self.current_screen = Screen::Main;
                        }

                        settings_view::show_settings_screen(
                            ui,
                            &self.theme,
                            &mut self.config,
                            &self.supervisor,
                            &mut self.config_saved_feedback,
                            &mut self.confirm_clean_storage,
                            &mut self.confirm_factory_reset,
                            &mut self.maintenance_status,
                        );
                    }
                }
            });
    }
}
