use crate::domain::GpuInfo;
#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
pub fn detect_windows_display_controllers() -> Vec<GpuInfo> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let ps_cmd = "Get-CimInstance Win32_VideoController | Select-Object Name, AdapterRAM, DriverVersion | ConvertTo-Json -Compress";
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps_cmd])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let mut list = Vec::new();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let trimmed = stdout.trim();
            if !trimmed.is_empty() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    let items = match val {
                        serde_json::Value::Array(arr) => arr,
                        serde_json::Value::Object(_) => vec![val],
                        _ => vec![],
                    };
                    for item in items {
                        if let Some(name) = item.get("Name").and_then(|n| n.as_str()) {
                            let name_trimmed = name.trim();
                            let name_lower = name_trimmed.to_lowercase();
                            if name_lower.contains("microsoft basic")
                                || name_lower.contains("remote display")
                                || name_lower.contains("virtualbox")
                                || name_lower.contains("vmware")
                                || name_lower.contains("citrix")
                                || name_lower.contains("parallels")
                                || name_lower.contains("rdp")
                                || name_lower.contains("vnc")
                            {
                                continue;
                            }

                            let is_nvidia = name_lower.contains("nvidia")
                                || name_lower.contains("geforce")
                                || name_lower.contains("quadro")
                                || name_lower.contains("rtx")
                                || name_lower.contains("gtx")
                                || name_lower.contains("tesla")
                                || name_lower.contains("titan");
                            let is_amd =
                                name_lower.contains("amd") || name_lower.contains("radeon");
                            let is_intel = name_lower.contains("intel");

                            let vendor = if is_nvidia {
                                "NVIDIA".to_string()
                            } else if is_amd {
                                "AMD".to_string()
                            } else if is_intel {
                                "Intel".to_string()
                            } else {
                                "Unknown".to_string()
                            };

                            let is_dedicated = if is_nvidia {
                                true
                            } else if is_amd {
                                name_lower.contains(" rx")
                                    || name_lower.contains("pro")
                                    || name_lower.contains("vega")
                                    || name_lower.contains("xt")
                                    || name_lower.contains("firepro")
                                    || !name_lower.contains("graphics")
                            } else if is_intel {
                                name_lower.contains("arc")
                            } else {
                                false
                            };

                            let ram = item.get("AdapterRAM").and_then(|r| r.as_u64());
                            let driver = item
                                .get("DriverVersion")
                                .and_then(|d| d.as_str())
                                .map(|s| s.to_string());

                            list.push(GpuInfo {
                                name: name_trimmed.to_string(),
                                vendor,
                                memory_total_bytes: ram,
                                driver_version: driver,
                                is_dedicated,
                            });
                        }
                    }
                }
            }
        }
    }
    list
}

#[cfg(not(windows))]
pub fn detect_windows_display_controllers() -> Vec<GpuInfo> {
    Vec::new()
}
