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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpstreamRuntimeEntry {
    pub version: String,
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub last_checked_at: u64,
    #[serde(default)]
    pub release_notes_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SystemDependency {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub check_library: Option<String>,
    #[serde(default)]
    pub check_binary: Option<String>,
    #[serde(default)]
    pub instructions: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SystemDependencyStatus {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_satisfied: bool,
    pub install_command: Option<String>,
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
    #[serde(default)]
    pub system_dependencies: Vec<SystemDependency>,
    #[serde(default)]
    pub release_assets: Vec<ReleaseAsset>,
}

static VERSION_PATTERN_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\d+(\.\d+)+").unwrap());

pub fn matches_wildcard(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    if !text.starts_with(parts[0]) {
        return false;
    }
    let mut remainder = &text[parts[0].len()..];
    for &part in &parts[1..parts.len() - 1] {
        if let Some(pos) = remainder.find(part) {
            remainder = &remainder[pos + part.len()..];
        } else {
            return false;
        }
    }
    remainder.ends_with(parts[parts.len() - 1])
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
        if !entry.assets.is_empty() {
            self.release_assets = entry.assets.clone();
        }
    }

    pub fn interpolate_string(&self, template: &str) -> String {
        let mut result = template.replace("{version}", &self.version);
        for (key, val) in &self.variables {
            result = result.replace(&format!("{{{}}}", key), val);
        }
        result
    }

    pub fn resolve_download_url(&self, template: &str) -> String {
        let interpolated = self.interpolate_string(template);
        if self.release_assets.is_empty() {
            return interpolated;
        }

        let pattern_filename = match interpolated.split('/').last() {
            Some(f) if !f.is_empty() => f,
            _ => return interpolated,
        };

        // 1. Direct exact filename match
        if let Some(asset) = self.release_assets.iter().find(|a| a.name == pattern_filename) {
            return asset.download_url.clone();
        }

        // 2. Wildcard pattern match (if pattern contains '*')
        if pattern_filename.contains('*') {
            let mut matches: Vec<&ReleaseAsset> = self
                .release_assets
                .iter()
                .filter(|a| matches_wildcard(pattern_filename, &a.name))
                .collect();
            // Sort descending so newer/higher versions (e.g. 26.04 over 24.04, 26.7.2 over 26.6.2) are prioritized
            matches.sort_by(|a, b| b.name.cmp(&a.name));
            if let Some(matched_asset) = matches.first() {
                return matched_asset.download_url.clone();
            }
        }

        // 3. Smart fallback: if literal URL didn't match directly, relax OS and toolchain versions
        if let Some(fuzzy_asset) = self.fuzzy_match_asset(pattern_filename) {
            return fuzzy_asset.download_url.clone();
        }

        interpolated
    }

    fn fuzzy_match_asset(&self, filename: &str) -> Option<&ReleaseAsset> {
        // Universally relax any semantic / dot-version strings (e.g. 24.04, 26.6.2, 7.14.0) into '*'
        let pattern = VERSION_PATTERN_REGEX.replace_all(filename, "*");

        if pattern.contains('*') {
            let mut matches: Vec<&ReleaseAsset> = self
                .release_assets
                .iter()
                .filter(|a| matches_wildcard(&pattern, &a.name))
                .collect();
            matches.sort_by(|a, b| b.name.cmp(&a.name));
            return matches.first().copied();
        }
        None
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
                url: self.resolve_download_url(&dt.url),
                archive_type: dt.archive_type.clone(),
                extra_archives: dt
                    .extra_archives
                    .iter()
                    .map(|ea| ExtraArchive {
                        url: self.resolve_download_url(&ea.url),
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
    #[serde(default)]
    pub system_dependencies: Vec<SystemDependencyStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_wildcard() {
        assert!(matches_wildcard(
            "sd-master-5a1ba5d-bin-Linux-Ubuntu-*-x86_64-vulkan.zip",
            "sd-master-5a1ba5d-bin-Linux-Ubuntu-26.04-x86_64-vulkan.zip"
        ));
        assert!(matches_wildcard(
            "sd-master-5a1ba5d-bin-Linux-Ubuntu-*-x86_64-vulkan.zip",
            "sd-master-5a1ba5d-bin-Linux-Ubuntu-24.04-x86_64-vulkan.zip"
        ));
        assert!(!matches_wildcard(
            "sd-master-5a1ba5d-bin-Linux-Ubuntu-*-x86_64-vulkan.zip",
            "sd-master-5a1ba5d-bin-Linux-Ubuntu-26.04-x86_64.zip"
        ));
        assert!(matches_wildcard(
            "sd-master-*-bin-Linux-Ubuntu-*-x86_64-rocm-*.zip",
            "sd-master-5a1ba5d-bin-Linux-Ubuntu-24.04-x86_64-rocm-10.1.0.zip"
        ));
        assert!(matches_wildcard(
            "sd-master-*-bin-Darwin-macOS-*-arm64.zip",
            "sd-master-5a1ba5d-bin-Darwin-macOS-26.7.2-arm64.zip"
        ));
    }

    #[test]
    fn test_resolve_download_url_with_wildcards() {
        let manifest = RuntimeManifest {
            id: "sd-cpp".to_string(),
            name: "Stable Diffusion".to_string(),
            version: "master-1053-5a1ba5d".to_string(),
            variables: HashMap::from([
                ("version".to_string(), "master-1053-5a1ba5d".to_string()),
                ("commit".to_string(), "5a1ba5d".to_string()),
            ]),
            upstream: None,
            icon: None,
            website: None,
            description: "".to_string(),
            features: vec![],
            supported_modalities: vec![],
            supported_formats: vec![],
            binary_name: "sd-server".to_string(),
            downloads: HashMap::new(),
            flags: vec![],
            system_dependencies: vec![],
            release_assets: vec![
                ReleaseAsset {
                    name: "sd-master-5a1ba5d-bin-Linux-Ubuntu-26.04-x86_64-vulkan.zip".to_string(),
                    download_url: "https://github.com/leejet/stable-diffusion.cpp/releases/download/master-1053-5a1ba5d/sd-master-5a1ba5d-bin-Linux-Ubuntu-26.04-x86_64-vulkan.zip".to_string(),
                    size: Some(1234),
                },
                ReleaseAsset {
                    name: "sd-master-5a1ba5d-bin-Linux-Ubuntu-26.04-x86_64.zip".to_string(),
                    download_url: "https://github.com/leejet/stable-diffusion.cpp/releases/download/master-1053-5a1ba5d/sd-master-5a1ba5d-bin-Linux-Ubuntu-26.04-x86_64.zip".to_string(),
                    size: Some(1234),
                },
                ReleaseAsset {
                    name: "sd-master-5a1ba5d-bin-Darwin-macOS-26.7.2-arm64.zip".to_string(),
                    download_url: "https://github.com/leejet/stable-diffusion.cpp/releases/download/master-1053-5a1ba5d/sd-master-5a1ba5d-bin-Darwin-macOS-26.7.2-arm64.zip".to_string(),
                    size: Some(1234),
                },
            ],
        };

        // 1. Explicit wildcard in template
        let template_vulkan = "https://github.com/leejet/stable-diffusion.cpp/releases/download/{version}/sd-master-{commit}-bin-Linux-Ubuntu-*-x86_64-vulkan.zip";
        let resolved_vulkan = manifest.resolve_download_url(template_vulkan);
        assert_eq!(
            resolved_vulkan,
            "https://github.com/leejet/stable-diffusion.cpp/releases/download/master-1053-5a1ba5d/sd-master-5a1ba5d-bin-Linux-Ubuntu-26.04-x86_64-vulkan.zip"
        );

        // 2. Legacy literal URL template with Ubuntu 24.04 (auto-relaxed via fuzzy matching)
        let legacy_template = "https://github.com/leejet/stable-diffusion.cpp/releases/download/{version}/sd-master-{commit}-bin-Linux-Ubuntu-24.04-x86_64-vulkan.zip";
        let resolved_legacy = manifest.resolve_download_url(legacy_template);
        assert_eq!(
            resolved_legacy,
            "https://github.com/leejet/stable-diffusion.cpp/releases/download/master-1053-5a1ba5d/sd-master-5a1ba5d-bin-Linux-Ubuntu-26.04-x86_64-vulkan.zip"
        );

        // 3. macOS version fluctuation (Darwin-macOS-26.6.2 in template vs 26.7.2 published)
        let macos_template = "https://github.com/leejet/stable-diffusion.cpp/releases/download/{version}/sd-master-{commit}-bin-Darwin-macOS-26.6.2-arm64.zip";
        let resolved_macos = manifest.resolve_download_url(macos_template);
        assert_eq!(
            resolved_macos,
            "https://github.com/leejet/stable-diffusion.cpp/releases/download/master-1053-5a1ba5d/sd-master-5a1ba5d-bin-Darwin-macOS-26.7.2-arm64.zip"
        );
    }
}
