use crate::core::config::AppConfig;
use crate::core::error::{AppError, AppResult};
use crate::domain::{
    CreatePresetPayload, ModelItem, ModelPreset, ModelRecord, ModelVoicesResponse,
    RunningModelStatus, RuntimeManifest, UpdatePresetPayload,
};
use crate::infrastructure::db::{ModelRepository, SettingRepository};
use crate::infrastructure::process::manager::{
    load_model_process, swap_model_process, unload_all_model_processes, unload_model_process,
    ProcessManager,
};
use crate::infrastructure::process::manifests::get_runtime_manifests;
use crate::infrastructure::scanner::{discover_model_files, scan_models};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};

#[derive(Clone)]
pub struct ModelService {
    repo: ModelRepository,
    setting_repo: SettingRepository,
    process_manager: ProcessManager,
    config: Arc<RwLock<AppConfig>>,
    last_synced: Arc<RwLock<Option<Instant>>>,
    sync_lock: Arc<Mutex<()>>,
}

impl ModelService {
    pub fn new(
        repo: ModelRepository,
        setting_repo: SettingRepository,
        process_manager: ProcessManager,
        config: Arc<RwLock<AppConfig>>,
    ) -> Self {
        Self {
            repo,
            setting_repo,
            process_manager,
            config,
            last_synced: Arc::new(RwLock::new(None)),
            sync_lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn resolve_models_dir(&self) -> PathBuf {
        let cfg = self.config.read().await;
        if let Some(ref dir) = cfg.models {
            crate::core::config::expand_tilde(dir)
        } else {
            crate::core::config::get_default_models_dir()
        }
    }

    pub async fn setup_canonical_folders(&self) {
        let models_dir = self.resolve_models_dir().await;
        let subfolders = [
            "text-to-text",
            "text-to-image",
            "text-to-speech",
            "automatic-speech-recognition",
            "embeddings",
            "miscellaneous",
        ];
        for sf in subfolders {
            let _ = tokio::fs::create_dir_all(models_dir.join(sf)).await;
        }
    }

    pub async fn invalidate_cache(&self) {
        let mut ls = self.last_synced.write().await;
        *ls = None;
    }

    pub async fn sync_local_models(&self) -> AppResult<()> {
        let _guard = self.sync_lock.lock().await;
        self.setup_canonical_folders().await;
        let models_dir = self.resolve_models_dir().await;

        let dir_clone = models_dir.clone();
        // 1. Discover all candidate model files by directory walking (0 files opened, instant filesystem metadata only)
        let discovered_files = tokio::task::spawn_blocking(move || discover_model_files(&dir_clone))
            .await
            .map_err(|e| AppError::internal(format!("Model discovery task failed: {}", e)))?;

        // 2. Fetch all known models from SQLite once
        let all_db_models = self.repo.get_all().await?;
        let existing_map: HashMap<String, ModelRecord> = all_db_models
            .into_iter()
            .map(|m| (m.relative_path.clone(), m))
            .collect();

        // 3. Process each discovered file
        for file in discovered_files {
            let norm_path = file.relative_path.replace('\\', "/");
            let parts: Vec<&str> = norm_path.split('/').collect();

            // Canonical structure: {type}/{user}/{model}:{filename}
            let (canonical_id, display_name, source_repo) = if parts.len() >= 4 {
                let user = parts[1];
                let repo = parts[2];
                let repo_slug = format!("{}/{}", user, repo);
                let id = format!("{}/{}:{}", user, repo, file.filename);
                (id, repo_slug.clone(), Some(repo_slug))
            } else if parts.len() == 3 {
                let folder = parts[1];
                let id = format!("{}:{}", folder, file.filename);
                (id, folder.to_string(), Some(folder.to_string()))
            } else {
                let stem = Path::new(&file.filename)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| file.filename.clone());
                (file.filename.clone(), stem, None)
            };

            if let Some(existing_record) = existing_map.get(&file.relative_path) {
                // The file ALREADY exists in the database!
                // We do NOT open or inspect the file header, avoiding disk I/O completely.
                let current_id = if existing_record.id != canonical_id {
                    let _ = self
                        .repo
                        .update_model_id(
                            &existing_record.id,
                            &canonical_id,
                            &display_name,
                            source_repo.as_deref(),
                        )
                        .await;
                    canonical_id.clone()
                } else {
                    existing_record.id.clone()
                };

                // One-time self-healing/migration check: only if model was misclassified as text-to-text
                // but resides in an audio or image folder or has an audio architecture
                let needs_migration = existing_record.modality == "text-to-text"
                    && (norm_path.starts_with("text-to-speech/")
                        || norm_path.starts_with("automatic-speech-recognition/")
                        || norm_path.starts_with("text-to-image/")
                        || norm_path.starts_with("embeddings/")
                        || existing_record.architecture.as_deref() == Some("kokoro_tts"));

                if needs_migration {
                    let full_path = models_dir.join(&file.relative_path);
                    let inspect = crate::infrastructure::scanner::gguf::inspect_model_file(&full_path);
                    if inspect.modality != existing_record.modality
                        || inspect.architecture != existing_record.architecture
                    {
                        let _ = self
                            .repo
                            .update_model_metadata(
                                &current_id,
                                &inspect.modality,
                                inspect.architecture.as_deref(),
                                inspect.is_auxiliary,
                                inspect.auxiliary_type.as_deref(),
                            )
                            .await;

                        let target_runtime = crate::infrastructure::process::find_default_runtime_for_modality(
                            None,
                            &inspect.modality,
                            &file.format,
                        );

                        if let Ok(presets) = self.repo.get_presets_for_model(&current_id).await {
                            for mut p in presets {
                                if p.is_default && p.runtime_id != target_runtime {
                                    p.runtime_id = target_runtime.clone();
                                    p.flags = crate::infrastructure::process::build_default_preset_flags(
                                        None,
                                        &target_runtime,
                                        &file.relative_path,
                                    );
                                    let _ = self.repo.update_preset(&p).await;
                                }
                            }
                        }
                    }
                }
            } else {
                // BRAND NEW MODEL added manually into models directory!
                // Inspect header once to register and build presets.
                let full_path = models_dir.join(&file.relative_path);
                let inspect = crate::infrastructure::scanner::gguf::inspect_model_file(&full_path);
                let name = inspect.model_name.unwrap_or_else(|| {
                    Path::new(&file.filename)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| display_name.clone())
                });

                let record = ModelRecord {
                    id: canonical_id.clone(),
                    name,
                    filename: file.filename.clone(),
                    relative_path: file.relative_path.clone(),
                    size_bytes: file.size_bytes as i64,
                    format: file.format.clone(),
                    modality: inspect.modality.clone(),
                    architecture: inspect.architecture.clone(),
                    source_repo,
                    is_auxiliary: inspect.is_auxiliary,
                    auxiliary_type: inspect.auxiliary_type.clone(),
                    created_at: String::new(),
                    presets: Vec::new(),
                    file_exists: true,
                };

                let _ = self.repo.insert_model(&record).await;

                if !inspect.is_auxiliary {
                    let runtime_id = crate::infrastructure::process::find_default_runtime_for_modality(
                        None,
                        &inspect.modality,
                        &file.format,
                    );

                    let default_preset = ModelPreset {
                        id: format!("{}:default", canonical_id),
                        model_id: canonical_id,
                        name: "Default".to_string(),
                        runtime_id: runtime_id.clone(),
                        flags: crate::infrastructure::process::build_default_preset_flags(
                            None,
                            &runtime_id,
                            &file.relative_path,
                        ),
                        is_default: true,
                        created_at: String::new(),
                    };

                    let _ = self.repo.insert_preset(&default_preset).await;
                }
            }
        }

        let mut ls = self.last_synced.write().await;
        *ls = Some(Instant::now());

        Ok(())
    }

