use crate::domain::{GpuDeviceInfo, GpuInfo};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run_cmd_no_window<P: AsRef<Path>>(
    cmd_path: P,
    args: &[&str],
) -> std::io::Result<std::process::Output> {
    let mut cmd = Command::new(cmd_path.as_ref());
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd.output()
}

pub fn find_nvidia_smi() -> Option<PathBuf> {
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

pub fn query_nvidia_smi_gpus(smi_path: &Path) -> Vec<GpuInfo> {
    let mut list = Vec::new();
    if let Ok(output) = run_cmd_no_window(
        smi_path,
        &[
            "--query-gpu=name,driver_version,memory.total",
            "--format=csv,noheader,nounits",
        ],
    ) {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if !parts.is_empty() && !parts[0].is_empty() {
                    let name = parts[0].to_string();
                    let driver = parts
                        .get(1)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
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

pub fn query_nvidia_metrics(smi_path: &Path) -> Vec<GpuDeviceInfo> {
    let mut results = Vec::new();
    if let Ok(output) = run_cmd_no_window(
        smi_path,
        &[
            "--query-gpu=index,name,driver_version,memory.total,memory.used,memory.free,utilization.gpu,temperature.gpu,clocks.current.graphics",
            "--format=csv,noheader,nounits",
        ],
    ) {
        if output.status.success() {
            let out_str = String::from_utf8_lossy(&output.stdout);
            for line in out_str.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    let idx = parts
                        .first()
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(results.len());
                    let name = parts.get(1).unwrap_or(&"NVIDIA GPU").to_string();
                    let driver = parts.get(2).filter(|s| !s.is_empty()).map(|s| s.to_string());
                    let mem_total = parts
                        .get(3)
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(|m| m * 1024 * 1024);
                    let mem_used = parts
                        .get(4)
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(|m| m * 1024 * 1024);
                    let mem_free = parts
                        .get(5)
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(|m| m * 1024 * 1024);
                    let usage = parts.get(6).and_then(|s| s.parse::<f32>().ok());
                    let temp_c = parts.get(7).and_then(|s| s.parse::<f32>().ok());
                    let clock_mhz = parts.get(8).and_then(|s| s.parse::<u32>().ok());

                    results.push(GpuDeviceInfo {
                        index: idx,
                        name,
                        vendor: "NVIDIA".to_string(),
                        memory_total_bytes: mem_total,
                        memory_used_bytes: mem_used,
                        memory_free_bytes: mem_free,
                        driver_version: driver,
                        is_dedicated: true,
                        usage,
                        temperature_c: temp_c,
                        clock_mhz,
                    });
                }
            }
        }
    }
    results
}
