use crate::core::error::{AppError, AppResult};
use crate::domain::{
    OpenAiChatCompletionRequest, OpenAiCompletionRequest, OpenAiEmbeddingRequest,
    OpenAiImageGenerationRequest, OpenAiImageGenerationResponse, OpenAiModelItem, OpenAiModelList,
};
use crate::state::AppState;
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};

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
) -> AppResult<Json<OpenAiModelList>> {
    let models = state.proxy_service.list_v1_models().await?;
    Ok(Json(models))
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
) -> AppResult<Json<OpenAiModelItem>> {
    let model = state.proxy_service.get_v1_model(&id).await?;
    Ok(Json(model))
}

/// Wildcard proxy handler for /v1/*path
pub async fn v1_proxy_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request,
) -> Response {
    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, 50 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(e) => {
            return AppError::openai(
                StatusCode::BAD_REQUEST,
                format!("Failed to read request body: {}", e),
                "invalid_request_error",
                "bad_request",
            )
            .into_response();
        }
    };

    state
        .proxy_service
        .forward_v1(&path, parts.method, parts.headers, parts.uri, body_bytes)
        .await
}

/// Wildcard proxy handler for native Stable Diffusion endpoints (/sdcpp/*path)
pub async fn sdcpp_proxy_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    req: Request,
) -> Response {
    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, 50 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": { "message": format!("Failed to read request body: {}", e) }
                })),
            )
                .into_response();
        }
    };

    state
        .proxy_service
        .forward_sdcpp(&path, parts.method, parts.headers, parts.uri, body_bytes)
        .await
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

/// Create images using local diffusion model (OpenAI standard POST /v1/images/generations)
#[utoipa::path(
    post,
    path = "/v1/images/generations",
    tag = "OpenAI Compatibility",
    request_body = OpenAiImageGenerationRequest,
    responses(
        (status = 200, description = "Generated image(s) response in OpenAI standard format", body = OpenAiImageGenerationResponse),
        (status = 400, description = "Invalid request payload or missing prompt/model"),
        (status = 502, description = "Upstream runtime process connection error")
    )
)]
#[allow(dead_code)]
pub async fn v1_images_generations_doc() {}
