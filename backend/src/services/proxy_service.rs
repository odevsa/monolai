use crate::core::config::AppConfig;
use crate::core::error::{AppError, AppResult};
use crate::domain::{ModelState, OpenAiModelItem, OpenAiModelList};
use crate::infrastructure::db::{ModelRepository, SettingRepository};
use crate::infrastructure::process::manager::{swap_model_process, ProcessManager};
use crate::infrastructure::scanner::check_model_file_exists;
use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ProxyService {
    model_repo: ModelRepository,
    setting_repo: SettingRepository,
    process_manager: ProcessManager,
    config: Arc<RwLock<AppConfig>>,
    http_client: reqwest::Client,
}

impl ProxyService {
    pub fn new(
        model_repo: ModelRepository,
        setting_repo: SettingRepository,
        process_manager: ProcessManager,
        config: Arc<RwLock<AppConfig>>,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_default();

        Self {
            model_repo,
            setting_repo,
            process_manager,
            config,
            http_client,
        }
    }

    pub async fn list_v1_models(&self) -> AppResult<OpenAiModelList> {
        let models = self.model_repo.get_all().await?;
        let cfg = self.config.read().await;
        let models_dir = if let Some(ref dir) = cfg.models {
            crate::core::config::expand_tilde(dir)
        } else {
            crate::core::config::get_default_models_dir()
        };

        let data = models
            .into_iter()
            .filter(|m| {
                let (exists, _) = check_model_file_exists(&models_dir, &m.flags);
                exists
            })
            .map(|m| OpenAiModelItem {
                id: m.id,
                object: "model",
                created: 1680000000,
                owned_by: "monolai",
            })
            .collect();

        Ok(OpenAiModelList {
            object: "list",
            data,
        })
    }

    pub async fn get_v1_model(&self, id: &str) -> AppResult<OpenAiModelItem> {
        let model = self.model_repo.get_by_id(id).await?;

        if let Some(m) = model {
            let cfg = self.config.read().await;
            let models_dir = if let Some(ref dir) = cfg.models {
                crate::core::config::expand_tilde(dir)
            } else {
                crate::core::config::get_default_models_dir()
            };

            let (exists, _) = check_model_file_exists(&models_dir, &m.flags);
            if !exists {
                return Err(AppError::openai(
                    StatusCode::NOT_FOUND,
                    format!("Model file for '{}' not found on disk", id),
                    "invalid_request_error",
                    "model_file_not_found",
                ));
            }

            Ok(OpenAiModelItem {
                id: m.id,
                object: "model",
                created: 1680000000,
                owned_by: "monolai",
            })
        } else {
            Err(AppError::openai(
                StatusCode::NOT_FOUND,
                format!("Model '{}' not found in database", id),
                "invalid_request_error",
                "model_not_found",
            ))
        }
    }

    pub async fn forward_v1(
        &self,
        path: &str,
        method: Method,
        headers: HeaderMap,
        uri: Uri,
        body_bytes: Bytes,
    ) -> Response {
        let mut model_id_opt: Option<String> = None;

        if !body_bytes.is_empty() {
            if let Ok(json_val) = serde_json::from_slice::<Value>(&body_bytes) {
                if let Some(m) = json_val.get("model").and_then(|v| v.as_str()) {
                    model_id_opt = Some(m.to_string());
                }
            }
        }

        if model_id_opt.is_none() {
            if let Some(query_str) = uri.query() {
                for pair in query_str.split('&') {
                    let mut parts = pair.splitn(2, '=');
                    if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                        if k == "model" {
                            model_id_opt = Some(v.to_string());
                            break;
                        }
                    }
                }
            }
        }