    pub async fn get_available_models(&self) -> Vec<ModelItem> {
        let models_dir = self.resolve_models_dir().await;
        tokio::task::spawn_blocking(move || scan_models(models_dir))
            .await
            .unwrap_or_default()
    }

    pub async fn get_models(
        &self,
        include_all: bool,
        force_sync: bool,
    ) -> AppResult<Vec<ModelRecord>> {
        let should_sync = {
            let ls = self.last_synced.read().await;
            match *ls {
                Some(_) => force_sync,
                None => true,
            }
        };

        if should_sync {
            let _ = self.sync_local_models().await;
        }

        let mut models = self.repo.get_all().await?;
        let models_dir = self.resolve_models_dir().await;

        for m in &mut models {
            let full_path = models_dir.join(&m.relative_path);
            m.file_exists = full_path.is_file();
        }

        if !include_all {
            models.retain(|m| m.file_exists);
        }

        Ok(models)
    }

    pub async fn get_model_by_id(&self, id: &str) -> AppResult<ModelRecord> {
        let mut model_opt = self.repo.get_by_id(id).await?;
        if model_opt.is_none() && id.contains(':') {
            if let Some(base_id) = id.split(':').next() {
                model_opt = self.repo.get_by_id(base_id).await?;
            }
        }

        let mut model = model_opt
            .ok_or_else(|| AppError::not_found(format!("Model '{}' not found", id)))?;

        let models_dir = self.resolve_models_dir().await;
        let full_path = models_dir.join(&model.relative_path);
        model.file_exists = full_path.is_file();

        Ok(model)
    }

