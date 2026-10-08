use crate::domain::RuntimeManifest;
use std::path::Path;

const LLAMA_CPP_MANIFEST: &str = include_str!("manifests/llama-cpp.yaml");
const SD_CPP_MANIFEST: &str = include_str!("manifests/sd-cpp.yaml");

/// Retrieve all available runtime manifests, optionally reading additional manifests from an extra folder.
pub fn get_runtime_manifests(extra_folder: Option<&Path>) -> Vec<RuntimeManifest> {
    let mut manifests = Vec::new();

    // 1. Embedded default manifests
    if let Ok(m) = serde_yaml::from_str::<RuntimeManifest>(LLAMA_CPP_MANIFEST) {
        manifests.push(m);
    } else {
        tracing::error!("Failed to parse embedded llama-cpp.yaml manifest");
    }

    if let Ok(m) = serde_yaml::from_str::<RuntimeManifest>(SD_CPP_MANIFEST) {
        manifests.push(m);
    } else {
        tracing::error!("Failed to parse embedded sd-cpp.yaml manifest");
    }

    // 2. Extra manifests folder if specified
    if let Some(folder) = extra_folder {
        if folder.is_dir() {
            if let Ok(entries) = std::fs::read_dir(folder) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() {
                        let is_yaml = path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext == "yaml" || ext == "yml")
                            .unwrap_or(false);

                        if is_yaml {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(m) = serde_yaml::from_str::<RuntimeManifest>(&content) {
                                    // Replace or append
                                    if let Some(pos) = manifests.iter().position(|existing| existing.id == m.id) {
                                        manifests[pos] = m;
                                    } else {
                                        manifests.push(m);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    manifests
}

pub fn get_manifest_for_runtime(extra_folder: Option<&Path>, runtime_id: &str) -> Option<RuntimeManifest> {
    get_runtime_manifests(extra_folder)
        .into_iter()
        .find(|m| m.id.eq_ignore_ascii_case(runtime_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_manifests_parse_correctly() {
        let manifests = get_runtime_manifests(None);
        assert!(manifests.len() >= 2);

        let llama = manifests.iter().find(|m| m.id == "llama-cpp");
        assert!(llama.is_some());
        let llama = llama.unwrap();
        assert_eq!(llama.binary_name, "llama-server");

        let sd = manifests.iter().find(|m| m.id == "sd-cpp");
        assert!(sd.is_some());
    }

    #[test]
    fn test_manifest_target_resolution() {
        let llama = get_manifest_for_runtime(None, "llama-cpp").expect("llama-cpp manifest should exist");
        let target = llama.get_download_target("linux", "x86_64", "cpu");
        assert!(target.is_some());
        let target = target.unwrap();
        assert!(target.url.contains("llama-"));
    }

    #[test]
    fn test_manifest_custom_variables_interpolation() {
        let sd = get_manifest_for_runtime(None, "sd-cpp").expect("sd-cpp manifest should exist");
        assert_eq!(sd.variables.get("commit").map(|s| s.as_str()), Some("a1ded76"));

        let target = sd.get_download_target("linux", "x86_64", "vulkan").expect("vulkan target should exist");
        assert!(target.url.contains("master-945-a1ded76"), "URL should contain version: {}", target.url);
        assert!(target.url.contains("sd-master-a1ded76-bin"), "URL should contain interpolated {{commit}}: {}", target.url);
        assert!(!target.url.contains("{commit}"), "URL must not contain raw {{commit}}: {}", target.url);
    }
}

