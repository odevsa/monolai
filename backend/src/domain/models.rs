use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct ModelRecord {
    pub id: String,
    pub runtime: String,
    pub flags: String,
    pub created_at: String,
    #[sqlx(default)]
    pub file_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateModelPayload {
    pub id: String,
    pub runtime: String,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub flags: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ModelItem {
    pub name: String,
    pub filename: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub format: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(tag = "status", content = "message")]
pub enum ModelState {
    Idle,
    Loading,
    Ready,
    Error(String),
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct RunningModelStatus {
    pub model_id: String,
    pub runtime_id: String,
    pub pid: u32,
    pub port: u16,
    pub state: ModelState,
    pub idle_seconds: u64,
}
