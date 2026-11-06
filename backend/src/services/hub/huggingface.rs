use super::provider::{ModelRepositoryProvider, RepositorySearchQuery};
use crate::core::error::{AppError, AppResult};
use crate::domain::{
    ClassificationContext, HubModelItem, HubModelReadme, HubModelSibling, ModalityClassifier,
    RepositoryRecord,
};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
pub struct HuggingFaceProvider {
    http_client: Client,
}

impl HuggingFaceProvider {
    pub fn new(http_client: Client) -> Self {
        Self { http_client }
    }
}

impl ModelRepositoryProvider for HuggingFaceProvider {
    fn repo_type(&self) -> &'static str {
        "huggingface"
    }

    fn search<'a>(
        &'a self,
        repo: &'a RepositoryRecord,
        query: RepositorySearchQuery<'a>,
    ) -> Pin<Box<dyn Future<Output = AppResult<Vec<HubModelItem>>> + Send + 'a>> {
        Box::pin(async move {
            let base_url = if repo.base_url.is_empty() {
                "https://huggingface.co"
            } else {
                repo.base_url.as_str()
            };

            let url = format!("{}/api/models", base_url);
            let search_text = query.query.unwrap_or("").trim();
            let sort_mode = query.sort.unwrap_or("downloads");
            let hf_sort = match sort_mode {
                "likes" => "likes",
                "trending" | "recent" => "lastModified",
                _ => "downloads",
            };

            let page_idx = query.page.unwrap_or(0);
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
                ("expand[]", "tags".to_string()),
                ("expand[]", "siblings".to_string()),
                ("expand[]", "gated".to_string()),
                ("expand[]", "private".to_string()),
            ];

            if !search_text.is_empty() {
                query_params.push(("search", search_text.to_string()));
            }

            if let Some(m) = query.modality {
                let filter = match m {
                    "text-to-text" => Some("text-generation"),
                    "text-to-image" => Some("text-to-image"),
                    "text-to-speech" => Some("text-to-speech"),
                    "automatic-speech-recognition" => Some("automatic-speech-recognition"),
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

            let items: Vec<Value> = resp.json().await.unwrap_or_default();
            let mut results = Vec::new();

            let has_token = repo
                .api_token
                .as_ref()
                .map(|t| !t.trim().is_empty())
                .unwrap_or(false);

            let effective_base = repo
                .get_mirror_url()
                .unwrap_or_else(|| base_url.to_string());
            let effective_base = effective_base.trim_end_matches('/');

            for item in items {
                let id = match item.get("id").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => continue,
                };

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

                let tags: Vec<String> = item
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|t| t.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

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

                let sibling_filenames: Vec<String> =
                    siblings.iter().map(|s| s.rfilename.clone()).collect();

                let classification = ModalityClassifier::classify(&ClassificationContext {
                    raw_pipeline_tag: pipeline_tag.as_deref(),
                    tags: &tags,
                    model_id: &id,
                    sibling_filenames: &sibling_filenames,
                    architecture: None,
                    directory_hint: None,
                });

                results.push(HubModelItem {
                    id,
                    author,
                    downloads,
                    likes,
                    pipeline_tag,
                    modality: classification.modality.as_str().to_string(),
                    siblings,
                    is_gated,
                    requires_auth,
                    tags,
                });
            }

            Ok(results)
        })
    }

    fn get_model_details<'a>(
        &'a self,
        repo: &'a RepositoryRecord,
        model_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = AppResult<HubModelItem>> + Send + 'a>> {
        Box::pin(async move {
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

            let tags: Vec<String> = item
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let effective_base = repo
                .get_mirror_url()
                .unwrap_or_else(|| base_url.to_string());
            let effective_base = effective_base.trim_end_matches('/');

            let mut size_map = HashMap::new();
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
                        for t_item in tree_items {
                            let item_type = t_item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                            if item_type == "file" || item_type.is_empty() {
                                if let (Some(path), Some(size)) = (
                                    t_item.get("path").and_then(|p| p.as_str()),
                                    t_item.get("size").and_then(|s| s.as_u64()),
                                ) {
                                    size_map.insert(path.to_string(), size);
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

            let sibling_filenames: Vec<String> =
                siblings.iter().map(|s| s.rfilename.clone()).collect();

            let classification = ModalityClassifier::classify(&ClassificationContext {
                raw_pipeline_tag: pipeline_tag.as_deref(),
                tags: &tags,
                model_id: &id,
                sibling_filenames: &sibling_filenames,
                architecture: None,
                directory_hint: None,
            });

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
                modality: classification.modality.as_str().to_string(),
                siblings,
                is_gated,
                requires_auth,
                tags,
            })
        })
    }

    fn get_model_readme<'a>(
        &'a self,
        repo: &'a RepositoryRecord,
        model_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = AppResult<HubModelReadme>> + Send + 'a>> {
        Box::pin(async move {
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
        })
    }
}

