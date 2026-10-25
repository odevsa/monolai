use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct ModelPreset {
    pub id: String,
    pub model_id: String,
    pub name: String,
    pub runtime_id: String,
    pub flags: String,
    pub is_default: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct ModelRecord {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub relative_path: String,
    pub size_bytes: i64,
    pub format: String,
    pub modality: String,
    pub architecture: Option<String>,
    pub source_repo: Option<String>,
    pub created_at: String,
    #[sqlx(default, skip)]
    pub presets: Vec<ModelPreset>,
    #[sqlx(default, skip)]
    pub file_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreatePresetPayload {
    pub name: String,
    pub runtime_id: String,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub flags: serde_json::Value,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdatePresetPayload {
    pub name: Option<String>,
    pub runtime_id: Option<String>,
    #[schema(value_type = Option<Object>)]
    pub flags: Option<serde_json::Value>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct RepositoryRecord {
    pub id: String,
    pub name: String,
    pub repo_type: String,
    pub base_url: String,
    pub api_token: Option<String>,
    pub enabled: bool,
    pub is_builtin: bool,
    pub config_json: String,
    pub created_at: String,
}

impl RepositoryRecord {
    pub fn get_mirror_url(&self) -> Option<String> {
        serde_json::from_str::<serde_json::Value>(&self.config_json)
            .ok()
            .and_then(|v| v.get("mirror_url").and_then(|m| m.as_str()).map(|s| s.to_string()))
            .filter(|s| !s.trim().is_empty())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateRepositoryPayload {
    pub id: String,
    pub name: String,
    pub repo_type: String,
    pub base_url: String,
    pub api_token: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    pub mirror_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateRepositoryPayload {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub api_token: Option<String>,
    pub enabled: Option<bool>,
    pub mirror_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct DownloadRecord {
    pub id: String,
    pub model_name: String,
    pub url: String,
    pub file_path: String,
    pub total_bytes: i64,
    pub downloaded_bytes: i64,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[sqlx(default, skip)]
    pub speed_bytes_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EnqueueDownloadPayload {
    #[serde(default)]
    pub url: Option<String>,
    pub model_name: String,
    pub filename: String,
    pub modality: Option<String>,
    pub repo_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HubModelSibling {
    pub rfilename: String,
    pub size: Option<u64>,
    #[serde(default)]
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HubModelItem {
    pub id: String,
    pub author: Option<String>,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
    pub pipeline_tag: Option<String>,
    pub modality: String,
    pub siblings: Vec<HubModelSibling>,
    #[serde(default)]
    pub is_gated: bool,
    #[serde(default)]
    pub requires_auth: bool,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ModelItem {
    pub name: String,
    pub filename: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub format: String,
    pub modality: String,
    pub architecture: Option<String>,
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
