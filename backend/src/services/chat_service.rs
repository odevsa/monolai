use crate::core::error::AppResult;
use crate::domain::{
    ChatMessageRecord, ChatRecord, CreateChatMessagePayload, CreateChatPayload,
    UpdateChatMessagePayload, UpdateChatPayload,
};
use crate::infrastructure::db::ChatRepository;

#[derive(Clone)]
pub struct ChatService {
    repo: ChatRepository,
}

impl ChatService {
    pub fn new(repo: ChatRepository) -> Self {
        Self { repo }
    }

    pub async fn get_all_chats(&self) -> AppResult<Vec<ChatRecord>> {
        self.repo.get_all_chats().await
    }

    pub async fn create_chat(&self, payload: CreateChatPayload) -> AppResult<ChatRecord> {
        let id = payload
            .id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let title = payload.title.unwrap_or_else(|| "New Chat".to_string());
        self.repo.insert_chat(&id, &title).await
    }

    pub async fn update_chat(&self, id: &str, payload: UpdateChatPayload) -> AppResult<ChatRecord> {
        self.repo.update_chat_title(id, &payload.title).await
    }

    pub async fn delete_chat(&self, id: &str) -> AppResult<()> {
        self.repo.delete_chat(id).await
    }

    pub async fn get_chat_messages(&self, chat_id: &str) -> AppResult<Vec<ChatMessageRecord>> {
        self.repo.get_chat_messages(chat_id).await
    }

    pub async fn create_chat_message(
        &self,
        chat_id: &str,
        payload: CreateChatMessagePayload,
    ) -> AppResult<ChatMessageRecord> {
        self.repo.insert_chat_message(chat_id, &payload).await
    }

    pub async fn update_chat_message(
        &self,
        msg_id: &str,
        payload: UpdateChatMessagePayload,
    ) -> AppResult<ChatMessageRecord> {
        self.repo.update_chat_message(msg_id, &payload).await
    }

    pub async fn delete_chat_message(&self, msg_id: &str) -> AppResult<()> {
        self.repo.delete_chat_message(msg_id).await
    }

    pub async fn clear_chat_messages(&self, chat_id: &str) -> AppResult<()> {
        self.repo.clear_chat_messages(chat_id).await
    }
}
