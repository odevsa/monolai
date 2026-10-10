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

        let runtime = payload.runtime.trim().to_string();
        if runtime.is_empty() {
            return Err(AppError::bad_request("Runtime engine cannot be empty."));
        }
        payload.runtime = runtime;

        let mut model = self.repo.update(id, &payload).await?;
        let models_dir = self.resolve_models_dir().await;
        let (exists, _) = check_model_file_exists(&models_dir, &model.flags);
        model.file_exists = exists;

        Ok(model)
    }

    pub async fn delete_model(&self, id: &str) -> AppResult<()> {
        self.repo.delete(id).await
    }

    pub async fn load_model(&self, id: &str) -> AppResult<RunningModelStatus> {
        let cfg = self.config.read().await.clone();
        load_model_process(
            &self.process_manager,
            &self.repo,
            &self.setting_repo,
            &cfg,
            id,
        )
        .await
    }

    pub async fn swap_model(&self, id: &str) -> AppResult<RunningModelStatus> {
        let cfg = self.config.read().await.clone();
        swap_model_process(
            &self.process_manager,
            &self.repo,
            &self.setting_repo,
            &cfg,
            id,
        )
        .await
    }

    pub async fn unload_model(&self, id: &str) -> AppResult<()> {
        unload_model_process(&self.process_manager, &self.setting_repo, id).await
    }

    pub async fn unload_all_models(&self) -> AppResult<()> {
        unload_all_model_processes(&self.process_manager, &self.setting_repo).await
    }

    pub async fn get_running_models(&self) -> Vec<RunningModelStatus> {
        self.process_manager.get_all_running_status().await
    }

    pub fn subscribe_state(&self) -> tokio::sync::broadcast::Receiver<Vec<RunningModelStatus>> {
        self.process_manager.subscribe_state()
    }
}
