use serde::Serialize;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ModelItem {
    pub name: String,
    pub filename: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub format: String,
    pub size_bytes: u64,
}

pub fn scan_models<P: AsRef<Path>>(root_dir: P) -> Vec<ModelItem> {
    let root = root_dir.as_ref();
    if !root.exists() || !root.is_dir() {
        return Vec::new();
    }

    let mut models = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden directories like .git, .cache, etc.
            if e.file_type().is_dir() {
                !e.file_name().to_string_lossy().starts_with('.')
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ext_lower == "gguf" || ext_lower == "safetensors" {
                    let filename = path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let relative_path = path
                        .strip_prefix(root)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| filename.clone());

                    let metadata = entry.metadata().ok();
                    let size_bytes = metadata.map(|m| m.len()).unwrap_or(0);

                    let name = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| filename.clone());

                    models.push(ModelItem {
                        name,
                        filename,
                        relative_path,
                        absolute_path: path.to_string_lossy().to_string(),
                        format: ext_lower,
                        size_bytes,
                    });
                }
            }
        }
    }

    models.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    models
}

/// Check if the model weights file referenced in the flags exists on disk.
/// Performs fast O(1) path checks without scanning disk hierarchies.
/// Returns (exists, optional_file_path_found)
pub fn check_model_file_exists<P: AsRef<Path>>(models_dir: P, flags_json: &str) -> (bool, Option<String>) {
    let models_dir = models_dir.as_ref();

    let flags_map: serde_json::Value = match serde_json::from_str(flags_json) {
        Ok(v) => v,
        Err(_) => return (false, None),
    };

    let obj = match flags_map.as_object() {
        Some(o) => o,
        None => return (false, None),
    };

    // Primary flags where model file is stored
    let primary_keys = ["--model", "-m", "--weights", "-w", "--model-path", "--checkpoint"];
    let mut file_val: Option<String> = None;

    for k in primary_keys {
        if let Some(v) = obj.get(k).and_then(|val| val.as_str()) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                file_val = Some(trimmed.to_string());
                break;
            }
        }
    }

    // Fallback: look for any flag value ending with model extensions
    if file_val.is_none() {
        for (_k, v) in obj {
            if let Some(s) = v.as_str() {
                let trimmed = s.trim();
                let lower = trimmed.to_lowercase();
                if lower.ends_with(".gguf")
                    || lower.ends_with(".safetensors")
                    || lower.ends_with(".bin")
                    || lower.ends_with(".pt")
                    || lower.ends_with(".onnx")
                {
                    file_val = Some(trimmed.to_string());
                    break;
                }
            }
        }
    }

    let val = match file_val {
        Some(v) => v,
        None => return (false, None),
    };

    // 1. Direct path check (handling ~ if present)
    let expanded = crate::config::expand_tilde(&val);
    if expanded.is_file() {
        return (true, Some(val));
    }

    // 2. Relative to models_dir
    let rel_path = models_dir.join(&val);
    if rel_path.is_file() {
        return (true, Some(val));
    }

    // 3. Just the file name relative to models_dir
    if let Some(fname) = Path::new(&val).file_name() {
        let direct_fname = models_dir.join(fname);
        if direct_fname.is_file() {
            return (true, Some(val));
        }
    }

    (false, Some(val))
}
