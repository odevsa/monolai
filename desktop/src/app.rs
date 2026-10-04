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
    #[cfg(target_os = "linux")]
    tray_dismiss_at: Option<Instant>,
    #[cfg(target_os = "linux")]
    tray_restore_at: Option<Instant>,
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
            let _ = crate::config::set_autostart_app(true);
            config.autostart_app = true;
            let _ = config.save();
        } else if crate::config::is_autostart_app_enabled() {
            config.autostart_app = true;
        }
        let supervisor = ProcessSupervisor::new();
        let tray = match TrayManager::new(cc.egui_ctx.clone()) {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("Tray manager initialization error: {}", e);
                None
            }
        };
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

        let mut app = Self {
            config,
            supervisor,
            tray,
            #[cfg(target_os = "linux")]
            tray_dismiss_at: None,
            #[cfg(target_os = "linux")]
            tray_restore_at: None,
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

        if std::env::args().any(|a| a == "--minimized") {
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

            // Auto minimize when successfully transitioned to Running if configured
            if let ServerStatus::Running { .. } = current_status {
                if self.config.minimize_on_start {
                    components::minimize_window(ctx);
                }
            }

            // Auto restore window if service exited unexpectedly
            if let ServerStatus::UnexpectedExit { exit_code } = current_status {
                components::restore_window(ctx);
                let code_msg = exit_code.map(|c| format!(" (exit code: {})", c)).unwrap_or_default();
                self.alert_message = Some((
                    format!("The Monolai service stopped unexpectedly{}!", code_msg),
                    Instant::now(),
                ));
            }

            self.previous_status = current_status.clone();
        }

        // Handle Tray Actions
        let mut tray_actions = Vec::new();
        if let Some(ref tray) = self.tray {
            while let Some(action) = tray.poll_action() {
                tray_actions.push(action);
            }
        }

        #[cfg(target_os = "linux")]
        let mut had_tray_action = false;
        for action in tray_actions {
            match action {
                crate::tray::TrayAction::Menu(menu_id) => match menu_id.as_str() {
                    "open_web" => {
                        let url = format!("http://localhost:{}", self.config.port);
                        let _ = open::that(url);
                        #[cfg(target_os = "linux")]
                        { had_tray_action = true; }
                    }
                    "show_panel" => {
                        components::restore_window(ctx);
                        #[cfg(target_os = "linux")]
                        { had_tray_action = true; }
                    }
                    "start_server" => {
                        if let Err(e) = self.supervisor.start(&self.config) {
                            self.alert_message = Some((e, Instant::now()));
                        }
                        #[cfg(target_os = "linux")]
                        { had_tray_action = true; }
                    }
                    "stop_server" => {
                        let _ = self.supervisor.stop();
                        #[cfg(target_os = "linux")]
                        { had_tray_action = true; }
                    }
                    "restart_server" => {
                        let _ = self.supervisor.restart(&self.config);
                        #[cfg(target_os = "linux")]
                        { had_tray_action = true; }
                    }
                    "quit" => {
                        let _ = self.supervisor.stop();
                        self.tray = None;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        #[cfg(target_os = "windows")]
                        std::process::exit(0);
                    }
                    _ => {}
                },
                crate::tray::TrayAction::IconDoubleClick | crate::tray::TrayAction::IconClick => {
                    components::restore_window(ctx);
                }
            }
        }

        // Schedule tray dismissal with a short 60ms delay so ksni finishes sending its D-Bus reply,
        // preventing the 7-second D-Bus RPC deadlock/timeout.
        #[cfg(target_os = "linux")]
        if had_tray_action {
            self.tray_dismiss_at = Some(Instant::now() + Duration::from_millis(60));
            ctx.request_repaint_after(Duration::from_millis(60));
        }

        // Dismiss tray after in-flight D-Bus RPC completes
        #[cfg(target_os = "linux")]
        if let Some(dismiss_at) = self.tray_dismiss_at {
            if Instant::now() >= dismiss_at {
                self.tray = None;
                self.tray_dismiss_at = None;
                self.tray_restore_at = Some(Instant::now() + Duration::from_millis(120));
                ctx.request_repaint_after(Duration::from_millis(120));
            } else {
                ctx.request_repaint_after(Duration::from_millis(20));
            }
        }

        // Restore tray after dismissal
        #[cfg(target_os = "linux")]
        if let Some(restore_at) = self.tray_restore_at {
            if Instant::now() >= restore_at {
                let is_running = matches!(self.supervisor.get_status(), ServerStatus::Running { .. });
                let port = self.config.port;
                if let Ok(mut tray) = crate::tray::TrayManager::new(ctx.clone()) {
                    tray.update_status(is_running, port);
                    self.tray = Some(tray);
                }
                self.tray_restore_at = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(25));
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