    pub async fn get_model_voices(&self, model_id: &str) -> AppResult<ModelVoicesResponse> {
        let model = self.get_model_by_id(model_id).await?;
        let models_dir = self.resolve_models_dir().await;
        let full_path = models_dir.join(&model.relative_path);

        let family = model.architecture.clone().unwrap_or_else(|| {
            if model.filename.to_lowercase().contains("kokoro") {
                "kokoro_tts".to_string()
            } else {
                "unknown".to_string()
            }
        });

        // 1. Try extracting embedded voices from GGUF file
        let mut voices = if full_path.is_file() && model.format.to_lowercase() == "gguf" {
            tokio::task::spawn_blocking(move || {
                crate::infrastructure::scanner::gguf::extract_gguf_voices(&full_path)
            })
            .await
            .unwrap_or(None)
            .unwrap_or_default()
        } else {
            Vec::new()
        };

        // 2. Multi-tier fallback if no embedded voices found
        if voices.is_empty() {
            let is_kokoro = family.to_lowercase().contains("kokoro")
                || model.filename.to_lowercase().contains("kokoro")
                || model.name.to_lowercase().contains("kokoro");

            if is_kokoro {
                voices = crate::infrastructure::scanner::gguf::get_canonical_kokoro_voices();
            } else if model.modality == "text-to-speech" {
                voices = crate::infrastructure::scanner::gguf::get_openai_standard_voices();
            }
        }

        Ok(ModelVoicesResponse {
            model_id: model.id,
            family,
            voices,
        })
    }

    pub async fn delete_model(&self, id: &str, delete_file: bool) -> AppResult<()> {
        let model = self.get_model_by_id(id).await?;
        let _ = self.unload_model(id).await;

        if delete_file {
            let models_dir = self.resolve_models_dir().await;
            let full_path = models_dir.join(&model.relative_path);
            let _ = tokio::fs::remove_file(&full_path).await;
            let part_path = PathBuf::from(format!("{}.part", full_path.display()));
            let _ = tokio::fs::remove_file(part_path).await;
            tracing::info!(target: "monolai", "Model file '{}' deleted from disk", full_path.display());
        }

        self.repo.delete_model(id).await?;
        self.invalidate_cache().await;
        tracing::info!(target: "monolai", "Model '{}' removed from repository", id);
        Ok(())
    }

    // Presets Management
    pub async fn get_presets(&self, model_id: &str) -> AppResult<Vec<ModelPreset>> {
        self.repo.get_presets_for_model(model_id).await
    }

