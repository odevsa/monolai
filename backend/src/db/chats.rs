use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct ChatRecord {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct ChatMessageRecord {
    pub id: String,
    pub chat_id: String,
    pub role: String,
    pub content: String,
    pub tokens: Option<i64>,
    pub duration: Option<String>,
    pub speed: Option<String>,
    pub model_tags: Option<String>,
    pub status: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateChatPayload {
    pub id: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateChatPayload {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateChatMessagePayload {
    pub id: Option<String>,
    pub role: String,
    pub content: String,
    pub tokens: Option<i64>,
    pub duration: Option<String>,
    pub speed: Option<String>,
    pub model_tags: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateChatMessagePayload {
    pub content: String,
    pub tokens: Option<i64>,
    pub duration: Option<String>,
    pub speed: Option<String>,
    pub model_tags: Option<String>,
    pub status: Option<String>,
}

pub async fn get_all_chats(pool: &SqlitePool) -> Result<Vec<ChatRecord>, sqlx::Error> {
    sqlx::query_as::<_, ChatRecord>("SELECT id, title, created_at, updated_at FROM chats ORDER BY updated_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn get_chat_by_id(pool: &SqlitePool, id: &str) -> Result<Option<ChatRecord>, sqlx::Error> {
    sqlx::query_as::<_, ChatRecord>("SELECT id, title, created_at, updated_at FROM chats WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_chat(
    pool: &SqlitePool,
    id: &str,
    title: &str,
) -> Result<ChatRecord, sqlx::Error> {
    sqlx::query("INSERT INTO chats (id, title, updated_at) VALUES (?, ?, datetime('now'))")
        .bind(id)
        .bind(title)
        .execute(pool)
        .await?;

    get_chat_by_id(pool, id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)
}

pub async fn update_chat_title(
    pool: &SqlitePool,
    id: &str,
    title: &str,
) -> Result<ChatRecord, sqlx::Error> {
    sqlx::query("UPDATE chats SET title = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(title)
        .bind(id)
        .execute(pool)
        .await?;

    get_chat_by_id(pool, id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)
}

pub async fn touch_chat_updated_at(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE chats SET updated_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_chat(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM chat_messages WHERE chat_id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM chats WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_chat_messages(
    pool: &SqlitePool,
    chat_id: &str,
) -> Result<Vec<ChatMessageRecord>, sqlx::Error> {
    sqlx::query_as::<_, ChatMessageRecord>(
        "SELECT id, chat_id, role, content, tokens, duration, speed, model_tags, status, created_at FROM chat_messages WHERE chat_id = ? ORDER BY created_at ASC",
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await
}

pub async fn insert_chat_message(
    pool: &SqlitePool,
    chat_id: &str,
    payload: &CreateChatMessagePayload,
) -> Result<ChatMessageRecord, sqlx::Error> {
    let msg_id = payload.id.clone().unwrap_or_else(|| uuid_simple());

    sqlx::query(
        "INSERT INTO chat_messages (id, chat_id, role, content, tokens, duration, speed, model_tags, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&msg_id)
    .bind(chat_id)
    .bind(&payload.role)
    .bind(&payload.content)
    .bind(payload.tokens)
    .bind(&payload.duration)
    .bind(&payload.speed)
    .bind(&payload.model_tags)
    .bind(&payload.status)
    .execute(pool)
    .await?;

    let _ = touch_chat_updated_at(pool, chat_id).await;

    sqlx::query_as::<_, ChatMessageRecord>(
        "SELECT id, chat_id, role, content, tokens, duration, speed, model_tags, status, created_at FROM chat_messages WHERE id = ?",
    )
    .bind(&msg_id)
    .fetch_one(pool)
    .await
}

pub async fn update_chat_message(
    pool: &SqlitePool,
    msg_id: &str,
    payload: &UpdateChatMessagePayload,
) -> Result<ChatMessageRecord, sqlx::Error> {
    sqlx::query(
        "UPDATE chat_messages SET content = ?, tokens = ?, duration = ?, speed = ?, model_tags = COALESCE(?, model_tags), status = COALESCE(?, status) WHERE id = ?",
    )
    .bind(&payload.content)
    .bind(payload.tokens)
    .bind(&payload.duration)
    .bind(&payload.speed)
    .bind(&payload.model_tags)
    .bind(&payload.status)
    .bind(msg_id)
    .execute(pool)
    .await?;

    sqlx::query_as::<_, ChatMessageRecord>(
        "SELECT id, chat_id, role, content, tokens, duration, speed, model_tags, status, created_at FROM chat_messages WHERE id = ?",
    )
    .bind(msg_id)
    .fetch_one(pool)
    .await
}

pub async fn delete_chat_message(pool: &SqlitePool, msg_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM chat_messages WHERE id = ?")
        .bind(msg_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn clear_chat_messages(pool: &SqlitePool, chat_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM chat_messages WHERE chat_id = ?")
        .bind(chat_id)
        .execute(pool)
        .await?;
    Ok(())
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}
