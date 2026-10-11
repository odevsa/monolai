#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub struct SingleInstanceGuard(windows_sys::Win32::Foundation::HANDLE);

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
pub fn ensure_single_instance(start_minimized: bool) -> Option<SingleInstanceGuard> {
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
pub struct SingleInstanceGuard(std::fs::File);

#[cfg(not(target_os = "windows"))]
pub fn ensure_single_instance(_start_minimized: bool) -> Option<SingleInstanceGuard> {
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