        let target_port: u16 = if let Some(model_id) = model_id_opt {
            match self.model_repo.get_by_id(&model_id).await {
                Ok(Some(_)) => {}
                Ok(None) => {
                    return AppError::openai(
                        StatusCode::NOT_FOUND,
                        format!("Model '{}' not found in database", model_id),
                        "invalid_request_error",
                        "model_not_found",
                    )
                    .into_response();
                }
                Err(e) => {
                    return AppError::openai(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database error checking model: {}", e),
                        "server_error",
                        "internal_error",
                    )
                    .into_response();
                }
            }

            let cfg = self.config.read().await.clone();
            match swap_model_process(
                &self.process_manager,
                &self.model_repo,
                &self.setting_repo,
                &cfg,
                &model_id,
            )
            .await
            {
                Ok(running_status) => {
                    if running_status.state != ModelState::Ready || running_status.port == 0 {
                        return AppError::openai(
                            StatusCode::CONFLICT,
                            format!("Model '{}' is not loaded or ready", model_id),
                            "server_error",
                            "model_not_ready",
                        )
                        .into_response();
                    }
                    self.process_manager.mark_model_active(&model_id).await;
                    running_status.port
                }
                Err(err) => {
                    return AppError::openai(
                        StatusCode::CONFLICT,
                        format!("Failed to swap model '{}': {}", model_id, err),
                        "server_error",
                        "model_swap_failed",
                    )
                    .into_response();
                }
            }
        } else {
            let running_list = self.process_manager.get_all_running_status().await;
            let ready = running_list
                .into_iter()
                .find(|m| m.state == ModelState::Ready && m.port > 0);
            match ready {
                Some(m) => {
                    self.process_manager.mark_model_active(&m.model_id).await;
                    m.port
                }
                None => {
                    return AppError::openai(
                        StatusCode::BAD_REQUEST,
                        "No 'model' parameter specified in request and no model is currently running.",
                        "invalid_request_error",
                        "missing_model_parameter",
                    )
                    .into_response();
                }
            }
        };

        let query_suffix = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();
        let upstream_url = format!(
            "http://127.0.0.1:{}/v1/{}{}",
            target_port, path, query_suffix
        );

        let mut final_body_bytes = body_bytes;

