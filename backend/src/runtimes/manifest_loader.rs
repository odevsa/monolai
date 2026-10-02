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
pub struct DownloadTarget {
    pub url: String,
    pub archive_type: String, // "zip", "tar.gz", "raw"
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
        let arch_map = self.downloads.get(os)?;
        let accel_map = arch_map.get(arch)?;

        // Try requested acceleration first, fallback to cpu
        if let Some(target) = accel_map.get(acceleration) {
            let interpolated_url = target.url.replace("{version}", &self.version);
            return Some(DownloadTarget {
                url: interpolated_url,
                archive_type: target.archive_type.clone(),
            });
        }

        if let Some(target) = accel_map.get("cpu") {
            let interpolated_url = target.url.replace("{version}", &self.version);
            return Some(DownloadTarget {
                url: interpolated_url,
                archive_type: target.archive_type.clone(),
            });
        }

        None
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
