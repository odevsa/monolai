use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
};

#[derive(Debug, Clone)]
pub enum TrayAction {
    OpenDashboard,
    Quit,
}

struct TrayContext {
    supervisor: crate::supervisor::ProcessSupervisor,
    tx: Sender<TrayAction>,
    egui_ctx: egui::Context,
}

static TRAY_CTX: Mutex<Option<TrayContext>> = Mutex::new(None);

pub struct TrayManager {
    _tray_icon: TrayIcon,
    status_item: MenuItem,
    icon_running: Icon,
    icon_stopped: Icon,
    action_rx: Receiver<TrayAction>,
}

impl TrayManager {
    pub fn new(
        egui_ctx: egui::Context,
        supervisor: crate::supervisor::ProcessSupervisor,
    ) -> Result<Self, String> {
        let menu = Menu::new();

        let status_item = MenuItem::with_id("status", "○ Monolai: Stopped", false, None);
        let show_panel_item = MenuItem::with_id("show_panel", "Open Dashboard", true, None);
        let quit_item = MenuItem::with_id("quit", "Quit Monolai", true, None);

        let _ = menu.append(&status_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&show_panel_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&quit_item);

        let icon_running = Self::load_icon(include_bytes!("../assets/tray/tray-running.png"))?;
        let icon_stopped = Self::load_icon(include_bytes!("../assets/tray/tray-stopped.png"))?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip("Monolai Service Manager")
            .with_icon(icon_stopped.clone())
            .build()
            .map_err(|e| format!("Failed to create system tray: {}", e))?;

        let (action_tx, action_rx) = channel();

        if let Ok(mut guard) = TRAY_CTX.lock() {
            *guard = Some(TrayContext {
                supervisor,
                tx: action_tx,
                egui_ctx: egui_ctx.clone(),
            });
        }

        MenuEvent::set_event_handler(Some(|event: MenuEvent| {
            let id = event.id.as_ref().to_string();

            if let Ok(guard) = TRAY_CTX.lock() {
                if let Some(ref tc) = *guard {
                    match id.as_str() {
                        "show_panel" => {
                            crate::ui::components::restore_window(&tc.egui_ctx);
                            let _ = tc.tx.send(TrayAction::OpenDashboard);
                            tc.egui_ctx.request_repaint();
                        }
                        "quit" => {
                            let _ = tc.supervisor.stop();
                            let _ = tc.tx.send(TrayAction::Quit);
                            tc.egui_ctx.request_repaint();
                            tc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            std::thread::sleep(std::time::Duration::from_millis(150));
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
            }
        }));

        TrayIconEvent::set_event_handler(Some(|event: TrayIconEvent| {
            match event {
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
                | TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    if let Ok(guard) = TRAY_CTX.lock() {
                        if let Some(ref tc) = *guard {
                            crate::ui::components::restore_window(&tc.egui_ctx);
                            let _ = tc.tx.send(TrayAction::OpenDashboard);
                            tc.egui_ctx.request_repaint();
                        }
                    }
                }
                _ => {}
            }
        }));

        Ok(Self {
            _tray_icon: tray,
            status_item,
            icon_running,
            icon_stopped,
            action_rx,
        })
    }

    fn load_icon(png_bytes: &[u8]) -> Result<Icon, String> {
        let img = image::load_from_memory(png_bytes)
            .map_err(|e| format!("Failed to parse tray icon png: {}", e))?
            .to_rgba8();
        let (width, height) = img.dimensions();
        Icon::from_rgba(img.into_raw(), width, height)
            .map_err(|e| format!("Failed to create tray icon from rgba: {}", e))
    }

    pub fn update_status(&mut self, is_running: bool, port: u16) {
        if is_running {
            let _ = self.status_item.set_text(format!("● Monolai: Running (:{})", port));
            let _ = self._tray_icon.set_icon(Some(self.icon_running.clone()));
            let _ = self._tray_icon.set_tooltip(Some(format!("Monolai: Running on port {}", port)));
        } else {
            let _ = self.status_item.set_text("○ Monolai: Stopped");
            let _ = self._tray_icon.set_icon(Some(self.icon_stopped.clone()));
            let _ = self._tray_icon.set_tooltip(Some("Monolai: Stopped"));
        }
    }

    pub fn poll_action(&self) -> Option<TrayAction> {
        self.action_rx.try_recv().ok()
    }
}
