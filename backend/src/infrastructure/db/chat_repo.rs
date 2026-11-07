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

    pub async fn delete_all_chats(&self) -> AppResult<u64> {
        sqlx::query("DELETE FROM chat_messages")
            .execute(&self.pool)
            .await?;

        let res = sqlx::query("DELETE FROM chats")
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn get_chat_messages(&self, chat_id: &str) -> AppResult<Vec<ChatMessageRecord>> {
        let rows = sqlx::query_as::<_, ChatMessageRecord>(
            "SELECT id, chat_id, role, content, tokens, duration, speed, model_tags, status, attachments, created_at FROM chat_messages WHERE chat_id = ? ORDER BY created_at ASC",
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
            "INSERT INTO chat_messages (id, chat_id, role, content, tokens, duration, speed, model_tags, status, attachments) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
        .bind(&payload.attachments)
        .execute(&self.pool)
        .await?;

        let _ = self.touch_chat_updated_at(chat_id).await;

        let row = sqlx::query_as::<_, ChatMessageRecord>(
            "SELECT id, chat_id, role, content, tokens, duration, speed, model_tags, status, attachments, created_at FROM chat_messages WHERE id = ?",
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
            "UPDATE chat_messages SET content = ?, tokens = ?, duration = ?, speed = ?, model_tags = COALESCE(?, model_tags), status = COALESCE(?, status), attachments = COALESCE(?, attachments) WHERE id = ?",
        )
        .bind(&payload.content)
        .bind(payload.tokens)
        .bind(&payload.duration)
        .bind(&payload.speed)
        .bind(&payload.model_tags)
        .bind(&payload.status)
        .bind(&payload.attachments)
        .bind(msg_id)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, ChatMessageRecord>(
            "SELECT id, chat_id, role, content, tokens, duration, speed, model_tags, status, attachments, created_at FROM chat_messages WHERE id = ?",
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_chat_delete_all() {
        let pool = create_test_pool().await;
        let repo = ChatRepository::new(pool);

        let chat1 = repo.insert_chat("chat-1", "Chat 1").await.unwrap();
        let chat2 = repo.insert_chat("chat-2", "Chat 2").await.unwrap();

        let msg_payload = CreateChatMessagePayload {
            id: None,
            role: "user".to_string(),
            content: "Hello".to_string(),
            tokens: None,
            duration: None,
            speed: None,
            model_tags: None,
            status: None,
            attachments: None,
        };
        repo.insert_chat_message(&chat1.id, &msg_payload)
            .await
            .unwrap();
        repo.insert_chat_message(&chat2.id, &msg_payload)
            .await
            .unwrap();

        let chats = repo.get_all_chats().await.unwrap();
        assert_eq!(chats.len(), 2);

        let deleted_count = repo.delete_all_chats().await.unwrap();
        assert_eq!(deleted_count, 2);

        let chats_after = repo.get_all_chats().await.unwrap();
        assert!(chats_after.is_empty());
        let msgs1 = repo.get_chat_messages(&chat1.id).await.unwrap();
        assert!(msgs1.is_empty());
    }
}
