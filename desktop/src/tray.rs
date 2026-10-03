use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
};

#[derive(Debug, Clone)]
pub enum TrayAction {
    Menu(String),
    IconClick,
    IconDoubleClick,
}

static ACTION_TX: Mutex<Option<(Sender<TrayAction>, egui::Context)>> = Mutex::new(None);

pub struct TrayManager {
    _tray_icon: TrayIcon,
    status_item: MenuItem,
    open_web_item: MenuItem,
    show_panel_item: MenuItem,
    start_item: MenuItem,
    stop_item: MenuItem,
    restart_item: MenuItem,
    icon_running: Icon,
    icon_stopped: Icon,
    action_rx: Receiver<TrayAction>,
}

impl TrayManager {
    pub fn new(egui_ctx: egui::Context) -> Result<Self, String> {
        let menu = Menu::new();

        let status_item = MenuItem::with_id("status", "○ Monolai: Stopped (offline)", false, None);
        let open_web_item = MenuItem::with_id("open_web", "Open Web UI (offline)", false, None);
        let show_panel_item = MenuItem::with_id("show_panel", "Open Dashboard", true, None);
        let start_item = MenuItem::with_id("start_server", "Start Service", true, None);
        let stop_item = MenuItem::with_id("stop_server", "Stop Service (inactive)", false, None);
        let restart_item = MenuItem::with_id("restart_server", "Restart Service (inactive)", false, None);
        let quit_item = MenuItem::with_id("quit", "Quit Monolai", true, None);

        let _ = menu.append(&status_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&open_web_item);
        let _ = menu.append(&show_panel_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&start_item);
        let _ = menu.append(&stop_item);
        let _ = menu.append(&restart_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&quit_item);

        let icon_running = Self::load_icon(include_bytes!("../../assets/tray/tray-running.png"))?;
        let icon_stopped = Self::load_icon(include_bytes!("../../assets/tray/tray-stopped.png"))?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Monolai Service Manager")
            .with_icon(icon_stopped.clone())
            .build()
            .map_err(|e| format!("Failed to create system tray: {}", e))?;

        let (action_tx, action_rx) = channel();

        // Update global sender and context so the persistent handler forwards to the active receiver
        if let Ok(mut guard) = ACTION_TX.lock() {
            *guard = Some((action_tx, egui_ctx.clone()));
        }

        // Register handlers once. The closure forwards dynamically to ACTION_TX
        MenuEvent::set_event_handler(Some(|event: MenuEvent| {
            if let Ok(guard) = ACTION_TX.lock() {
                if let Some((ref tx, ref ctx)) = *guard {
                    let id = event.id.as_ref().to_string();
                    let _ = tx.send(TrayAction::Menu(id));
                    ctx.request_repaint();
                }
            }
        }));

        static LAST_CLICK: Mutex<Option<std::time::Instant>> = Mutex::new(None);

        TrayIconEvent::set_event_handler(Some(|event: TrayIconEvent| {
            match event {
                TrayIconEvent::DoubleClick { .. } => {
                    if let Ok(guard) = ACTION_TX.lock() {
                        if let Some((ref tx, ref ctx)) = *guard {
                            let _ = tx.send(TrayAction::IconDoubleClick);
                            ctx.request_repaint();
                        }
                    }
                }
                TrayIconEvent::Click { .. } => {
                    let mut is_double = false;
                    if let Ok(mut last) = LAST_CLICK.lock() {
                        if let Some(prev) = *last {
                            if prev.elapsed() < std::time::Duration::from_millis(500) {
                                is_double = true;
                                *last = None;
                            } else {
                                *last = Some(std::time::Instant::now());
                            }
                        } else {
                            *last = Some(std::time::Instant::now());
                        }
                    }

                    if let Ok(guard) = ACTION_TX.lock() {
                        if let Some((ref tx, ref ctx)) = *guard {
                            if is_double {
                                let _ = tx.send(TrayAction::IconDoubleClick);
                            } else {
                                let _ = tx.send(TrayAction::IconClick);
                            }
                            ctx.request_repaint();
                        }
                    }
                }
                _ => {}
            }
        }));

        Ok(Self {
            _tray_icon: tray,
            status_item,
            open_web_item,
            show_panel_item,
            start_item,
            stop_item,
            restart_item,
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
            let _ = self.status_item.set_enabled(false);

            let _ = self.open_web_item.set_text("Open Web UI");
            let _ = self.open_web_item.set_enabled(true);

            let _ = self.show_panel_item.set_text("Open Dashboard");
            let _ = self.show_panel_item.set_enabled(true);

            let _ = self.start_item.set_text("Start Service (running)");
            let _ = self.start_item.set_enabled(false);

            let _ = self.stop_item.set_text("Stop Service");
            let _ = self.stop_item.set_enabled(true);

            let _ = self.restart_item.set_text("Restart Service");
            let _ = self.restart_item.set_enabled(true);

            let _ = self._tray_icon.set_icon(Some(self.icon_running.clone()));
            let _ = self._tray_icon.set_tooltip(Some(format!("Monolai: Running on port {}", port)));
        } else {
            let _ = self.status_item.set_text("○ Monolai: Stopped (offline)");
            let _ = self.status_item.set_enabled(false);

            let _ = self.open_web_item.set_text("Open Web UI (offline)");
            let _ = self.open_web_item.set_enabled(false);

            let _ = self.show_panel_item.set_text("Open Dashboard");
            let _ = self.show_panel_item.set_enabled(true);

            let _ = self.start_item.set_text("Start Service");
            let _ = self.start_item.set_enabled(true);

            let _ = self.stop_item.set_text("Stop Service (inactive)");
            let _ = self.stop_item.set_enabled(false);

            let _ = self.restart_item.set_text("Restart Service (inactive)");
            let _ = self.restart_item.set_enabled(false);

            let _ = self._tray_icon.set_icon(Some(self.icon_stopped.clone()));
            let _ = self._tray_icon.set_tooltip(Some("Monolai: Stopped"));
        }
    }

    pub fn poll_action(&self) -> Option<TrayAction> {
        self.action_rx.try_recv().ok()
    }
}
