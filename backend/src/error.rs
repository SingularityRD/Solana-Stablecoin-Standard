use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Solana error: {0}")]
    Solana(String),
    
    #[error("Rate limit exceeded")]
    RateLimited,
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            },
            ApiError::Solana(msg) => {
                tracing::error!("Solana error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Blockchain error".to_string())
            },
            ApiError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()),
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
        };

        let body = Json(json!({
            "error": {
                "code": status.as_u16(),
                "message": error_message,
            }
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.constraint().is_some() {
                    ApiError::Conflict(db_err.message().to_string())
                } else {
                    ApiError::Database(db_err.message().to_string())
                }
            },
            _ => ApiError::Database(err.to_string()),
        }
    }
}

impl From<argon2::password_hash::Error> for ApiError {
    fn from(err: argon2::password_hash::Error) -> Self {
        ApiError::Internal(format!("Password hashing error: {}", err))
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        ApiError::Unauthorized(err.to_string())
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
