use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
    pub attachments: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateChatPayload {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateChatPayload {
    pub title: String,
}

fn deserialize_attachments<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<serde_json::Value>::deserialize(deserializer)?;
    match opt {
        Some(serde_json::Value::String(s)) => Ok(Some(s)),
        Some(serde_json::Value::Null) | None => Ok(None),
        Some(val) => Ok(Some(val.to_string())),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateChatMessagePayload {
    #[serde(default)]
    pub id: Option<String>,
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub tokens: Option<i64>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub speed: Option<String>,
    #[serde(default)]
    pub model_tags: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, deserialize_with = "deserialize_attachments")]
    pub attachments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateChatMessagePayload {
    pub content: String,
    #[serde(default)]
    pub tokens: Option<i64>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub speed: Option<String>,
    #[serde(default)]
    pub model_tags: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, deserialize_with = "deserialize_attachments")]
    pub attachments: Option<String>,
}
