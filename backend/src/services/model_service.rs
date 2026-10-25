use crate::core::config::AppConfig;
use crate::core::error::{AppError, AppResult};
use crate::domain::{
    CreatePresetPayload, ModelItem, ModelPreset, ModelRecord, RunningModelStatus,
    RuntimeManifest, UpdatePresetPayload,
};
use crate::infrastructure::db::{ModelRepository, SettingRepository};
use crate::infrastructure::process::manager::{
    load_model_process, swap_model_process, unload_all_model_processes, unload_model_process,
    ProcessManager,
};
use crate::infrastructure::process::manifests::get_runtime_manifests;
use crate::infrastructure::scanner::scan_models;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ModelService {
    repo: ModelRepository,
    setting_repo: SettingRepository,
    process_manager: ProcessManager,
    config: Arc<RwLock<AppConfig>>,
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
            "speech-to-text",
            "embeddings",
            "miscellaneous",
        ];
        for sf in subfolders {
            let _ = tokio::fs::create_dir_all(models_dir.join(sf)).await;
        }
    }

    pub async fn sync_local_models(&self) -> AppResult<()> {
        self.setup_canonical_folders().await;
        let models_dir = self.resolve_models_dir().await;
        let scanned = scan_models(&models_dir);

        for item in scanned {
            let existing = self.repo.find_by_relative_path(&item.relative_path).await?;
            if existing.is_none() {
                // Register newly detected model
                let model_id = item
                    .filename
                    .strip_suffix(".gguf")
                    .or_else(|| item.filename.strip_suffix(".safetensors"))
                    .unwrap_or(&item.filename)
                    .to_string();

                let record = ModelRecord {
                    id: model_id.clone(),
                    name: item.name.clone(),
                    filename: item.filename.clone(),
                    relative_path: item.relative_path.clone(),
                    size_bytes: item.size_bytes as i64,
                    format: item.format.clone(),
                    modality: item.modality.clone(),
                    architecture: item.architecture.clone(),
                    source_repo: None,
                    created_at: String::new(),
                    presets: Vec::new(),
                    file_exists: true,
                };

                // Check if ID is unique; if not, suffix with hash/random
                let unique_id = if self.repo.get_by_id(&model_id).await?.is_some() {
                    format!("{}-{}", model_id, &uuid::Uuid::new_v4().to_string()[..6])
                } else {
                    model_id
                };

                let mut to_insert = record;
                to_insert.id = unique_id.clone();
                let _ = self.repo.insert_model(&to_insert).await;

                // Create default preset with compatible runtime
                let runtime_id = if item.modality == "text-to-image" {
                    "sd-cpp"
                } else {
                    "llama-cpp"
                };

                let default_preset = ModelPreset {
                    id: format!("{}:default", unique_id),
                    model_id: unique_id,
                    name: "Default".to_string(),
                    runtime_id: runtime_id.to_string(),
                    flags: serde_json::json!({ "--model": item.relative_path }).to_string(),
                    is_default: true,
                    created_at: String::new(),
                };

                let _ = self.repo.insert_preset(&default_preset).await;
            }
        }

        Ok(())
    }

    pub async fn get_available_models(&self) -> Vec<ModelItem> {
        let models_dir = self.resolve_models_dir().await;
        scan_models(models_dir)
    }

    pub async fn get_models(&self, include_all: bool) -> AppResult<Vec<ModelRecord>> {
        let _ = self.sync_local_models().await;
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
        let mut model = self
            .repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found(format!("Model '{}' not found", id)))?;

        let models_dir = self.resolve_models_dir().await;
        let full_path = models_dir.join(&model.relative_path);
        model.file_exists = full_path.is_file();

        Ok(model)
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
                let _ = self.repo.set_default_preset(&preset.model_id, &next.id).await;
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
                        && !m.supported_modalities.iter().any(|sm| sm.eq_ignore_ascii_case(target_mod))
                    {
                        return false;
                    }
                }
                if let Some(target_fmt) = format {
                    if !m.supported_formats.is_empty()
                        && !m.supported_formats.iter().any(|sf| sf.eq_ignore_ascii_case(target_fmt))
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
