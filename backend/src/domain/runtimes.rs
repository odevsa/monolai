use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ManifestFlag {
    pub flag: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub flag_type: String,
    #[serde(default)]
    pub important: bool,
    #[serde(default)]
    pub model: bool,
    #[serde(default)]
    pub default_value: String,
    #[serde(default)]
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ExtraArchive {
    pub url: String,
    pub archive_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DownloadTarget {
    pub url: String,
    pub archive_type: String,
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
        "openvino" => "Intel OpenVINO".to_string(),
        "sycl" => "Intel oneAPI SYCL".to_string(),
        "sycl-fp32" => "Intel SYCL (FP32)".to_string(),
        "sycl-fp16" => "Intel SYCL (FP16)".to_string(),
        "snapdragon" => "Qualcomm Snapdragon".to_string(),
        "opencl-adreno" => "Qualcomm OpenCL Adreno".to_string(),
        other => other.to_string(),
    }
}

fn default_gitlab_host() -> String {
    "https://gitlab.com".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum UpstreamConfig {
    #[serde(rename = "github_release")]
    GitHubRelease {
        repository: String,
        #[serde(default)]
        include_prereleases: bool,
        #[serde(default)]
        tag_regex: Option<String>,
    },
    #[serde(rename = "gitlab_release")]
    GitLabRelease {
        #[serde(default = "default_gitlab_host")]
        host: String,
        project_id: String,
        #[serde(default)]
        tag_regex: Option<String>,
    },
    #[serde(rename = "http_json")]
    HttpJson {
        url: String,
        #[serde(default)]
        variables: HashMap<String, String>,
    },
    #[serde(rename = "plain_text")]
    PlainText {
        url: String,
        regex: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpstreamRuntimeEntry {
    pub version: String,
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub last_checked_at: u64,
    #[serde(default)]
    pub release_notes_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RuntimeManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub upstream: Option<UpstreamConfig>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    pub description: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub supported_modalities: Vec<String>,
    #[serde(default)]
    pub supported_formats: Vec<String>,
    pub binary_name: String,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub downloads: HashMap<String, HashMap<String, HashMap<String, DownloadTarget>>>,
    #[serde(default)]
    pub flags: Vec<ManifestFlag>,
}

impl RuntimeManifest {
    pub fn find_model_flag(&self) -> Option<&ManifestFlag> {
        self.flags.iter().find(|f| f.model).or_else(|| {
            self.flags
                .iter()
                .find(|f| f.flag_type == "file" || f.flag == "--model" || f.flag == "--checkpoint")
        })
    }

    pub fn apply_upstream(&mut self, entry: &UpstreamRuntimeEntry) {
        self.version = entry.version.clone();
        for (key, val) in &entry.variables {
            self.variables.insert(key.clone(), val.clone());
        }
    }

    pub fn interpolate_string(&self, template: &str) -> String {
        let mut result = template.replace("{version}", &self.version);
        for (key, val) in &self.variables {
            result = result.replace(&format!("{{{}}}", key), val);
        }
        result
    }

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
                url: self.interpolate_string(&dt.url),
                archive_type: dt.archive_type.clone(),
                extra_archives: dt
                    .extra_archives
                    .iter()
                    .map(|ea| ExtraArchive {
                        url: self.interpolate_string(&ea.url),
                        archive_type: ea.archive_type.clone(),
                    })
                    .collect(),
            }
        };

        if let Some(target) = accel_map.get(acceleration) {
            return Some(interpolate(target));
        }

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

        if acceleration.starts_with("cuda") {
            if let Some(target) = accel_map.get("cuda") {
                return Some(interpolate(target));
            }
        }

        if acceleration == "metal" {
            if let Some(target) = accel_map.get("metal") {
                return Some(interpolate(target));
            }
        }

        if let Some(target) = accel_map.get("cpu") {
            return Some(interpolate(target));
        }

        None
    }

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

        let has_specific_cuda = list.iter().any(|k| k.starts_with("cuda-"));
        if has_specific_cuda {
            list.retain(|k| k != "cuda");
        }

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

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct InstallProgress {
    pub runtime_id: String,
    pub status: String,
    pub percent: f32,
    pub speed_mbps: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub error_message: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct RuntimeItem {
    pub id: String,
    pub name: String,
    pub version: String,
    pub icon: Option<String>,
    pub website: Option<String>,
    pub description: String,
    pub features: Vec<String>,
    pub is_installed: bool,
    pub installed_path: Option<String>,
    pub installed_version: Option<String>,
    pub has_update: bool,
    pub active_acceleration: String,
    pub installed_acceleration: Option<String>,
    pub available_accelerations: Vec<AccelerationOption>,
    pub install_progress: Option<InstallProgress>,
    #[serde(default)]
    pub release_notes_url: Option<String>,
}
