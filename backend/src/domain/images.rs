use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedImageRecord {
    pub id: String,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub model: String,
    pub size: String,
    pub steps: i64,
    pub cfg_scale: f64,
    pub seed: Option<i64>,
    pub src: String,
    pub generation_time_secs: Option<f64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateGeneratedImagePayload {
    #[serde(default)]
    pub id: Option<String>,
    pub prompt: String,
    #[serde(default)]
    pub negative_prompt: Option<String>,
    pub model: String,
    pub size: String,
    pub steps: i64,
    pub cfg_scale: f64,
    #[serde(default)]
    pub seed: Option<i64>,
    pub src: String,
    #[serde(default)]
    pub generation_time_secs: Option<f64>,
}

