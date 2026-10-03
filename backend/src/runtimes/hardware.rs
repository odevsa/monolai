use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub memory_total_bytes: Option<u64>,
    pub driver_version: Option<String>,
    pub is_dedicated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HardwareReport {
    pub os: String,
    pub arch: String,
    pub available_accelerations: Vec<String>,
    pub recommended_acceleration: String,
    pub detected_gpus: Vec<String>,
}

pub struct GpuTracker {
    pub gpus: Vec<GpuInfo>,
    pub primary_gpu: Option<GpuInfo>,
    usage_cache: Mutex<(Option<f32>, Instant)>,
    nvidia_smi_path: Option<PathBuf>,
}

fn run_cmd_no_window<P: AsRef<Path>>(cmd_path: P, args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut cmd = Command::new(cmd_path.as_ref());
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd.output()
}

fn find_nvidia_smi() -> Option<PathBuf> {
    if let Ok(output) = run_cmd_no_window("nvidia-smi", &["--version"]) {
        if output.status.success() {
            return Some(PathBuf::from("nvidia-smi"));
        }
    }

    #[cfg(windows)]
    {
        let standard_paths = [
            r"C:\Windows\System32\nvidia-smi.exe",
            r"C:\Program Files\NVIDIA Corporation\NVSMI\nvidia-smi.exe",
        ];
        for p in standard_paths {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }
    }

    #[cfg(not(windows))]
    {
        let linux_paths = ["/usr/bin/nvidia-smi", "/usr/local/cuda/bin/nvidia-smi"];
        for p in linux_paths {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

fn query_nvidia_smi_gpus(smi_path: &Path) -> Vec<GpuInfo> {
    let mut list = Vec::new();
    if let Ok(output) = run_cmd_no_window(
        smi_path,
        &["--query-gpu=name,driver_version,memory.total", "--format=csv,noheader,nounits"],
    ) {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if !parts.is_empty() && !parts[0].is_empty() {
                    let name = parts[0].to_string();
                    let driver = parts.get(1).filter(|s| !s.is_empty()).map(|s| s.to_string());
                    let mem_bytes = parts
                        .get(2)
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(|mib| mib * 1024 * 1024);

                    list.push(GpuInfo {
                        name,
                        vendor: "NVIDIA".to_string(),
                        memory_total_bytes: mem_bytes,
                        driver_version: driver,
                        is_dedicated: true,
                    });
                }
            }
        }
    }
    list
}

#[cfg(windows)]
fn detect_windows_display_controllers() -> Vec<GpuInfo> {
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
                            let is_amd = name_lower.contains("amd") || name_lower.contains("radeon");
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
fn detect_windows_display_controllers() -> Vec<GpuInfo> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn detect_linux_pci_gpus() -> Vec<GpuInfo> {
    let mut list = Vec::new();
    if let Ok(output) = Command::new("lspci").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line_lower = line.to_lowercase();
                if line_lower.contains("vga compatible controller")
                    || line_lower.contains("3d controller")
                    || line_lower.contains("display controller")
                {
                    let is_nvidia = line_lower.contains("nvidia");
                    let is_amd = line_lower.contains("amd")
                        || line_lower.contains("radeon")
                        || line_lower.contains("advanced micro devices");
                    let is_intel = line_lower.contains("intel");

                    let vendor = if is_nvidia {
                        "NVIDIA".to_string()
                    } else if is_amd {
                        "AMD".to_string()
                    } else if is_intel {
                        "Intel".to_string()
                    } else {
                        "Unknown".to_string()
                    };

                    let is_dedicated = is_nvidia
                        || (is_amd && !line_lower.contains("integrated"))
                        || (is_intel && line_lower.contains("arc"));
                    let name = line.split(':').last().unwrap_or(line).trim().to_string();

                    list.push(GpuInfo {
                        name,
                        vendor,
                        memory_total_bytes: None,
                        driver_version: None,
                        is_dedicated,
                    });
                }
            }
        }
    }
    list
}

