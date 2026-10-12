use crate::core::config::AppConfig;
use crate::core::error::{AppError, AppResult};
use crate::domain::{CreateModelPayload, ModelItem, ModelRecord, RunningModelStatus};
use crate::infrastructure::db::{ModelRepository, SettingRepository};
use crate::infrastructure::process::manager::{
    load_model_process, swap_model_process, unload_all_model_processes, unload_model_process,
    ProcessManager,
};
use crate::infrastructure::scanner::{check_model_file_exists, scan_models};
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

    async fn resolve_models_dir(&self) -> PathBuf {
        let cfg = self.config.read().await;
        if let Some(ref dir) = cfg.models {
            crate::core::config::expand_tilde(dir)
        } else {
            crate::core::config::get_default_models_dir()
        }
    }

    pub async fn get_available_models(&self) -> Vec<ModelItem> {
        let models_dir = self.resolve_models_dir().await;
        scan_models(models_dir)
    }

    pub async fn get_models(&self, include_all: bool) -> AppResult<Vec<ModelRecord>> {
        let mut models = self.repo.get_all().await?;
        let models_dir = self.resolve_models_dir().await;

        for m in &mut models {
            let (exists, _) = check_model_file_exists(&models_dir, &m.flags);
            m.file_exists = exists;
        }

        if !include_all {
            models.retain(|m| m.file_exists);
        }

        Ok(models)
    }

    pub async fn create_model(&self, mut payload: CreateModelPayload) -> AppResult<ModelRecord> {
        let id = payload.id.trim().to_string();

        if id.is_empty() {
            return Err(AppError::bad_request("Model ID cannot be empty."));
        }

        if id.contains(' ') || id.contains('\t') || id.contains('\n') {
            return Err(AppError::bad_request("Model ID cannot contain whitespace."));
        }

        if self.repo.get_by_id(&id).await?.is_some() {
            return Err(AppError::conflict(format!(
                "A model with ID '{}' already exists.",
                id
            )));
        }

        let runtime = payload.runtime.trim().to_string();
        if runtime.is_empty() {
            return Err(AppError::bad_request("Runtime engine cannot be empty."));
        }

        payload.id = id;
        payload.runtime = runtime;

        let mut model = self.repo.insert(&payload).await?;
        let models_dir = self.resolve_models_dir().await;
        let (exists, _) = check_model_file_exists(&models_dir, &model.flags);
        model.file_exists = exists;

        tracing::info!(
            target: "monolai",
            "Model '{}' registered successfully with runtime '{}'",
            model.id,
            model.runtime
        );

        Ok(model)
    }

    pub async fn update_model(
        &self,
        id: &str,
        mut payload: CreateModelPayload,
    ) -> AppResult<ModelRecord> {
        if self.repo.get_by_id(id).await?.is_none() {
            return Err(AppError::not_found(format!(
                "Model with ID '{}' not found.",
                id
            )));
        }

        let target_id = payload.id.trim();
        let target_id = if target_id.is_empty() {
            id.to_string()
        } else {
            if target_id.contains(' ') || target_id.contains('\t') || target_id.contains('\n') {
                return Err(AppError::bad_request("Model ID cannot contain whitespace."));
            }
            if target_id != id && self.repo.get_by_id(target_id).await?.is_some() {
                return Err(AppError::conflict(format!(
                    "A model with ID '{}' already exists.",
                    target_id
                )));
            }
            target_id.to_string()
        };
        payload.id = target_id.clone();

        let runtime = payload.runtime.trim().to_string();
        let runtime = if runtime.is_empty() {
            if let Some(existing) = self.repo.get_by_id(id).await? {
                existing.runtime
            } else {
                return Err(AppError::bad_request("Runtime engine cannot be empty."));
            }
        } else {
            runtime
        };
        payload.runtime = runtime;

        let mut model = self.repo.update(id, &payload).await?;
        let models_dir = self.resolve_models_dir().await;
        let (exists, _) = check_model_file_exists(&models_dir, &model.flags);
        model.file_exists = exists;

        if target_id != id {
            let mut running_map = self.process_manager.running.lock().await;
            if let Some(mut rm) = running_map.remove(id) {
                rm.model_id = target_id.clone();
                running_map.insert(target_id.clone(), rm);
            }
            drop(running_map);
            let mut disk_records =
                crate::infrastructure::process::read_processes_json(&self.process_manager.db_dir);
            if let Some(mut rec) = disk_records.remove(id) {
                rec.model_id = target_id.clone();
                disk_records.insert(target_id.clone(), rec);
                crate::infrastructure::process::write_processes_json(
                    &self.process_manager.db_dir,
                    &disk_records,
                );
            }
            self.process_manager.notify_state_changed().await;
        }

        tracing::info!(
            target: "monolai",
            "Model '{}' (updated ID '{}') configuration updated successfully",
            id,
            model.id
        );

        Ok(model)
    }

    pub async fn delete_model(&self, id: &str) -> AppResult<()> {
        self.repo.delete(id).await?;
        tracing::info!(
            target: "monolai",
            "Model '{}' deleted from registry",
            id
        );
        Ok(())
    }

    pub async fn load_model(&self, id: &str) -> AppResult<RunningModelStatus> {
        tracing::info!(
            target: "monolai",
            "Loading model '{}' into memory...",
            id
        );
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
        tracing::info!(
            target: "monolai",
            "Swapping active model to '{}'...",
            id
        );
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
        tracing::info!(
            target: "monolai",
            "Unloading model '{}'...",
            id
        );
        unload_model_process(&self.process_manager, &self.setting_repo, id).await?;
        tracing::info!(
            target: "monolai",
            "Model '{}' unloaded successfully",
            id
        );
        Ok(())
    }

    pub async fn unload_all_models(&self) -> AppResult<()> {
        tracing::info!(
            target: "monolai",
            "Unloading all active models from memory..."
        );
        unload_all_model_processes(&self.process_manager, &self.setting_repo).await?;
        tracing::info!(
            target: "monolai",
            "All active models unloaded successfully"
        );
        Ok(())
    }

    pub async fn get_running_models(&self) -> Vec<RunningModelStatus> {
        self.process_manager.get_all_running_status().await
    }

    pub fn subscribe_state(&self) -> tokio::sync::broadcast::Receiver<Vec<RunningModelStatus>> {
        self.process_manager.subscribe_state()
    }
}