    pub async fn create_preset(
        &self,
        model_id: &str,
        payload: CreatePresetPayload,
    ) -> AppResult<ModelPreset> {
        let _model = self.get_model_by_id(model_id).await?;

        let name = payload.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::bad_request("Preset name cannot be empty."));
        }

        let runtime_id = payload.runtime_id.trim().to_string();
        if runtime_id.is_empty() {
            return Err(AppError::bad_request("Runtime engine cannot be empty."));
        }

        let slug = name.to_lowercase().replace(' ', "-");
        let preset_id = format!("{}:{}", model_id, slug);

        let flags_str = crate::infrastructure::db::normalize_flags(&payload.flags);

        let preset = ModelPreset {
            id: preset_id,
            model_id: model_id.to_string(),
            name,
            runtime_id,
            flags: flags_str,
            is_default: payload.is_default,
            created_at: String::new(),
        };

        self.repo.insert_preset(&preset).await?;
        self.repo
            .get_preset_by_id(&preset.id)
            .await?
            .ok_or_else(|| AppError::internal("Failed to retrieve created preset"))
    }

    pub async fn update_preset(
        &self,
        preset_id: &str,
        payload: UpdatePresetPayload,
    ) -> AppResult<ModelPreset> {
        let mut preset = self
            .repo
            .get_preset_by_id(preset_id)
            .await?
            .ok_or_else(|| AppError::not_found(format!("Preset '{}' not found", preset_id)))?;

        if let Some(name) = payload.name {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                preset.name = trimmed.to_string();
            }
        }

        if let Some(runtime_id) = payload.runtime_id {
            let trimmed = runtime_id.trim();
            if !trimmed.is_empty() {
                preset.runtime_id = trimmed.to_string();
            }
        }

        if let Some(flags) = payload.flags {
            preset.flags = crate::infrastructure::db::normalize_flags(&flags);
        }

        if let Some(is_default) = payload.is_default {
            preset.is_default = is_default;
        }

        self.repo.update_preset(&preset).await?;
        self.repo
            .get_preset_by_id(preset_id)
            .await?
            .ok_or_else(|| AppError::internal("Failed to retrieve updated preset"))
    }

    pub async fn delete_preset(&self, preset_id: &str) -> AppResult<()> {
        let preset = self
            .repo
            .get_preset_by_id(preset_id)
            .await?
            .ok_or_else(|| AppError::not_found(format!("Preset '{}' not found", preset_id)))?;

        let all_presets = self.repo.get_presets_for_model(&preset.model_id).await?;
        if all_presets.len() <= 1 {
            return Err(AppError::bad_request(
                "Cannot delete the last remaining preset for a model.",
            ));
        }

        self.repo.delete_preset(preset_id).await?;

        // If the deleted preset was default, promote the first remaining preset to default
        if preset.is_default {
            if let Some(next) = all_presets.into_iter().find(|p| p.id != preset_id) {
                let _ = self
                    .repo
                    .set_default_preset(&preset.model_id, &next.id)
                    .await;
            }
        }

        Ok(())
    }

    pub async fn set_default_preset(&self, model_id: &str, preset_id: &str) -> AppResult<()> {
        self.repo.set_default_preset(model_id, preset_id).await
    }

    pub async fn get_compatible_runtimes(
        &self,
        modality: Option<&str>,
        format: Option<&str>,
    ) -> Vec<RuntimeManifest> {
        let manifests = get_runtime_manifests(None);
        manifests
            .into_iter()
            .filter(|m| {
                if let Some(target_mod) = modality {
                    if !m.supported_modalities.is_empty()
                        && !m
                            .supported_modalities
                            .iter()
                            .any(|sm| sm.eq_ignore_ascii_case(target_mod))
                    {
                        return false;
                    }
                }
                if let Some(target_fmt) = format {
                    if !m.supported_formats.is_empty()
                        && !m
                            .supported_formats
                            .iter()
                            .any(|sf| sf.eq_ignore_ascii_case(target_fmt))
                    {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    pub async fn load_model(&self, id: &str) -> AppResult<RunningModelStatus> {
        tracing::info!(target: "monolai", "Loading model/preset '{}' into memory...", id);
        let cfg = self.config.read().await.clone();
        let status = load_model_process(
            &self.process_manager,
            &self.repo,
            &self.setting_repo,
            &cfg,
            id,
        )
        .await?;
        tracing::info!(
            target: "monolai",
            "Model '{}' loaded successfully and ready (port: {})",
            id,
            status.port
        );
        Ok(status)
    }

    pub async fn swap_model(&self, id: &str) -> AppResult<RunningModelStatus> {
        tracing::info!(target: "monolai", "Swapping active model to '{}'...", id);
        let cfg = self.config.read().await.clone();
        let status = swap_model_process(
            &self.process_manager,
            &self.repo,
            &self.setting_repo,
            &cfg,
            id,
        )
        .await?;
        tracing::info!(
            target: "monolai",
            "Model swapped successfully to '{}' (port: {})",
            id,
            status.port
        );
        Ok(status)
    }

    pub async fn unload_model(&self, id: &str) -> AppResult<()> {
        tracing::info!(target: "monolai", "Unloading model '{}'...", id);
        unload_model_process(&self.process_manager, &self.setting_repo, id).await?;
        tracing::info!(target: "monolai", "Model '{}' unloaded successfully", id);
        Ok(())
    }

    pub async fn unload_all_models(&self) -> AppResult<()> {
        tracing::info!(target: "monolai", "Unloading all active models from memory...");
        unload_all_model_processes(&self.process_manager, &self.setting_repo).await?;
        tracing::info!(target: "monolai", "All active models unloaded successfully");
        Ok(())
    }

    pub async fn get_running_models(&self) -> Vec<RunningModelStatus> {
        self.process_manager.get_all_running_status().await
    }

    pub fn subscribe_state(&self) -> tokio::sync::broadcast::Receiver<Vec<RunningModelStatus>> {
        self.process_manager.subscribe_state()
    }
}
