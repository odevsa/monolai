use eframe::egui;

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

/// Minimize window to tray across Wayland, X11, Windows, and macOS
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

/// Restore window from tray across Wayland, X11, Windows, and macOS
pub fn restore_window(ctx: &egui::Context) {
    #[cfg(target_os = "windows")]
    {
        let hwnd = WINDOWS_HWND.load(std::sync::atomic::Ordering::SeqCst);
        if hwnd != 0 {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
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
