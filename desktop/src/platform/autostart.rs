pub fn is_autostart_app_enabled() -> bool {
    if let Ok(exe_path) = std::env::current_exe() {
        let mut builder = auto_launch::AutoLaunchBuilder::new();
        builder
            .set_app_name("Monolai")
            .set_app_path(&exe_path.to_string_lossy());

        #[cfg(target_os = "windows")]
        builder.set_windows_enable_mode(auto_launch::WindowsEnableMode::CurrentUser);

        if let Ok(auto) = builder.build() {
            return auto.is_enabled().unwrap_or(false);
        }
    }
    false
}

pub fn set_autostart_app(enable: bool, minimized: bool) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        // Clean up any stale HKLM Run entry created by past runs or elevated installers
        let mut sys_builder = auto_launch::AutoLaunchBuilder::new();
        sys_builder
            .set_app_name("Monolai")
            .set_app_path(&exe_path.to_string_lossy())
            .set_windows_enable_mode(auto_launch::WindowsEnableMode::System);
        if let Ok(sys_auto) = sys_builder.build() {
            let _ = sys_auto.disable();
        }
    }

    let mut builder = auto_launch::AutoLaunchBuilder::new();
    builder
        .set_app_name("Monolai")
        .set_app_path(&exe_path.to_string_lossy());

    #[cfg(target_os = "windows")]
    builder.set_windows_enable_mode(auto_launch::WindowsEnableMode::CurrentUser);

    if minimized {
        builder.set_args(&["--minimized"]);
    }

    let auto = builder
        .build()
        .map_err(|e| format!("Failed to build autostart config: {}", e))?;

    if enable {
        let _ = auto.enable();
    } else {
        let _ = auto.disable();
    }
    Ok(())
}
