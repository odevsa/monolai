use std::time::{Duration, Instant};
use eframe::egui::{self, Margin};

use crate::core::config::GuiConfig;
use crate::core::events::{SupervisorEvent, TrayAction};
use crate::core::status::ServerStatus;
use crate::platform::{autostart, window};
use crate::supervisor::SupervisorHandle;
use crate::tray::TrayManager;
use crate::ui::components;
use crate::ui::theme::{self, Theme};
use crate::ui::views::{main_view, settings_view};

#[derive(PartialEq, Clone, Copy)]
pub enum Screen {
    Main,
    Settings,
}

#[derive(Default)]
pub struct FeedbackState {
    pub copied: Option<Instant>,
    pub saved: Option<Instant>,
    pub alert: Option<(String, Instant)>,
}

impl FeedbackState {
    pub fn is_copied_active(&self) -> bool {
        self.copied
            .map(|t| t.elapsed() < Duration::from_secs(2))
            .unwrap_or(false)
    }

    pub fn is_saved_active(&self) -> bool {
        self.saved
            .map(|t| t.elapsed() < Duration::from_secs(2))
            .unwrap_or(false)
    }

    pub fn current_alert(&self) -> Option<&str> {
        self.alert.as_ref().and_then(|(msg, time)| {
            if time.elapsed() < Duration::from_secs(6) {
                Some(msg.as_str())
            } else {
                None
            }
        })
    }
}

#[derive(Default)]
pub struct MaintenanceUiState {
    pub confirm_clean: Option<Instant>,
    pub confirm_reset: Option<Instant>,
    pub in_progress: bool,
    pub status_message: Option<(String, bool, Instant)>,
}

impl MaintenanceUiState {
    pub fn is_clean_confirming(&self) -> bool {
        self.confirm_clean
            .map(|t| t.elapsed() < Duration::from_secs(5))
            .unwrap_or(false)
    }

    pub fn is_reset_confirming(&self) -> bool {
        self.confirm_reset
            .map(|t| t.elapsed() < Duration::from_secs(5))
            .unwrap_or(false)
    }

    pub fn current_status(&self) -> Option<(&str, bool)> {
        self.status_message.as_ref().and_then(|(msg, ok, time)| {
            if time.elapsed() < Duration::from_secs(6) {
                Some((msg.as_str(), *ok))
            } else {
                None
            }
        })
    }
}

pub struct DesktopApp {
    config: GuiConfig,
    supervisor: SupervisorHandle,
    tray: Option<TrayManager>,
    status: ServerStatus,
    window_visible: bool,
    is_quitting: bool,
    minimize_after: Option<Instant>,
    current_screen: Screen,
    feedback: FeedbackState,
    maintenance: MaintenanceUiState,
    logo_dark: Option<egui::TextureHandle>,
    logo_light: Option<egui::TextureHandle>,
    theme: Theme,
    last_theme_check: Instant,
}

