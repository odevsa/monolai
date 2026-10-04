#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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

#[cfg(target_os = "windows")]
#[allow(dead_code)]
struct SingleInstanceGuard(windows_sys::Win32::Foundation::HANDLE);

#[cfg(target_os = "windows")]
impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(self.0);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn ensure_single_instance(start_minimized: bool) -> Option<SingleInstanceGuard> {
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;

    let mutex_name: Vec<u16> = "Local\\Monolai_GUI_SingleInstance_Mutex\0"
        .encode_utf16()
        .collect();

    unsafe {
        let handle = CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr());
        if handle.is_null() {
            return None;
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            windows_sys::Win32::Foundation::CloseHandle(handle);
            if !start_minimized {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
                };
                let title: Vec<u16> = "Monolai\0".encode_utf16().collect();
                let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
                if !hwnd.is_null() {
                    ShowWindow(hwnd, SW_SHOW);
                    ShowWindow(hwnd, SW_RESTORE);
                    SetForegroundWindow(hwnd);
                }
            }
            std::process::exit(0);
        }
        Some(SingleInstanceGuard(handle))
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
struct SingleInstanceGuard(std::fs::File);

#[cfg(not(target_os = "windows"))]
fn ensure_single_instance(_start_minimized: bool) -> Option<SingleInstanceGuard> {
    use std::os::unix::io::AsRawFd;
    let lock_dir = dirs::runtime_dir().or_else(dirs::cache_dir)?;
    let _ = std::fs::create_dir_all(&lock_dir);
    let lock_path = lock_dir.join("monolai-gui.lock");
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .ok()?;
    let fd = file.as_raw_fd();
    let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
    if ret != 0 {
        eprintln!("Monolai GUI is already running.");
        std::process::exit(0);
    }
    Some(SingleInstanceGuard(file))
}

fn main() -> eframe::Result<()> {
    let start_minimized = std::env::args().any(|a| a == "--minimized");
    let _single_instance = ensure_single_instance(start_minimized);

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Monolai")
        .with_app_id("monolai")
        .with_inner_size([500.0, 560.0])
        .with_min_inner_size([440.0, 480.0])
        .with_resizable(true)
        .with_minimize_button(true)
        .with_close_button(true);

    if start_minimized {
        viewport = viewport.with_visible(false);
    }

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
