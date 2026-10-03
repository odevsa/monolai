use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ManifestFlag {
    pub flag: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub flag_type: String,
    pub important: bool,
    #[serde(default)]
    pub default_value: String,
    #[serde(default)]
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ExtraArchive {
    pub url: String,
    pub archive_type: String, // "zip", "tar.gz"
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DownloadTarget {
    pub url: String,
    pub archive_type: String, // "zip", "tar.gz", "raw"
    #[serde(default)]
    pub extra_archives: Vec<ExtraArchive>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AccelerationOption {
    pub id: String,
    pub label: String,
    pub is_recommended: bool,
}

pub fn format_acceleration_label(id: &str) -> String {
    match id {
        "cpu" => "CPU Only".to_string(),
        "vulkan" => "Vulkan (Cross-vendor)".to_string(),
        "rocm" => "AMD ROCm".to_string(),
        "metal" => "Apple Metal".to_string(),
        "cuda" => "NVIDIA CUDA".to_string(),
        s if s.starts_with("cuda-") => {
            let ver = s.trim_start_matches("cuda-");
            format!("NVIDIA CUDA {}", ver)
        }
        other => other.to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RuntimeManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    pub description: String,
    #[serde(default)]
    pub features: Vec<String>,
    pub binary_name: String,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub downloads: HashMap<String, HashMap<String, HashMap<String, DownloadTarget>>>, // os -> arch -> acceleration -> target
    #[serde(default)]
    pub flags: Vec<ManifestFlag>,
}

impl RuntimeManifest {
    /// Find appropriate download target for specified OS, architecture and hardware acceleration.
    pub fn get_download_target(
        &self,
        os: &str,
        arch: &str,
        acceleration: &str,
    ) -> Option<DownloadTarget> {
        let os_key = if os == "darwin" { "macos" } else { os };
        let arch_key = if arch == "aarch64" { "arm64" } else { arch };

        let arch_map = self.downloads.get(os_key)?;
        let accel_map = arch_map.get(arch_key)?;

        let interpolate = |dt: &DownloadTarget| -> DownloadTarget {
            DownloadTarget {
                url: dt.url.replace("{version}", &self.version),
                archive_type: dt.archive_type.clone(),
                extra_archives: dt
                    .extra_archives
                    .iter()
                    .map(|ea| ExtraArchive {
                        url: ea.url.replace("{version}", &self.version),
                        archive_type: ea.archive_type.clone(),
                    })
                    .collect(),
            }
        };

        // 1. Direct match (e.g. "cuda-12.4", "vulkan", "cpu")
        if let Some(target) = accel_map.get(acceleration) {
            return Some(interpolate(target));
        }

        // 2. Generic "cuda" requested -> find exact "cuda" or first "cuda-*"
        if acceleration == "cuda" {
            if let Some(target) = accel_map.get("cuda") {
                return Some(interpolate(target));
            }
            for (key, target) in accel_map {
                if key.starts_with("cuda") {
                    return Some(interpolate(target));
                }
            }
        }

        // 3. Specific "cuda-*" requested but only generic "cuda" exists
        if acceleration.starts_with("cuda") {
            if let Some(target) = accel_map.get("cuda") {
                return Some(interpolate(target));
            }
        }

        // 4. Metal requested -> fallback to cpu if not found
        if acceleration == "metal" {
            if let Some(target) = accel_map.get("metal") {
                return Some(interpolate(target));
            }
        }

        // 5. Fallback to CPU
        if let Some(target) = accel_map.get("cpu") {
            return Some(interpolate(target));
        }

        None
    }

    /// List all acceleration choices available for this OS and arch
    pub fn get_available_accelerations(&self, os: &str, arch: &str) -> Vec<String> {
        let os_key = if os == "darwin" { "macos" } else { os };
        let arch_key = if arch == "aarch64" { "arm64" } else { arch };

        let mut list = Vec::new();
        if let Some(arch_map) = self.downloads.get(os_key) {
            if let Some(accel_map) = arch_map.get(arch_key) {
                for key in accel_map.keys() {
                    list.push(key.clone());
                }
            }
        }

        // If specific CUDA versions exist (like "cuda-12.4", "cuda-13.4"), remove the redundant generic "cuda"
        let has_specific_cuda = list.iter().any(|k| k.starts_with("cuda-"));
        if has_specific_cuda {
            list.retain(|k| k != "cuda");
        }

        // Sort: cuda first, then metal/rocm, then vulkan, then cpu
        list.sort_by(|a, b| {
            let score = |s: &str| {
                if s.starts_with("cuda") {
                    1
                } else if s.starts_with("metal") {
                    2
                } else if s.starts_with("rocm") {
                    3
                } else if s.starts_with("vulkan") {
                    4
                } else if s == "cpu" {
                    5
                } else {
                    6
                }
            };
            score(a).cmp(&score(b)).then_with(|| a.cmp(b))
        });

        list
    }
}

const LLAMA_CPP_MANIFEST: &str = include_str!("manifests/llama-cpp.yaml");
const SD_CPP_MANIFEST: &str = include_str!("manifests/sd-cpp.yaml");

/// Retrieve all available runtime manifests parsed from disk (if present) or embedded YAML.
pub fn get_runtime_manifests() -> Vec<RuntimeManifest> {
    let mut manifests = Vec::new();

    // Try reading from file on disk first, fallback to embedded
    let llama_yaml = std::fs::read_to_string("backend/src/runtimes/manifests/llama-cpp.yaml")
        .or_else(|_| std::fs::read_to_string("src/runtimes/manifests/llama-cpp.yaml"))
        .unwrap_or_else(|_| LLAMA_CPP_MANIFEST.to_string());

    if let Ok(m) = serde_yaml::from_str::<RuntimeManifest>(&llama_yaml) {
        manifests.push(m);
    } else if let Ok(m) = serde_yaml::from_str::<RuntimeManifest>(LLAMA_CPP_MANIFEST) {
        manifests.push(m);
    } else {
        tracing::error!("Failed to parse embedded llama-cpp.yaml manifest");
    }

    let sd_yaml = std::fs::read_to_string("backend/src/runtimes/manifests/sd-cpp.yaml")
        .or_else(|_| std::fs::read_to_string("src/runtimes/manifests/sd-cpp.yaml"))
        .unwrap_or_else(|_| SD_CPP_MANIFEST.to_string());

    if let Ok(m) = serde_yaml::from_str::<RuntimeManifest>(&sd_yaml) {
        manifests.push(m);
    } else if let Ok(m) = serde_yaml::from_str::<RuntimeManifest>(SD_CPP_MANIFEST) {
        manifests.push(m);
    } else {
        tracing::error!("Failed to parse embedded sd-cpp.yaml manifest");
    }

    manifests
}

/// Retrieve a manifest by matching runtime ID.
pub fn get_manifest_for_runtime(runtime_id: &str) -> Option<RuntimeManifest> {
    let manifests = get_runtime_manifests();
    manifests
        .into_iter()
        .find(|m| m.id.eq_ignore_ascii_case(runtime_id))
}