impl DesktopApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let mut config = GuiConfig::load();
        if !GuiConfig::config_path().exists() {
            let _ = autostart::set_autostart_app(true, config.minimize_on_start);
            config.autostart_app = true;
            let _ = config.save();
        } else if autostart::is_autostart_app_enabled() {
            config.autostart_app = true;
        }

        let supervisor = SupervisorHandle::new(cc.egui_ctx.clone());
        let tray = match TrayManager::new(cc.egui_ctx.clone()) {
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
                    window::set_windows_hwnd(win32_handle.hwnd.get() as isize);
                }
            }
        }

        let theme = theme::get_system_theme();

        let logo_dark = components::load_png_texture(
            &cc.egui_ctx,
            "logo_dark",
            include_bytes!("../../assets/icons/logo-dark-128.png"),
        );
        let logo_light = components::load_png_texture(
            &cc.egui_ctx,
            "logo_light",
            include_bytes!("../../assets/icons/logo-light-128.png"),
        );

        let start_minimized = std::env::args().any(|a| a == "--minimized");
        let window_visible = !start_minimized;

        let app = Self {
            config: config.clone(),
            supervisor,
            tray,
            status: ServerStatus::Stopped,
            window_visible,
            is_quitting: false,
            minimize_after: None,
            current_screen: Screen::Main,
            feedback: FeedbackState::default(),
            maintenance: MaintenanceUiState::default(),
            logo_dark,
            logo_light,
            theme,
            last_theme_check: Instant::now(),
        };

        if app.config.autostart_server {
            app.supervisor.start(config);
        }

        if start_minimized {
            window::minimize_window(&cc.egui_ctx);
        }

        app
    }

    fn handle_supervisor_events(&mut self, ctx: &egui::Context) {
        while let Some(event) = self.supervisor.poll_event() {
            match event {
                SupervisorEvent::StatusChanged(new_status) => {
                    let was_running = matches!(self.status, ServerStatus::Running { .. });
                    let is_running = matches!(new_status, ServerStatus::Running { .. });

                    if let Some(ref mut tray) = self.tray {
                        let port = match &new_status {
                            ServerStatus::Running { port, .. } => *port,
                            _ => self.config.port,
                        };
                        tray.update_status(is_running, port);
                    }

                    // Auto-minimize when transitioning to Running if configured
                    if is_running && !was_running && self.config.minimize_on_start {
                        self.minimize_after = Some(Instant::now() + Duration::from_secs(1));
                        ctx.request_repaint_after(Duration::from_secs(1));
                    }

                    // Auto-restore window on unexpected exit
                    if let ServerStatus::UnexpectedExit { exit_code } = &new_status {
                        self.minimize_after = None;
                        self.window_visible = true;
                        window::restore_window(ctx);
                        let code_msg = exit_code
                            .map(|c| format!(" (exit code: {})", c))
                            .unwrap_or_default();
                        self.feedback.alert = Some((
                            format!("The Monolai service stopped unexpectedly{}!", code_msg),
                            Instant::now(),
                        ));
                    }

                    self.status = new_status;
                }
                SupervisorEvent::AlertMessage(msg) => {
                    self.feedback.alert = Some((msg, Instant::now()));
                }
                SupervisorEvent::MaintenanceResult { flag, result } => {
                    self.maintenance.in_progress = false;
                    match result {
                        Ok(msg) => {
                            if flag == "--factory-reset" {
                                self.config = GuiConfig::default();
                                let _ = self.config.save();
                            }
                            self.maintenance.status_message =
                                Some((msg, true, Instant::now()));
                        }
                        Err(err) => {
                            self.maintenance.status_message =
                                Some((err, false, Instant::now()));
                        }
                    }
                }
                SupervisorEvent::ConfigNotificationResult(Err(err)) => {
                    eprintln!("Failed to notify backend of config change: {}", err);
                }
                SupervisorEvent::ConfigNotificationResult(Ok(())) => {}
            }
        }
    }

    fn handle_tray_actions(&mut self, ctx: &egui::Context) {
        let mut actions = Vec::new();
        if let Some(ref tray) = self.tray {
            while let Some(action) = tray.poll_action() {
                actions.push(action);
            }
        }

        for action in actions {
            match action {
                TrayAction::OpenDashboard => {
                    self.minimize_after = None;
                    self.window_visible = true;
                    window::restore_window(ctx);
                }
                TrayAction::Quit => {
                    self.minimize_after = None;
                    self.is_quitting = true;
                    self.supervisor.shutdown();
                    self.tray = None;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    std::process::exit(0);
                }
            }
        }
    }
}

impl Drop for DesktopApp {
    fn drop(&mut self) {
        self.supervisor.shutdown();
    }
}

impl eframe::App for DesktopApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.supervisor.shutdown();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(target_os = "windows")]
        if window::get_windows_hwnd() == 0 {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(handle) = _frame.window_handle() {
                if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                    window::set_windows_hwnd(win32_handle.hwnd.get() as isize);
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

        // Process asynchronous events from supervisor and tray
        self.handle_supervisor_events(ctx);
        self.handle_tray_actions(ctx);

        // Delayed minimize to tray after start visual feedback
        if let Some(target) = self.minimize_after {
            if Instant::now() >= target {
                self.minimize_after = None;
                self.window_visible = false;
                window::minimize_window(ctx);
            } else {
                ctx.request_repaint_after(target.saturating_duration_since(Instant::now()));
            }
        }

        // Intercept close button ("X") to minimize to tray instead of quitting
        if ctx.input(|i| i.viewport().close_requested()) && !self.is_quitting {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.window_visible = false;
            window::minimize_window(ctx);
        }

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
            .show(ctx, |ui| match self.current_screen {
                Screen::Main => {
                    let mut go_to_settings = false;
                    main_view::show_main_header(
                        ui,
                        &self.theme,
                        &self.status,
                        || go_to_settings = true,
                    );
                    if go_to_settings {
                        self.current_screen = Screen::Settings;
                    }

                    main_view::show_main_screen(
                        ui,
                        ctx,
                        &self.theme,
                        &self.config,
                        &self.status,
                        &self.supervisor,
                        self.logo_dark.as_ref(),
                        self.logo_light.as_ref(),
                        &mut self.feedback,
                    );
                }
                Screen::Settings => {
                    let mut go_back = false;
                    settings_view::show_settings_header(ui, &self.theme, || go_back = true);
                    if go_back {
                        self.current_screen = Screen::Main;
                    }

                    settings_view::show_settings_screen(
                        ui,
                        &self.theme,
                        &mut self.config,
                        &self.supervisor,
                        &mut self.feedback,
                        &mut self.maintenance,
                    );
                }
            });
    }
}
