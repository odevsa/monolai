use crate::db::chats::{
    clear_chat_messages, delete_chat, delete_chat_message, get_all_chats, get_chat_by_id,
    get_chat_messages, insert_chat, insert_chat_message, update_chat_message, update_chat_title,
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
) -> Result<Json<Vec<ChatRecord>>, (StatusCode, String)> {
    get_all_chats(&state.db)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
) -> Result<Json<ChatRecord>, (StatusCode, String)> {
    let id = payload
        .id
        .unwrap_or_else(generate_uuid_v4);
    let title = payload.title.unwrap_or_else(|| "New Chat".to_string());

    insert_chat(&state.db, &id, &title)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
) -> Result<Json<ChatRecord>, (StatusCode, String)> {
    update_chat_title(&state.db, &id, &payload.title)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
) -> Result<StatusCode, (StatusCode, String)> {
    delete_chat(&state.db, &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
        (status = 200, description = "Messages for the chat", body = Vec<ChatMessageRecord>),
        (status = 500, description = "Database error", body = String)
    )
)]
pub async fn get_chat_messages_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ChatMessageRecord>>, (StatusCode, String)> {
    get_chat_messages(&state.db, &id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Add a message to a chat conversation
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
    Path(id): Path<String>,
    Json(payload): Json<CreateChatMessagePayload>,
) -> Result<Json<ChatMessageRecord>, (StatusCode, String)> {
    // Ensure chat exists, or create default chat if missing
    if get_chat_by_id(&state.db, &id).await.ok().flatten().is_none() {
        let _ = insert_chat(&state.db, &id, "New Chat").await;
    }

    insert_chat_message(&state.db, &id, &payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Update an existing chat message
#[utoipa::path(
    put,
    path = "/api/chats/{chat_id}/messages/{msg_id}",
    tag = "Chats",
    params(
        ("chat_id" = String, Path, description = "Chat ID"),
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
) -> Result<Json<ChatMessageRecord>, (StatusCode, String)> {
    update_chat_message(&state.db, &msg_id, &payload)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Delete a specific chat message
#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}/messages/{msg_id}",
    tag = "Chats",
    params(
        ("chat_id" = String, Path, description = "Chat ID"),
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
) -> Result<StatusCode, (StatusCode, String)> {
    delete_chat_message(&state.db, &msg_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
) -> Result<StatusCode, (StatusCode, String)> {
    clear_chat_messages(&state.db, &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

fn generate_uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let r1 = (now & 0xFFFFFFFF) as u32;
    let r2 = ((now >> 32) & 0xFFFF) as u16;
    let r3 = (((now >> 48) & 0x0FFF) as u16) | 0x4000;
    let r4 = (0x8000 | (now & 0x3FFF)) as u16;
    let r5 = (now >> 16) as u64;
    format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", r1, r2, r3, r4, r5)
}
