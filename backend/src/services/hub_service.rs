use crate::core::error::{AppError, AppResult};
use crate::domain::{
    CreateRepositoryPayload, HubModelItem, HubModelReadme, HubModelSibling, RepositoryRecord,
    UpdateRepositoryPayload,
};
use crate::infrastructure::db::RepositoryRepository;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct HubService {
    repo: RepositoryRepository,
    http_client: reqwest::Client,
}

impl HubService {
    pub fn new(repo: RepositoryRepository) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Monolai/0.2.3")
            .build()
            .unwrap_or_default();

        Self { repo, http_client }
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

        if self.repo.get_by_id(&id).await?.is_some() {
            return Err(AppError::conflict(format!(
                "Repository with ID '{}' already exists.",
                id
            )));
        }

        let mut config_val = serde_json::json!({});
        if let Some(mirror) = payload.mirror_url {
            if !mirror.trim().is_empty() {
                config_val["mirror_url"] = serde_json::Value::String(mirror.trim().to_string());
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
            config_json: config_val.to_string(),
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
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                record.name = trimmed.to_string();
            }
        }

        if let Some(base_url) = payload.base_url {
            let trimmed = base_url.trim().trim_end_matches('/');
            if !trimmed.is_empty() {
                record.base_url = trimmed.to_string();
            }
        }

        if let Some(token) = payload.api_token {
            let trimmed = token.trim();
            record.api_token = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            };
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

