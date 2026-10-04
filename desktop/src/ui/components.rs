use eframe::egui::{self, Align, Color32, Margin, Ui, Vec2};

/// TextEdit singleline with vertically centered text, padding, and text clipping
pub fn singleline_input<'a>(text: &'a mut String) -> egui::TextEdit<'a> {
    egui::TextEdit::singleline(text)
        .vertical_align(Align::Center)
        .margin(Margin::symmetric(10.0, 6.0))
        .clip_text(true)
}

/// Draw a clean circular status dot (pure GPU vector paint, never broken by missing fonts)
pub fn draw_status_dot(ui: &mut Ui, color: Color32, radius: f32) {
    let size = Vec2::splat((radius + 1.0) * 2.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), radius, color);
}

#[cfg(target_os = "windows")]
static WINDOWS_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

#[cfg(target_os = "windows")]
pub fn set_windows_hwnd(hwnd: isize) {
    WINDOWS_HWND.store(hwnd, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(target_os = "windows")]
pub fn get_windows_hwnd() -> isize {
    WINDOWS_HWND.load(std::sync::atomic::Ordering::SeqCst)
}

/// Minimize window to tray across Wayland and X11/Windows/macOS
pub fn minimize_window(ctx: &egui::Context) {
    #[cfg(target_os = "windows")]
    {
        let hwnd = WINDOWS_HWND.load(std::sync::atomic::Ordering::SeqCst);
        if hwnd != 0 {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
                ShowWindow(hwnd as _, SW_HIDE);
            }
        }
    }

    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    #[cfg(not(target_os = "windows"))]
    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    ctx.request_repaint();
}

/// Restore window from tray across Wayland (COSMIC) and X11/Windows/macOS
pub fn restore_window(ctx: &egui::Context) {
    #[cfg(target_os = "windows")]
    {
        let hwnd = WINDOWS_HWND.load(std::sync::atomic::Ordering::SeqCst);
        if hwnd != 0 {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    ShowWindow, SetForegroundWindow, SW_RESTORE, SW_SHOW,
                };
                ShowWindow(hwnd as _, SW_SHOW);
                ShowWindow(hwnd as _, SW_RESTORE);
                SetForegroundWindow(hwnd as _);
            }
        }
    }

    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
    ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
        egui::UserAttentionType::Critical,
    ));
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    ctx.request_repaint();
}

/// Load and register a PNG image buffer as an egui TextureHandle
pub fn load_png_texture(
    ctx: &egui::Context,
    name: &str,
    png_bytes: &[u8],
) -> Option<egui::TextureHandle> {
    if let Ok(img) = image::load_from_memory(png_bytes) {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            rgba.as_flat_samples().as_slice(),
        );
        Some(ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR))
    } else {
        None
    }
}
