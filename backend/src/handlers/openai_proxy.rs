use crate::db::models::{get_all_models, get_model_by_id};
use crate::runtimes::process_manager::{
    get_all_running_status, mark_model_active, swap_model_process, ModelState,
};
use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, utoipa::ToSchema)]
pub struct OpenAiModelItem {
    pub id: String,
    pub object: &'static str,
    pub created: u64,
    pub owned_by: &'static str,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct OpenAiModelList {
    pub object: &'static str,
    pub data: Vec<OpenAiModelItem>,
}

/// List all models in OpenAI standard format (GET /v1/models)
#[utoipa::path(
    get,
    path = "/v1/models",
    tag = "OpenAI Compatibility",
    responses(
        (status = 200, description = "List of models in OpenAI format", body = OpenAiModelList),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn v1_models_handler(
    State(state): State<AppState>,
) -> Result<Json<OpenAiModelList>, (StatusCode, Json<Value>)> {
    let models = get_all_models(&state.db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "message": format!("Database error: {}", e),
                    "type": "server_error",
                    "param": null,
                    "code": "internal_error"
                }
            })),
        )
    })?;

    let status = state.config_status.lock().unwrap();
    let models_dir = if let Some(ref dir_str) = status.models_dir {
        crate::config::expand_tilde(dir_str)
    } else {
        crate::config::get_default_models_dir()
    };

    let data = models
        .into_iter()
        .filter(|m| {
            let (exists, _) = crate::runtimes::model_scanner::check_model_file_exists(&models_dir, &m.flags);
            exists
        })
        .map(|m| OpenAiModelItem {
            id: m.id,
            object: "model",
            created: 1680000000,
            owned_by: "monolai",
        })
        .collect();

    Ok(Json(OpenAiModelList {
        object: "list",
        data,
    }))
}

/// Get a specific model in OpenAI standard format (GET /v1/models/{id})
#[utoipa::path(
    get,
    path = "/v1/models/{id}",
    tag = "OpenAI Compatibility",
    params(
        ("id" = String, Path, description = "Model ID")
    ),
    responses(
        (status = 200, description = "Model details in OpenAI format", body = OpenAiModelItem),
        (status = 404, description = "Model not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn v1_model_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<OpenAiModelItem>, (StatusCode, Json<Value>)> {
    let model = get_model_by_id(&state.db, &id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "message": format!("Database error: {}", e),
                    "type": "server_error",
                    "param": null,
                    "code": "internal_error"
                }
            })),
        )
    })?;

    if let Some(m) = model {
        let status = state.config_status.lock().unwrap();
        let models_dir = if let Some(ref dir_str) = status.models_dir {
            crate::config::expand_tilde(dir_str)
        } else {
            crate::config::get_default_models_dir()
        };
        let (exists, _) = crate::runtimes::model_scanner::check_model_file_exists(&models_dir, &m.flags);
        if !exists {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Model file for '{}' not found on disk", id),
                        "type": "invalid_request_error",
                        "param": "model",
                        "code": "model_file_not_found"
                    }
                })),
            ));
        }

        Ok(Json(OpenAiModelItem {
            id: m.id,
            object: "model",
            created: 1680000000,
            owned_by: "monolai",
        }))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": {
                    "message": format!("Model '{}' not found in database", id),
                    "type": "invalid_request_error",
                    "param": "id",
                    "code": "model_not_found"
                }
            })),
        ))
    }
}

