use crate::core::AppResult;
use crate::domain::{
    ChatMessageRecord, ChatRecord, CreateChatMessagePayload, UpdateChatMessagePayload,
};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ChatRepository {
    pool: SqlitePool,
}

impl ChatRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_all_chats(&self) -> AppResult<Vec<ChatRecord>> {
        let rows = sqlx::query_as::<_, ChatRecord>(
            "SELECT id, title, created_at, updated_at FROM chats ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_chat_by_id(&self, id: &str) -> AppResult<Option<ChatRecord>> {
        let row = sqlx::query_as::<_, ChatRecord>(
            "SELECT id, title, created_at, updated_at FROM chats WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn insert_chat(&self, id: &str, title: &str) -> AppResult<ChatRecord> {
        sqlx::query("INSERT INTO chats (id, title, updated_at) VALUES (?, ?, datetime('now'))")
            .bind(id)
            .bind(title)
            .execute(&self.pool)
            .await?;

        self.get_chat_by_id(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound.into())
    }

    pub async fn update_chat_title(&self, id: &str, title: &str) -> AppResult<ChatRecord> {
        sqlx::query("UPDATE chats SET title = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(title)
            .bind(id)
            .execute(&self.pool)
            .await?;

        self.get_chat_by_id(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound.into())
    }

    pub async fn touch_chat_updated_at(&self, id: &str) -> AppResult<()> {
        sqlx::query("UPDATE chats SET updated_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_chat(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM chat_messages WHERE chat_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM chats WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_chat_messages(&self, chat_id: &str) -> AppResult<Vec<ChatMessageRecord>> {
        let rows = sqlx::query_as::<_, ChatMessageRecord>(
            "SELECT id, chat_id, role, content, tokens, duration, speed, model_tags, status, created_at FROM chat_messages WHERE chat_id = ? ORDER BY created_at ASC",
        )
        .bind(chat_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn insert_chat_message(
        &self,
        chat_id: &str,
        payload: &CreateChatMessagePayload,
    ) -> AppResult<ChatMessageRecord> {
        let msg_id = payload
            .id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

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
        .execute(&self.pool)
        .await?;

        let _ = self.touch_chat_updated_at(chat_id).await;

        let row = sqlx::query_as::<_, ChatMessageRecord>(
            "SELECT id, chat_id, role, content, tokens, duration, speed, model_tags, status, created_at FROM chat_messages WHERE id = ?",
        )
        .bind(&msg_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_chat_message(
        &self,
        msg_id: &str,
        payload: &UpdateChatMessagePayload,
    ) -> AppResult<ChatMessageRecord> {
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
        .execute(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, ChatMessageRecord>(
            "SELECT id, chat_id, role, content, tokens, duration, speed, model_tags, status, created_at FROM chat_messages WHERE id = ?",
        )
        .bind(msg_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_chat_message(&self, msg_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM chat_messages WHERE id = ?")
            .bind(msg_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn clear_chat_messages(&self, chat_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM chat_messages WHERE chat_id = ?")
            .bind(chat_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