        // If this is an image generation request forwarded to sd-server/sd-cpp, ensure
        // parameters like seed, steps, cfg_scale, and negative_prompt are embedded via
        // <sd_cpp_extra_args> if not already present in the prompt string.
        if (path == "images/generations" || path.starts_with("images/generations"))
            && !final_body_bytes.is_empty()
        {
            if let Ok(mut json_val) = serde_json::from_slice::<Value>(&final_body_bytes) {
                if let Some(prompt_str) = json_val
                    .get("prompt")
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string())
                {
                    if !prompt_str.contains("<sd_cpp_extra_args>") {
                        let mut extra_args = serde_json::Map::new();

                        // If seed is provided and non-negative, use it; otherwise default to -1 (random)
                        // to prevent sd-server from falling back to default seed 42.
                        let seed_val = match json_val.get("seed").and_then(|s| s.as_i64()) {
                            Some(s) if s >= 0 => serde_json::json!(s),
                            _ => serde_json::json!(-1),
                        };
                        extra_args.insert("seed".to_string(), seed_val);

                        if let Some(neg) = json_val.get("negative_prompt") {
                            extra_args.insert("negative_prompt".to_string(), neg.clone());
                        }

                        if let Some(steps) = json_val
                            .get("steps")
                            .or_else(|| json_val.get("sample_steps"))
                        {
                            extra_args.insert("sample_steps".to_string(), steps.clone());
                        }

                        if let Some(cfg) = json_val.get("cfg_scale") {
                            extra_args.insert("cfg_scale".to_string(), cfg.clone());
                        }

                        let extra_json = serde_json::to_string(&extra_args).unwrap_or_default();
                        let new_prompt = format!(
                            "{} <sd_cpp_extra_args>{}</sd_cpp_extra_args>",
                            prompt_str.trim(),
                            extra_json
                        );

                        if let Some(obj) = json_val.as_object_mut() {
                            obj.insert("prompt".to_string(), Value::String(new_prompt));
                        }
                        if let Ok(new_bytes) = serde_json::to_vec(&json_val) {
                            final_body_bytes = Bytes::from(new_bytes);
                        }
                    }
                }
            }
        }

        self.forward_http(upstream_url, method, headers, final_body_bytes, true)
            .await
    }

    pub async fn forward_sdcpp(
        &self,
        path: &str,
        method: Method,
        headers: HeaderMap,
        uri: Uri,
        body_bytes: Bytes,
    ) -> Response {
        let target_port: u16 = {
            let running_list = self.process_manager.get_all_running_status().await;
            let ready_sd = running_list.into_iter().find(|m| {
                (m.runtime_id == "sd-cpp" || m.runtime_id.contains("sd"))
                    && m.state == ModelState::Ready
                    && m.port > 0
            });

            if let Some(m) = ready_sd {
                self.process_manager.mark_model_active(&m.model_id).await;
                m.port
            } else {
                let all_models = self.model_repo.get_all().await.unwrap_or_default();
                let sd_model = all_models
                    .into_iter()
                    .find(|m| m.runtime == "sd-cpp" || m.runtime.contains("sd"));
                if let Some(m) = sd_model {
                    let cfg = self.config.read().await.clone();
                    match swap_model_process(
                        &self.process_manager,
                        &self.model_repo,
                        &self.setting_repo,
                        &cfg,
                        &m.id,
                    )
                    .await
                    {
                        Ok(running) => running.port,
                        Err(e) => {
                            return (
                                StatusCode::BAD_GATEWAY,
                                Json(serde_json::json!({
                                    "error": { "message": format!("Failed to start sd-cpp model: {}", e) }
                                })),
                            )
                                .into_response();
                        }
                    }
                } else {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "error": { "message": "No Stable Diffusion model registered or currently running." }
                        })),
                    )
                        .into_response();
                }
            }
        };

        let query_suffix = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();
        let upstream_url = format!(
            "http://127.0.0.1:{}/sdcpp/{}{}",
            target_port, path, query_suffix
        );

        self.forward_http(upstream_url, method, headers, body_bytes, false)
            .await
    }

    async fn forward_http(
        &self,
        upstream_url: String,
        method: Method,
        headers: HeaderMap,
        body_bytes: Bytes,
        retry_loading: bool,
    ) -> Response {
        let max_retries = if retry_loading { 60 } else { 0 };
        let mut retry_count = 0;

        let upstream_res = loop {
            let mut req_builder = self.http_client.request(method.clone(), &upstream_url);

            for (name, value) in &headers {
                let name_str = name.as_str().to_lowercase();
                if name_str != "host" && name_str != "content-length" && name_str != "connection" {
                    req_builder = req_builder.header(name, value);
                }
            }

            if !body_bytes.is_empty() {
                req_builder = req_builder.body(body_bytes.clone());
            }

            match req_builder.send().await {
                Ok(res) => {
                    if res.status() == StatusCode::SERVICE_UNAVAILABLE && retry_count < max_retries
                    {
                        let res_headers = res.headers().clone();
                        match res.bytes().await {
                            Ok(bytes) => {
                                let text = String::from_utf8_lossy(&bytes).to_lowercase();
                                if text.contains("loading model") {
                                    tracing::info!(
                                        "Upstream returned 'Loading model' (503), retrying ({}/{})...",
                                        retry_count + 1,
                                        max_retries
                                    );
                                    tokio::time::sleep(Duration::from_millis(500)).await;
                                    retry_count += 1;
                                    continue;
                                } else {
                                    let mut response_builder =
                                        Response::builder().status(StatusCode::SERVICE_UNAVAILABLE);
                                    for (name, value) in &res_headers {
                                        let name_str = name.as_str().to_lowercase();
                                        if name_str != "transfer-encoding"
                                            && name_str != "connection"
                                        {
                                            response_builder = response_builder.header(name, value);
                                        }
                                    }
                                    return response_builder
                                        .body(Body::from(bytes))
                                        .unwrap_or_else(|_| {
                                            (
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                                "Proxy response error",
                                            )
                                                .into_response()
                                        });
                                }
                            }
                            Err(e) => {
                                return AppError::openai(
                                    StatusCode::SERVICE_UNAVAILABLE,
                                    format!("Upstream read error: {}", e),
                                    "upstream_error",
                                    "upstream_read_error",
                                )
                                .into_response();
                            }
                        }
                    } else {
                        break res;
                    }
                }
                Err(e) => {
                    if retry_count < 10 {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        retry_count += 1;
                        continue;
                    }
                    return AppError::openai(
                        StatusCode::BAD_GATEWAY,
                        format!("Upstream connection error: {}", e),
                        "bad_gateway",
                        "upstream_error",
                    )
                    .into_response();
                }
            }
        };

        let status = upstream_res.status();
        let mut response_builder = Response::builder().status(status);

        for (name, value) in upstream_res.headers() {
            let name_str = name.as_str().to_lowercase();
            if name_str != "transfer-encoding" && name_str != "connection" {
                response_builder = response_builder.header(name, value);
            }
        }

        let body_stream = upstream_res.bytes_stream();
        let response_body = Body::from_stream(body_stream);

        response_builder.body(response_body).unwrap_or_else(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "Proxy response error").into_response()
        })
    }
}