#[cfg(not(target_os = "linux"))]
fn detect_linux_pci_gpus() -> Vec<GpuInfo> {
    Vec::new()
}

#[cfg(target_os = "macos")]
fn detect_macos_gpus() -> Vec<GpuInfo> {
    let mut list = Vec::new();
    if let Ok(out) = Command::new("sysctl").args(["-n", "machdep.cpu.brand_string"]).output() {
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
fn detect_macos_gpus() -> Vec<GpuInfo> {
    Vec::new()
}

impl GpuTracker {
    pub fn new() -> Self {
        let nvidia_smi_path = find_nvidia_smi();
        let mut gpus = Vec::new();

        // 1. Try nvidia-smi if found
        if let Some(ref smi) = nvidia_smi_path {
            let smi_gpus = query_nvidia_smi_gpus(smi);
            gpus.extend(smi_gpus);
        }

        // 2. Query Windows display controllers (NVIDIA, AMD, Intel)
        let win_gpus = detect_windows_display_controllers();
        for g in win_gpus {
            let exists = gpus.iter().any(|existing| {
                existing.name.eq_ignore_ascii_case(&g.name)
                    || (existing.vendor == g.vendor && existing.name.contains(&g.name))
                    || (existing.vendor == g.vendor && g.name.contains(&existing.name))
            });
            if !exists {
                gpus.push(g);
            }
        }

        // 3. Query Linux PCI display devices
        let linux_gpus = detect_linux_pci_gpus();
        for g in linux_gpus {
            let exists = gpus.iter().any(|existing| {
                existing.name.eq_ignore_ascii_case(&g.name)
                    || (existing.vendor == g.vendor && existing.name.contains(&g.name))
                    || (existing.vendor == g.vendor && g.name.contains(&existing.name))
            });
            if !exists {
                gpus.push(g);
            }
        }

        // 4. Query macOS display devices
        let mac_gpus = detect_macos_gpus();
        for g in mac_gpus {
            gpus.push(g);
        }

        // 5. Driver / filesystem fallbacks
        if gpus.is_empty() {
            if Path::new("/proc/driver/nvidia").exists()
                || Path::new("/dev/nvidia0").exists()
                || Path::new("C:\\Windows\\System32\\nvcuda.dll").exists()
            {
                gpus.push(GpuInfo {
                    name: "NVIDIA Dedicated GPU (CUDA)".to_string(),
                    vendor: "NVIDIA".to_string(),
                    memory_total_bytes: None,
                    driver_version: None,
                    is_dedicated: true,
                });
            } else if Path::new("/dev/kfd").exists() {
                gpus.push(GpuInfo {
                    name: "AMD ROCm GPU".to_string(),
                    vendor: "AMD".to_string(),
                    memory_total_bytes: None,
                    driver_version: None,
                    is_dedicated: true,
                });
            }
        }

        // Prioritize dedicated GPUs, then VRAM
        let mut sorted = gpus.clone();
        sorted.sort_by(|a, b| {
            b.is_dedicated
                .cmp(&a.is_dedicated)
                .then_with(|| b.memory_total_bytes.unwrap_or(0).cmp(&a.memory_total_bytes.unwrap_or(0)))
        });

        let primary_gpu = sorted.into_iter().next();

        Self {
            gpus,
            primary_gpu,
            usage_cache: Mutex::new((None, Instant::now() - Duration::from_secs(10))),
            nvidia_smi_path,
        }
    }

    pub fn primary_gpu(&self) -> Option<GpuInfo> {
        self.primary_gpu.clone()
    }

    #[allow(dead_code)]
    pub fn all_gpus(&self) -> &[GpuInfo] {
        &self.gpus
    }

    pub fn current_usage(&self) -> Option<f32> {
        let _primary = self.primary_gpu.as_ref()?;

        let mut cache = self.usage_cache.lock().unwrap();
        if let (Some(val), last_time) = *cache {
            if last_time.elapsed() < Duration::from_millis(1500) {
                return Some(val);
            }
        }

        // Query nvidia-smi if available
        if let Some(ref smi) = self.nvidia_smi_path {
            if let Ok(output) = run_cmd_no_window(
                smi,
                &["--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"],
            ) {
                if output.status.success() {
                    let out_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(line) = out_str.lines().next() {
                        if let Ok(pct) = line.trim().parse::<f32>() {
                            *cache = (Some(pct), Instant::now());
                            return Some(pct);
                        }
                    }
                }
            }
        }

        // Query Linux AMD sysfs if available
        #[cfg(target_os = "linux")]
        {
            let paths = [
                "/sys/class/drm/card0/device/gpu_busy_percent",
                "/sys/class/drm/card1/device/gpu_busy_percent",
            ];
            for p in paths {
                if let Ok(content) = std::fs::read_to_string(p) {
                    if let Ok(pct) = content.trim().parse::<f32>() {
                        *cache = (Some(pct), Instant::now());
                        return Some(pct);
                    }
                }
            }
        }

        // Dedicated GPU exists, but dynamic utilization counter unavailable on this platform
        let default_val = 0.0;
        *cache = (Some(default_val), Instant::now());
        Some(default_val)
    }
}

/// Detect host system hardware acceleration capabilities.
pub fn detect_hardware() -> HardwareReport {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();

    let tracker = GpuTracker::new();
    let mut available = vec!["cpu".to_string()];
    let mut detected_gpus = Vec::new();

    let mut has_nvidia = false;
    let mut has_amd = false;
    let mut has_vulkan = false;

    for gpu in &tracker.gpus {
        let label = if let Some(mem) = gpu.memory_total_bytes {
            let mib = mem / (1024 * 1024);
            format!("{} ({} MiB)", gpu.name, mib)
        } else {
            gpu.name.clone()
        };
        detected_gpus.push(label);

        if gpu.vendor == "NVIDIA" {
            has_nvidia = true;
        } else if gpu.vendor == "AMD" {
            has_amd = true;
        }
    }

    if !has_nvidia
        && (Path::new("/proc/driver/nvidia").exists()
            || Path::new("/dev/nvidia0").exists()
            || Path::new("C:\\Windows\\System32\\nvcuda.dll").exists())
    {
        has_nvidia = true;
        if detected_gpus.is_empty() {
            detected_gpus.push("NVIDIA GPU (CUDA)".to_string());
        }
    }

    if !has_amd && Path::new("/dev/kfd").exists() {
        has_amd = true;
        if !detected_gpus.iter().any(|g| g.contains("AMD")) {
            detected_gpus.push("AMD ROCm GPU".to_string());
        }
    }

    // Check for Vulkan
    if Path::new("/usr/share/vulkan/icd.d").exists()
        || Path::new("C:\\Windows\\System32\\vulkan-1.dll").exists()
        || Command::new("vulkaninfo").arg("--summary").output().is_ok()
    {
        has_vulkan = true;
        available.push("vulkan".to_string());
    }

    if os == "macos" {
        available.push("metal".to_string());
    }

    if has_nvidia {
        available.push("cuda".to_string());
    }
    if has_amd {
        available.push("rocm".to_string());
    }

    // Determine recommended acceleration
    let recommended = if has_nvidia {
        "cuda".to_string()
    } else if has_amd {
        "rocm".to_string()
    } else if os == "macos" {
        "metal".to_string()
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
        Some(s) if s.starts_with("cuda") => s.to_string(),
        Some("rocm") => "rocm".to_string(),
        Some("vulkan") => "vulkan".to_string(),
        Some("metal") => "metal".to_string(),
        Some("oneapi") => "oneapi".to_string(),
        Some("cpu") => "cpu".to_string(),
        _ => report.recommended_acceleration,
    }
}