        match target_repo.repo_type.as_str() {
            "huggingface" => {
                self.search_huggingface(&target_repo, query, modality, sort, page)
                    .await
            }
            _ => {
                self.search_huggingface(&target_repo, query, modality, sort, page)
                    .await
            }
        }
    }

    async fn search_huggingface(
        &self,
        repo: &RepositoryRecord,
        query: Option<&str>,
        modality: Option<&str>,
        sort: Option<&str>,
        page: Option<u32>,
    ) -> AppResult<Vec<HubModelItem>> {
        let base_url = if repo.base_url.is_empty() {
            "https://huggingface.co"
        } else {
            repo.base_url.as_str()
        };

        let url = format!("{}/api/models", base_url);
        let search_text = query.unwrap_or("").trim();
        let sort_mode = sort.unwrap_or("downloads");
        let hf_sort = match sort_mode {
            "likes" => "likes",
            "trending" | "recent" => "lastModified",
            _ => "downloads",
        };

        let page_idx = page.unwrap_or(0);
        let limit: u32 = 30;
        let skip = page_idx * limit;

        let mut query_params: Vec<(&str, String)> = vec![
            ("limit", limit.to_string()),
            ("skip", skip.to_string()),
            ("full", "true".to_string()),
            ("sort", hf_sort.to_string()),
            ("direction", "-1".to_string()),
            ("expand[]", "author".to_string()),
            ("expand[]", "downloads".to_string()),
            ("expand[]", "likes".to_string()),
            ("expand[]", "pipeline_tag".to_string()),
            ("expand[]", "siblings".to_string()),
            ("expand[]", "gated".to_string()),
            ("expand[]", "private".to_string()),
        ];

        if !search_text.is_empty() {
            query_params.push(("search", search_text.to_string()));
        }

        if let Some(m) = modality {
            let filter = match m {
                "text-to-text" => Some("text-generation"),
                "text-to-image" => Some("text-to-image"),
                "text-to-speech" => Some("text-to-speech"),
                "speech-to-text" => Some("automatic-speech-recognition"),
                "embeddings" => Some("feature-extraction"),
                _ => None,
            };
            if let Some(f) = filter {
                query_params.push(("filter", f.to_string()));
            }
        }

        let mut req = self.http_client.get(&url).query(&query_params);
        if let Some(ref token) = repo.api_token {
            if !token.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", token));
            }
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::internal(format!("Failed to reach Hugging Face Hub: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::internal(format!(
                "Hugging Face Hub error ({}): {}",
                status, text
            )));
        }

        let raw_items: Vec<Value> = resp.json().await.unwrap_or_default();
        let has_token = repo
            .api_token
            .as_ref()
            .map(|t| !t.trim().is_empty())
            .unwrap_or(false);
        let mut results = Vec::new();

        for item in raw_items {
            let id = item
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if id.is_empty() {
                continue;
            }

            let is_gated = match item.get("gated") {
                Some(Value::Bool(b)) => *b,
                Some(Value::String(s)) => !s.eq_ignore_ascii_case("false") && !s.is_empty(),
                _ => false,
            } || item
                .get("private")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let requires_auth = is_gated && !has_token;

            let author = item
                .get("author")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
                .or_else(|| {
                    if id.contains('/') {
                        id.split('/')
                            .next()
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                });
            let downloads = item.get("downloads").and_then(|v| v.as_u64());
            let likes = item.get("likes").and_then(|v| v.as_u64());
            let pipeline_tag = item
                .get("pipeline_tag")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let item_modality = match pipeline_tag.as_deref() {
                Some("text-generation") | Some("text2text-generation") => "text-to-text",
                Some("text-to-image") | Some("image-to-image") => "text-to-image",
                Some("text-to-speech") => "text-to-speech",
                Some("automatic-speech-recognition") => "speech-to-text",
                Some("feature-extraction") | Some("sentence-similarity") => "embeddings",
                _ => "text-to-text",
            };

            let effective_base = repo
                .get_mirror_url()
                .unwrap_or_else(|| base_url.to_string());
            let effective_base = effective_base.trim_end_matches('/');

            let mut siblings = Vec::new();
            if let Some(arr) = item.get("siblings").and_then(|v| v.as_array()) {
                for s in arr {
                    if let Some(rfilename) = s.get("rfilename").and_then(|v| v.as_str()) {
                        let lower = rfilename.to_lowercase();
                        if lower.ends_with(".gguf") || lower.ends_with(".safetensors") {
                            let size = s.get("size").and_then(|v| v.as_u64());
                            let download_url =
                                format!("{}/{}/resolve/main/{}", effective_base, id, rfilename);
                            siblings.push(HubModelSibling {
                                rfilename: rfilename.to_string(),
                                size,
                                download_url: Some(download_url),
                            });
                        }
                    }
                }
            }

            results.push(HubModelItem {
                id,
                author,
                downloads,
                likes,
                pipeline_tag,
                modality: item_modality.to_string(),
                siblings,
                is_gated,
                requires_auth,
            });
        }

        Ok(results)
    }

    pub async fn get_model_details(
        &self,
        repo_id: &str,
        model_id: &str,
    ) -> AppResult<HubModelItem> {
        let repo = self.get_repository_by_id(repo_id).await?;
        let base_url = if repo.base_url.is_empty() {
            "https://huggingface.co"
        } else {
            repo.base_url.as_str()
        };

        let url = format!("{}/api/models/{}", base_url, model_id);
        let mut req = self.http_client.get(&url);
        if let Some(ref token) = repo.api_token {
            if !token.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", token));
            }
        }

        let resp = req.send().await.map_err(|e| {
            AppError::internal(format!("Failed to fetch model details from Hub: {}", e))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            return Err(AppError::not_found(format!(
                "Model '{}' not found on Hub (status: {})",
                model_id, status
            )));
        }

        let item: Value = resp.json().await.unwrap_or_default();
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(model_id)
            .to_string();
        let author = item
            .get("author")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let downloads = item.get("downloads").and_then(|v| v.as_u64());
        let likes = item.get("likes").and_then(|v| v.as_u64());
        let pipeline_tag = item
            .get("pipeline_tag")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let modality = match pipeline_tag.as_deref() {
            Some("text-to-image") | Some("image-to-image") => "text-to-image",
            Some("text-to-speech") => "text-to-speech",
            Some("automatic-speech-recognition") => "speech-to-text",
            Some("feature-extraction") => "embeddings",
            _ => "text-to-text",
        };

        let effective_base = repo
            .get_mirror_url()
            .unwrap_or_else(|| base_url.to_string());
        let effective_base = effective_base.trim_end_matches('/');

        // Fetch exact file sizes from /tree/main endpoint for Hugging Face repositories
        let mut size_map = std::collections::HashMap::new();
        if repo.repo_type == "huggingface" {
            let tree_url = format!(
                "{}/api/models/{}/tree/main?recursive=true",
                base_url, model_id
            );
            let mut tree_req = self.http_client.get(&tree_url);
            if let Some(ref token) = repo.api_token {
                if !token.is_empty() {
                    tree_req = tree_req.header("Authorization", format!("Bearer {}", token));
                }
            }
            if let Ok(tree_resp) = tree_req.send().await {
                if tree_resp.status().is_success() {
                    if let Ok(tree_items) = tree_resp.json::<Vec<Value>>().await {
                        for item in tree_items {
                            let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                            if item_type == "file" || item_type.is_empty() {
                                if let (Some(path), Some(size)) = (
                                    item.get("path").and_then(|p| p.as_str()),
                                    item.get("size").and_then(|s| s.as_u64()),
                                ) {
                                    size_map.insert(path.to_string(), size);
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut siblings = Vec::new();
        if let Some(arr) = item.get("siblings").and_then(|v| v.as_array()) {
            for s in arr {
                if let Some(rfilename) = s.get("rfilename").and_then(|v| v.as_str()) {
                    let lower = rfilename.to_lowercase();
                    if lower.ends_with(".gguf") || lower.ends_with(".safetensors") {
                        let size = s
                            .get("size")
                            .and_then(|v| v.as_u64())
                            .or_else(|| size_map.get(rfilename).copied());
                        let download_url =
                            format!("{}/{}/resolve/main/{}", effective_base, id, rfilename);
                        siblings.push(HubModelSibling {
                            rfilename: rfilename.to_string(),
                            size,
                            download_url: Some(download_url),
                        });
                    }
                }
            }
        }

        if siblings.is_empty() && !size_map.is_empty() {
            for (path, size) in size_map {
                let lower = path.to_lowercase();
                if lower.ends_with(".gguf") || lower.ends_with(".safetensors") {
                    let download_url = format!("{}/{}/resolve/main/{}", effective_base, id, path);
                    siblings.push(HubModelSibling {
                        rfilename: path,
                        size: Some(size),
                        download_url: Some(download_url),
                    });
                }
            }
            siblings.sort_by(|a, b| a.rfilename.cmp(&b.rfilename));
        }

        let has_token = repo
            .api_token
            .as_ref()
            .map(|t| !t.trim().is_empty())
            .unwrap_or(false);
        let is_gated = match item.get("gated") {
            Some(Value::Bool(b)) => *b,
            Some(Value::String(s)) => !s.eq_ignore_ascii_case("false") && !s.is_empty(),
            _ => false,
        } || item
            .get("private")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let requires_auth = is_gated && !has_token;

        Ok(HubModelItem {
            id,
            author,
            downloads,
            likes,
            pipeline_tag,
            modality: modality.to_string(),
            siblings,
            is_gated,
            requires_auth,
        })
    }

    pub async fn get_model_readme(
        &self,
        repo_id: &str,
        model_id: &str,
    ) -> AppResult<HubModelReadme> {
        let repo = self.get_repository_by_id(repo_id).await?;
        let base_url = if repo.base_url.is_empty() {
            "https://huggingface.co"
        } else {
            repo.base_url.as_str()
        };

        let effective_base = repo
            .get_mirror_url()
            .unwrap_or_else(|| base_url.to_string());
        let effective_base = effective_base.trim_end_matches('/');

        let branches = ["main", "master"];
        let filenames = ["README.md", "readme.md"];

        for branch in &branches {
            for filename in &filenames {
                let url = format!("{}/{}/raw/{}/{}", effective_base, model_id, branch, filename);
                let mut req = self.http_client.get(&url);
                if let Some(ref token) = repo.api_token {
                    if !token.is_empty() {
                        req = req.header("Authorization", format!("Bearer {}", token));
                    }
                }

                if let Ok(resp) = req.send().await {
                    if resp.status().is_success() {
                        let content = resp.text().await.unwrap_or_default();
                        return Ok(HubModelReadme { content });
                    }
                }
            }
        }

        Err(AppError::not_found(format!(
            "README.md not found for model '{}'",
            model_id
        )))
    }
}
