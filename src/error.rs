use actix_web::{HttpResponse, ResponseError};
use actix_identity::error::LoginError;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("UUID parse error: {0}")]
    UuidParse(#[from] uuid::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Login error: {0}")]
    Login(#[from] LoginError),

    #[error("Authentication required")]
    AuthRequired,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error")]
    InternalServerError,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, message) = match self {
            AppError::Database(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            AppError::UuidParse(_) => (actix_web::http::StatusCode::BAD_REQUEST, "Invalid UUID format".to_string()),
            AppError::Serde(_) => (actix_web::http::StatusCode::BAD_REQUEST, "Invalid JSON".to_string()),
            AppError::Login(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "Login error".to_string()),
            AppError::AuthRequired => (actix_web::http::StatusCode::UNAUTHORIZED, "Authentication required".to_string()),
            AppError::InsufficientPermissions => (actix_web::http::StatusCode::FORBIDDEN, "Insufficient permissions".to_string()),
            AppError::InvalidInput(msg) => (actix_web::http::StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotFound(msg) => (actix_web::http::StatusCode::NOT_FOUND, msg.clone()),
            AppError::InternalServerError => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let error_response = ErrorResponse {
            error: status.to_string(),
            message: message,
        };

        HttpResponse::build(status).json(error_response)
    }
}

pub type AppResult<T> = Result<T, AppError>;