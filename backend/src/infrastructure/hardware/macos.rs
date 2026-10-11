use crate::domain::GpuInfo;
#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "macos")]
pub fn detect_macos_gpus() -> Vec<GpuInfo> {
    let mut list = Vec::new();
    if let Ok(out) = Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
    {
        let brand = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if brand.contains("Apple") {
            list.push(GpuInfo {
                name: format!("{} (Metal)", brand),
                vendor: "Apple".to_string(),
                memory_total_bytes: None,
                driver_version: None,
                is_dedicated: true,
            });
        }
    }
    list
}

#[cfg(not(target_os = "macos"))]
pub fn detect_macos_gpus() -> Vec<GpuInfo> {
    Vec::new()
}
