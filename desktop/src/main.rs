mod app;
mod config;
mod supervisor;
mod theme;
mod tray;
mod ui;

use app::DesktopApp;
use eframe::egui::{self, IconData};
use std::sync::Arc;

fn load_window_icon() -> Option<IconData> {
    let bytes = include_bytes!("../assets/icons/logo-dark-128.png");
    if let Ok(img) = image::load_from_memory(bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Monolai")
        .with_app_id("monolai")
        .with_inner_size([500.0, 560.0])
        .with_min_inner_size([440.0, 480.0])
        .with_resizable(true)
        .with_minimize_button(true);

    if let Some(icon) = load_window_icon() {
        viewport = viewport.with_icon(Arc::new(icon));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Monolai",
        native_options,
        Box::new(|cc| {
            let initial_theme = theme::get_system_theme();
            theme::apply_theme(&cc.egui_ctx, &initial_theme);
            Ok(Box::new(DesktopApp::new(cc)))
        }),
    )
}
