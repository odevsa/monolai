use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Upstream error: {0}")]
    Upstream(String),

    #[error("Upstream connection error: {0}")]
    BadGateway(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("OpenAI proxy error ({status}): {message}")]
    OpenAi {
        status: StatusCode,
        message: String,
        error_type: String,
        code: String,
    },
}

#[allow(dead_code)]
impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn bad_gateway(msg: impl Into<String>) -> Self {
        Self::BadGateway(msg.into())
    }

    pub fn openai(
        status: StatusCode,
        message: impl Into<String>,
        error_type: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self::OpenAi {
            status,
            message: message.into(),
            error_type: error_type.into(),
            code: code.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::OpenAi {
                status,
                message,
                error_type,
                code,
            } => {
                let body = json!({
                    "error": {
                        "message": message,
                        "type": error_type,
                        "param": serde_json::Value::Null,
                        "code": code
                    }
                });
                (status, Json(body)).into_response()
            }
            Self::Validation(msg) => {
                let body = json!({
                    "error": msg,
                    "code": "validation_error"
                });
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
            }
            Self::NotFound(msg) => {
                let body = json!({
                    "error": msg,
                    "code": "not_found"
                });
                (StatusCode::NOT_FOUND, Json(body)).into_response()
            }
            Self::Conflict(msg) => {
                let body = json!({
                    "error": msg,
                    "code": "conflict"
                });
                (StatusCode::CONFLICT, Json(body)).into_response()
            }
            Self::Database(err) => {
                tracing::error!("Database error: {}", err);
                let body = json!({
                    "error": err.to_string(),
                    "code": "database_error"
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
            }
            Self::Io(err) => {
                tracing::error!("I/O error: {}", err);
                let body = json!({
                    "error": err.to_string(),
                    "code": "io_error"
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
            }
            Self::Upstream(msg) => {
                let body = json!({
                    "error": msg,
                    "code": "upstream_error"
                });
                (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
            }
            Self::BadGateway(msg) => {
                let body = json!({
                    "error": msg,
                    "code": "bad_gateway"
                });
                (StatusCode::BAD_GATEWAY, Json(body)).into_response()
            }
            Self::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                let body = json!({
                    "error": msg,
                    "code": "internal_error"
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
            }
        }
    }
}
