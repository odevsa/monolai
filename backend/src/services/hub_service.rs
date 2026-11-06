use crate::core::error::{AppError, AppResult};
use crate::domain::{
    CreateRepositoryPayload, HubModelItem, HubModelReadme, RepositoryRecord,
    UpdateRepositoryPayload,
};
use crate::infrastructure::db::RepositoryRepository;
use crate::services::hub::{HuggingFaceProvider, ModelRepositoryProvider, RepositorySearchQuery};
use std::time::Duration;

#[derive(Clone)]
pub struct HubService {
    repo: RepositoryRepository,
    hf_provider: HuggingFaceProvider,
}

impl HubService {
    pub fn new(repo: RepositoryRepository) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Monolai/0.2.3")
            .build()
            .unwrap_or_default();

        let hf_provider = HuggingFaceProvider::new(http_client);

        Self { repo, hf_provider }
    }

    fn get_provider(&self, repo_type: &str) -> AppResult<&dyn ModelRepositoryProvider> {
        match repo_type {
            "huggingface" => Ok(&self.hf_provider),
            _ => Ok(&self.hf_provider),
        }
    }

    pub async fn get_repositories(&self) -> AppResult<Vec<RepositoryRecord>> {
        self.repo.get_all().await
    }

    pub async fn get_repository_by_id(&self, id: &str) -> AppResult<RepositoryRecord> {
        self.repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found(format!("Repository '{}' not found", id)))
    }

    pub async fn create_repository(
        &self,
        payload: CreateRepositoryPayload,
    ) -> AppResult<RepositoryRecord> {
        let id = payload.id.trim().to_lowercase();
        if id.is_empty() {
            return Err(AppError::bad_request("Repository ID cannot be empty."));
        }
        if payload.name.trim().is_empty() {
            return Err(AppError::bad_request("Repository Name cannot be empty."));
        }
        if payload.base_url.trim().is_empty() {
            return Err(AppError::bad_request(
                "Repository Base URL cannot be empty.",
            ));
        }

        if let Ok(Some(_)) = self.repo.get_by_id(&id).await {
            return Err(AppError::bad_request(format!(
                "Repository with ID '{}' already exists.",
                id
            )));
        }

        let mut config_json = "{}".to_string();
        if let Some(mirror) = payload.mirror_url {
            if !mirror.trim().is_empty() {
                config_json = serde_json::json!({ "mirror_url": mirror.trim() }).to_string();
            }
        }

        let record = RepositoryRecord {
            id: id.clone(),
            name: payload.name.trim().to_string(),
            repo_type: payload.repo_type.trim().to_string(),
            base_url: payload.base_url.trim().trim_end_matches('/').to_string(),
            api_token: payload
                .api_token
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty()),
            enabled: payload.enabled,
            is_builtin: false,
            config_json,
            created_at: String::new(),
        };

        self.repo.insert(&record).await?;
        self.get_repository_by_id(&id).await
    }

    pub async fn update_repository(
        &self,
        id: &str,
        payload: UpdateRepositoryPayload,
    ) -> AppResult<RepositoryRecord> {
        let mut record = self.get_repository_by_id(id).await?;

        if let Some(name) = payload.name {
            if !name.trim().is_empty() {
                record.name = name.trim().to_string();
            }
        }
        if let Some(base_url) = payload.base_url {
            if !base_url.trim().is_empty() {
                record.base_url = base_url.trim().trim_end_matches('/').to_string();
            }
        }
        if let Some(token) = payload.api_token {
            let trimmed = token.trim().to_string();
            record.api_token = if trimmed.is_empty() { None } else { Some(trimmed) };
        }
        if let Some(enabled) = payload.enabled {
            record.enabled = enabled;
        }
        if let Some(mirror) = payload.mirror_url {
            let mut val: serde_json::Value =
                serde_json::from_str(&record.config_json).unwrap_or_else(|_| serde_json::json!({}));
            if mirror.trim().is_empty() {
                if let Some(obj) = val.as_object_mut() {
                    obj.remove("mirror_url");
                }
            } else {
                val["mirror_url"] = serde_json::Value::String(mirror.trim().to_string());
            }
            record.config_json = val.to_string();
        }

        self.repo.update(&record).await?;
        self.get_repository_by_id(id).await
    }

    pub async fn delete_repository(&self, id: &str) -> AppResult<()> {
        self.repo.delete(id).await
    }

    pub async fn search_models(
        &self,
        repo_id: Option<&str>,
        query: Option<&str>,
        modality: Option<&str>,
        sort: Option<&str>,
        page: Option<u32>,
    ) -> AppResult<Vec<HubModelItem>> {
        let target_repo = if let Some(rid) = repo_id {
            self.get_repository_by_id(rid).await?
        } else {
            let enabled = self.repo.get_enabled().await?;
            if let Some(first) = enabled.into_iter().next() {
                first
            } else {
                return Ok(Vec::new());
            }
        };

        if !target_repo.enabled {
            return Err(AppError::bad_request(format!(
                "Repository '{}' is currently disabled in Settings.",
                target_repo.name
            )));
        }

        let provider = self.get_provider(&target_repo.repo_type)?;
        provider
            .search(
                &target_repo,
                RepositorySearchQuery {
                    query,
                    modality,
                    sort,
                    page,
                },
            )
            .await
    }

    pub async fn get_model_details(
        &self,
        repo_id: &str,
        model_id: &str,
    ) -> AppResult<HubModelItem> {
        let repo = self.get_repository_by_id(repo_id).await?;
        let provider = self.get_provider(&repo.repo_type)?;
        provider.get_model_details(&repo, model_id).await
    }

    pub async fn get_model_readme(
        &self,
        repo_id: &str,
        model_id: &str,
    ) -> AppResult<HubModelReadme> {
        let repo = self.get_repository_by_id(repo_id).await?;
        let provider = self.get_provider(&repo.repo_type)?;
        provider.get_model_readme(&repo, model_id).await
    }
}
