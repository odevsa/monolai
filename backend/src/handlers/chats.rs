use crate::core::error::AppResult;
use crate::domain::{
    ChatMessageRecord, ChatRecord, CreateChatMessagePayload, CreateChatPayload,
    UpdateChatMessagePayload, UpdateChatPayload,
};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

/// List all chat conversations
#[utoipa::path(
    get,
    path = "/api/chats",
    tag = "Chats",
    responses(
        (status = 200, description = "List of all chat sessions", body = Vec<ChatRecord>),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn get_chats_handler(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ChatRecord>>> {
    let chats = state.chat_service.get_all_chats().await?;
    Ok(Json(chats))
}

/// Create a new chat conversation
#[utoipa::path(
    post,
    path = "/api/chats",
    tag = "Chats",
    request_body = CreateChatPayload,
    responses(
        (status = 200, description = "Chat created successfully", body = ChatRecord),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn create_chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateChatPayload>,
) -> AppResult<Json<ChatRecord>> {
    let chat = state.chat_service.create_chat(payload).await?;
    Ok(Json(chat))
}

/// Rename/update a chat conversation
#[utoipa::path(
    put,
    path = "/api/chats/{id}",
    tag = "Chats",
    params(
        ("id" = String, Path, description = "Chat ID")
    ),
    request_body = UpdateChatPayload,
    responses(
        (status = 200, description = "Chat updated successfully", body = ChatRecord),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn update_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateChatPayload>,
) -> AppResult<Json<ChatRecord>> {
    let chat = state.chat_service.update_chat(&id, payload).await?;
    Ok(Json(chat))
}

/// Delete a chat conversation and all its messages
#[utoipa::path(
    delete,
    path = "/api/chats/{id}",
    tag = "Chats",
    params(
        ("id" = String, Path, description = "Chat ID")
    ),
    responses(
        (status = 204, description = "Chat deleted successfully"),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn delete_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.chat_service.delete_chat(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// List all messages in a chat conversation
#[utoipa::path(
    get,
    path = "/api/chats/{id}/messages",
    tag = "Chats",
    params(
        ("id" = String, Path, description = "Chat ID")
    ),
    responses(
        (status = 200, description = "List of chat messages", body = Vec<ChatMessageRecord>),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn get_chat_messages_handler(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
) -> AppResult<Json<Vec<ChatMessageRecord>>> {
    let messages = state.chat_service.get_chat_messages(&chat_id).await?;
    Ok(Json(messages))
}

/// Append a new message to a chat conversation
#[utoipa::path(
    post,
    path = "/api/chats/{id}/messages",
    tag = "Chats",
    params(
        ("id" = String, Path, description = "Chat ID")
    ),
    request_body = CreateChatMessagePayload,
    responses(
        (status = 200, description = "Message created successfully", body = ChatMessageRecord),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn create_chat_message_handler(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    Json(payload): Json<CreateChatMessagePayload>,
) -> AppResult<Json<ChatMessageRecord>> {
    let message = state.chat_service.create_chat_message(&chat_id, payload).await?;
    Ok(Json(message))
}

/// Update an existing chat message
#[utoipa::path(
    put,
    path = "/api/chats/{id}/messages/{msg_id}",
    tag = "Chats",
    params(
        ("id" = String, Path, description = "Chat ID"),
        ("msg_id" = String, Path, description = "Message ID")
    ),
    request_body = UpdateChatMessagePayload,
    responses(
        (status = 200, description = "Message updated successfully", body = ChatMessageRecord),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn update_chat_message_handler(
    State(state): State<AppState>,
    Path((_chat_id, msg_id)): Path<(String, String)>,
    Json(payload): Json<UpdateChatMessagePayload>,
) -> AppResult<Json<ChatMessageRecord>> {
    let message = state.chat_service.update_chat_message(&msg_id, payload).await?;
    Ok(Json(message))
}

/// Delete a specific chat message
#[utoipa::path(
    delete,
    path = "/api/chats/{id}/messages/{msg_id}",
    tag = "Chats",
    params(
        ("id" = String, Path, description = "Chat ID"),
        ("msg_id" = String, Path, description = "Message ID")
    ),
    responses(
        (status = 204, description = "Message deleted successfully"),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn delete_chat_message_handler(
    State(state): State<AppState>,
    Path((_chat_id, msg_id)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    state.chat_service.delete_chat_message(&msg_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Clear all messages in a chat conversation
#[utoipa::path(
    delete,
    path = "/api/chats/{id}/messages",
    tag = "Chats",
    params(
        ("id" = String, Path, description = "Chat ID")
    ),
    responses(
        (status = 204, description = "Messages cleared successfully"),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn clear_chat_messages_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.chat_service.clear_chat_messages(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
