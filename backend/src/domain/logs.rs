use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LogEntry {
    pub id: u64,
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, Deserialize, utoipa::IntoParams)]
pub struct LogFilterQuery {
    pub service: Option<String>,
    pub level: Option<String>,
    pub search: Option<String>,
    pub limit: Option<usize>,
}