/// Wildcard proxy handler for /v1/*path (e.g., /v1/chat/completions, /v1/completions, /v1/embeddings, etc.)
pub async fn v1_proxy_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request,
) -> Response {
    let (parts, body) = req.into_parts();
    let method = parts.method;
    let headers = parts.headers;
    let uri = parts.uri;

    // 1. Read request body bytes up to 50MB
    let body_bytes = match axum::body::to_bytes(body, 50 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Failed to read request body: {}", e),
                        "type": "invalid_request_error",
                        "param": null,
                        "code": "bad_request"
                    }
                })),
            )
                .into_response();
        }
    };

    // 2. Extract model ID from JSON payload or query string
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

    // 3. Resolve target port & ensure model is swapped and ready
    let target_port: u16 = if let Some(model_id) = model_id_opt {
        // Verify model exists in database first
        match get_model_by_id(&state.db, &model_id).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": {
                            "message": format!("Model '{}' not found in database", model_id),
                            "type": "invalid_request_error",
                            "param": "model",
                            "code": "model_not_found"
                        }
                    })),
                )
                    .into_response();
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": {
                            "message": format!("Database error checking model: {}", e),
                            "type": "server_error",
                            "param": "model",
                            "code": "internal_error"
                        }
                    })),
                )
                    .into_response();
            }
        }

        // Swap to model
        let config = state.config.lock().unwrap().clone();
        match swap_model_process(&state.process_manager, &state.db, &config, &model_id).await {
            Ok(running_status) => {
                if running_status.state != ModelState::Ready || running_status.port == 0 {
                    return (
                        StatusCode::CONFLICT, // 409 if not loaded/ready
                        Json(serde_json::json!({
                            "error": {
                                "message": format!("Model '{}' is not loaded or ready", model_id),
                                "type": "server_error",
                                "param": "model",
                                "code": "model_not_ready"
                            }
                        })),
                    )
                        .into_response();
                }
                mark_model_active(&state.process_manager, &model_id).await;
                running_status.port
            }
            Err(err_msg) => {
                return (
                    StatusCode::CONFLICT, // 409 if loading/swapping failed
                    Json(serde_json::json!({
                        "error": {
                            "message": format!("Failed to swap model '{}': {}", model_id, err_msg),
                            "type": "server_error",
                            "param": "model",
                            "code": "model_swap_failed"
                        }
                    })),
                )
                    .into_response();
            }
        }
    } else {
        // Fallback: If no model specified in request, check if any model is currently running & ready
        let running_list = get_all_running_status(&state.process_manager).await;
        let ready = running_list
            .into_iter()
            .find(|m| m.state == ModelState::Ready && m.port > 0);
        match ready {
            Some(m) => {
                mark_model_active(&state.process_manager, &m.model_id).await;
                m.port
            }
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": {
                            "message": "No 'model' parameter specified in request and no model is currently running.",
                            "type": "invalid_request_error",
                            "param": "model",
                            "code": "missing_model_parameter"
                        }
                    })),
                )
                    .into_response();
            }
        }
    };

    // 4. Build target URL to proxy upstream process
    let query_suffix = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();
    let upstream_url = format!("http://127.0.0.1:{}/v1/{}{}", target_port, path, query_suffix);

    // 5. Send proxy request to upstream process with automatic retry if model is still loading
    let client = reqwest::Client::new();
    let max_retries = 60; // 60 * 500ms = 30 seconds
    let mut retry_count = 0;

    let upstream_res = loop {
        let mut req_builder = client.request(method.clone(), &upstream_url);

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
                if res.status() == StatusCode::SERVICE_UNAVAILABLE && retry_count < max_retries {
                    let res_headers = res.headers().clone();
                    match res.bytes().await {
                        Ok(bytes) => {
                            let text = String::from_utf8_lossy(&bytes).to_lowercase();
                            if text.contains("loading model") {
                                tracing::info!(
                                    "Upstream returned 'Loading model' (503), waiting and retrying ({}/{})...",
                                    retry_count + 1,
                                    max_retries
                                );
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                retry_count += 1;
                                continue;
                            } else {
                                // Non-'Loading model' 503 error, return it
                                let mut response_builder = Response::builder().status(StatusCode::SERVICE_UNAVAILABLE);
                                for (name, value) in &res_headers {
                                    let name_str = name.as_str().to_lowercase();
                                    if name_str != "transfer-encoding" && name_str != "connection" {
                                        response_builder = response_builder.header(name, value);
                                    }
                                }
                                return response_builder
                                    .body(Body::from(bytes))
                                    .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Proxy response error").into_response());
                            }
                        }
                        Err(e) => {
                            return (
                                StatusCode::SERVICE_UNAVAILABLE,
                                Json(serde_json::json!({
                                    "error": {
                                        "message": format!("Upstream read error: {}", e),
                                        "type": "upstream_error",
                                        "param": null,
                                        "code": "upstream_read_error"
                                    }
                                })),
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
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    retry_count += 1;
                    continue;
                }
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({
                        "error": {
                            "message": format!("Upstream connection error: {}", e),
                            "type": "bad_gateway",
                            "param": null,
                            "code": "upstream_error"
                        }
                    })),
                )
                    .into_response();
            }
        }
    };

    // 6. Transparently stream upstream response back to consumer
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

    response_builder
        .body(response_body)
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Proxy response error").into_response())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct OpenAiChatMessage {
    /// Role of the message author ("system", "user", "assistant")
    pub role: String,
    /// Content of the message
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct OpenAiChatCompletionRequest {
    /// ID of the model to use (e.g. "llama-3-8b")
    pub model: String,
    /// List of messages comprising the conversation so far
    pub messages: Vec<OpenAiChatMessage>,
    /// Sampling temperature (0.0 to 2.0)
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Whether to stream back partial tokens via SSE
    #[serde(default)]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct OpenAiCompletionRequest {
    /// ID of the model to use
    pub model: String,
    /// Prompt text to complete
    pub prompt: String,
    /// Maximum tokens to generate
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Sampling temperature
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Whether to stream back partial tokens
    #[serde(default)]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct OpenAiEmbeddingRequest {
    /// ID of the model to use
    pub model: String,
    /// Input text or array of strings to embed
    #[schema(value_type = Object)]
    pub input: serde_json::Value,
}

/// Create a chat completion (OpenAI standard POST /v1/chat/completions)
#[utoipa::path(
    post,
    path = "/v1/chat/completions",
    tag = "OpenAI Compatibility",
    request_body = OpenAiChatCompletionRequest,
    responses(
        (status = 200, description = "Chat completion response or Server-Sent Events stream"),
        (status = 400, description = "Invalid request payload or missing model"),
        (status = 502, description = "Upstream runtime process connection error")
    )
)]
#[allow(dead_code)]
pub async fn v1_chat_completions_doc() {}

/// Create a text completion (OpenAI standard POST /v1/completions)
#[utoipa::path(
    post,
    path = "/v1/completions",
    tag = "OpenAI Compatibility",
    request_body = OpenAiCompletionRequest,
    responses(
        (status = 200, description = "Completion response or Server-Sent Events stream"),
        (status = 400, description = "Invalid request payload or missing model"),
        (status = 502, description = "Upstream runtime process connection error")
    )
)]
#[allow(dead_code)]
pub async fn v1_completions_doc() {}

/// Create embeddings for given input (OpenAI standard POST /v1/embeddings)
#[utoipa::path(
    post,
    path = "/v1/embeddings",
    tag = "OpenAI Compatibility",
    request_body = OpenAiEmbeddingRequest,
    responses(
        (status = 200, description = "Embeddings response"),
        (status = 400, description = "Invalid request payload or missing model"),
        (status = 502, description = "Upstream runtime process connection error")
    )
)]
#[allow(dead_code)]
pub async fn v1_embeddings_doc() {}

