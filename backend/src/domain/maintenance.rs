use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MaintenanceResultDto {
    pub success: bool,
    pub message: String,
    pub files_removed: usize,
}

