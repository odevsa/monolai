use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HardwareReport {
    pub os: String,
    pub arch: String,
    pub available_accelerations: Vec<String>,
    pub recommended_acceleration: String,
    pub detected_gpus: Vec<String>,
}

/// Detect host system hardware acceleration capabilities.
pub fn detect_hardware() -> HardwareReport {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();

    let mut available = vec!["cpu".to_string()];
    let mut detected_gpus = Vec::new();
    let mut has_nvidia = false;
    let mut has_amd = false;
    let mut has_vulkan = false;

    // Check for NVIDIA GPU / CUDA
    if Path::new("/proc/driver/nvidia").exists()
        || Path::new("/dev/nvidia0").exists()
        || Path::new("C:\\Windows\\System32\\nvcuda.dll").exists()
    {
        has_nvidia = true;
    } else if let Ok(output) = Command::new("nvidia-smi").arg("-L").output() {
        if output.status.success() {
            has_nvidia = true;
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if !line.trim().is_empty() {
                    detected_gpus.push(line.trim().to_string());
                }
            }
        }
    }

    if has_nvidia {
        available.push("cuda".to_string());
        if detected_gpus.is_empty() {
            detected_gpus.push("NVIDIA GPU (CUDA)".to_string());
        }
    }

    // Check for AMD ROCm
    if Path::new("/dev/kfd").exists() {
        has_amd = true;
        available.push("rocm".to_string());
        detected_gpus.push("AMD ROCm GPU".to_string());
    }

    // Check for Vulkan
    if Path::new("/usr/share/vulkan/icd.d").exists()
        || Command::new("vulkaninfo").arg("--summary").output().is_ok()
    {
        has_vulkan = true;
        available.push("vulkan".to_string());
    }

    // Determine recommended acceleration
    let recommended = if has_nvidia {
        "cuda".to_string()
    } else if has_amd {
        "rocm".to_string()
    } else if has_vulkan {
        "vulkan".to_string()
    } else {
        "cpu".to_string()
    };

    HardwareReport {
        os,
        arch,
        available_accelerations: available,
        recommended_acceleration: recommended,
        detected_gpus,
    }
}

/// Resolve the active acceleration string according to user config and host capabilities.
pub fn resolve_target_acceleration(configured_hw: Option<&str>) -> String {
    let report = detect_hardware();
    match configured_hw {
        Some("cuda") => "cuda".to_string(),
        Some("rocm") => "rocm".to_string(),
        Some("vulkan") => "vulkan".to_string(),
        Some("oneapi") => "oneapi".to_string(),
        Some("cpu") => "cpu".to_string(),
        _ => report.recommended_acceleration,
    }
}
