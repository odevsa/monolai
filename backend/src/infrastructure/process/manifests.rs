use crate::domain::RuntimeManifest;
use std::path::Path;

const LLAMA_CPP_MANIFEST: &str = include_str!("manifests/llama-cpp.yaml");
const SD_CPP_MANIFEST: &str = include_str!("manifests/sd-cpp.yaml");
const AUDIO_CPP_MANIFEST: &str = include_str!("manifests/audio-cpp.yaml");

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

    if let Ok(m) = serde_yaml::from_str::<RuntimeManifest>(AUDIO_CPP_MANIFEST) {
        manifests.push(m);
    } else {
        tracing::error!("Failed to parse embedded audio-cpp.yaml manifest");
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
                                    if let Some(pos) =
                                        manifests.iter().position(|existing| existing.id == m.id)
                                    {
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

pub fn get_manifest_for_runtime(
    extra_folder: Option<&Path>,
    runtime_id: &str,
) -> Option<RuntimeManifest> {
    get_runtime_manifests(extra_folder)
        .into_iter()
        .find(|m| m.id.eq_ignore_ascii_case(runtime_id))
}

/// Find the best default runtime ID that supports the given modality and model format,
/// querying runtime manifests dynamically (SSoT).
pub fn find_default_runtime_for_modality(
    extra_folder: Option<&Path>,
    modality: &str,
    format: &str,
) -> String {
    let manifests = get_runtime_manifests(extra_folder);
    for m in &manifests {
        let supports_modality = m
            .supported_modalities
            .iter()
            .any(|sm| sm.eq_ignore_ascii_case(modality));
        let supports_format = m.supported_formats.is_empty()
            || m.supported_formats
                .iter()
                .any(|sf| sf.eq_ignore_ascii_case(format));

        if supports_modality && supports_format {
            return m.id.clone();
        }
    }

    "llama-cpp".to_string()
}

/// Generate default preset flags from a runtime manifest, prioritizing important flags
/// and mapping the primary model path dynamically to the manifest flag marked with `model: true`.
pub fn build_preset_flags_from_manifest(
    manifest: &RuntimeManifest,
    model_relative_path: &str,
) -> serde_json::Value {
    let mut flags_map = serde_json::Map::new();

    let model_flag = manifest.find_model_flag();
    let model_flag_name = model_flag.map(|f| f.flag.as_str()).unwrap_or("--model");

    flags_map.insert(
        model_flag_name.to_string(),
        serde_json::Value::String(model_relative_path.to_string()),
    );

    for flag in &manifest.flags {
        if flag.important && flag.flag != model_flag_name {
            flags_map.insert(
                flag.flag.clone(),
                serde_json::Value::String(flag.default_value.clone()),
            );
        }
    }

    serde_json::Value::Object(flags_map)
}

/// Helper to build default preset flags JSON string directly for a given runtime ID.
pub fn build_default_preset_flags(
    extra_folder: Option<&Path>,
    runtime_id: &str,
    model_relative_path: &str,
) -> String {
    if let Some(manifest) = get_manifest_for_runtime(extra_folder, runtime_id) {
        build_preset_flags_from_manifest(&manifest, model_relative_path).to_string()
    } else {
        serde_json::json!({ "--model": model_relative_path }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_default_preset_flags_from_manifests() {
        let llama_flags = build_default_preset_flags(None, "llama-cpp", "models/llama-3.gguf");
        let parsed_llama: serde_json::Value = serde_json::from_str(&llama_flags).unwrap();
        assert_eq!(parsed_llama["--model"], "models/llama-3.gguf");
        assert_eq!(parsed_llama["--ctx-size"], "4096");
        assert_eq!(parsed_llama["--n-gpu-layers"], "99");

        let sd_flags = build_default_preset_flags(None, "sd-cpp", "models/sd.safetensors");
        let parsed_sd: serde_json::Value = serde_json::from_str(&sd_flags).unwrap();
        assert_eq!(parsed_sd["--model"], "models/sd.safetensors");
        assert!(parsed_sd.get("--ctx-size").is_none());
    }

    #[test]
    fn test_embedded_manifests_parse_correctly() {
        let manifests = get_runtime_manifests(None);
        assert!(manifests.len() >= 3);

        let llama = manifests.iter().find(|m| m.id == "llama-cpp");
        assert!(llama.is_some());
        let llama = llama.unwrap();
        assert_eq!(llama.binary_name, "llama-server");

        let sd = manifests.iter().find(|m| m.id == "sd-cpp");
        assert!(sd.is_some());
        let sd = sd.unwrap();
        assert_eq!(sd.binary_name, "sd-server");

        let audio = manifests.iter().find(|m| m.id == "audio-cpp");
        assert!(audio.is_some());
        let audio = audio.unwrap();
        assert_eq!(audio.binary_name, "audiocpp_server");
    }

    #[test]
    fn test_manifest_target_resolution() {
        let mut llama =
            get_manifest_for_runtime(None, "llama-cpp").expect("llama-cpp manifest should exist");
        llama.version = "b12023".to_string();
        let target = llama.get_download_target("linux", "x86_64", "cpu");
        assert!(target.is_some());
        let target = target.unwrap();
        assert!(target.url.contains("llama-b12023"));
    }

    #[test]
    fn test_manifest_custom_variables_interpolation() {
        use crate::domain::runtimes::{UpstreamConfig, UpstreamRuntimeEntry};
        use std::collections::HashMap;

        let mut sd = get_manifest_for_runtime(None, "sd-cpp").expect("sd-cpp manifest should exist");
        assert!(matches!(sd.upstream, Some(UpstreamConfig::GitHubRelease { .. })));

        let mut vars = HashMap::new();
        vars.insert("version".to_string(), "master-1031-c6dfa9d".to_string());
        vars.insert("commit".to_string(), "c6dfa9d".to_string());

        sd.apply_upstream(&UpstreamRuntimeEntry {
            version: "master-1031-c6dfa9d".to_string(),
            variables: vars,
            assets: Vec::new(),
            etag: None,
            last_checked_at: 0,
            release_notes_url: None,
        });

        assert_eq!(
            sd.variables.get("commit").map(|s| s.as_str()),
            Some("c6dfa9d")
        );

        let target = sd
            .get_download_target("linux", "x86_64", "vulkan")
            .expect("vulkan target should exist");
        assert!(
            target.url.contains("master-1031-c6dfa9d"),
            "URL should contain version: {}",
            target.url
        );
        assert!(
            target.url.contains("sd-master-c6dfa9d-bin"),
            "URL should contain interpolated {{commit}}: {}",
            target.url
        );
        assert!(
            !target.url.contains("{commit}"),
            "URL must not contain raw {{commit}}: {}",
            target.url
        );
    }
}
