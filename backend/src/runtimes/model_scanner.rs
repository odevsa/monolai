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

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
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
