use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("create chat error: {0}")]
    CreateChatError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("email `{0}` already exists")]
    EmailAlreadyExists(String),

    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("general error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    #[error("http header parser error: {0}")]
    HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Self::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::AnyhowError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::HttpHeaderError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::CreateChatError(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}
