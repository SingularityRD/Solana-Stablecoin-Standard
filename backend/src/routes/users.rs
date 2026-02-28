use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use sqlx::query_as;

use crate::{
    error::{ApiError, ApiResult},
    models::{User, UserPublic},
    app_middleware::auth::AuthUser,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub solana_pubkey: Option<String>,
}

/// Get current user profile
pub async fn me(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> ApiResult<impl IntoResponse> {
    let user: User = query_as(
        "SELECT * FROM users WHERE id = $1 AND is_active = true"
    )
    .bind(user.id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
    
    Ok(Json(UserPublic::from(user)))
}

/// Update current user profile
pub async fn update(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<UpdateUserRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate solana pubkey if provided
    if let Some(ref pubkey) = req.solana_pubkey {
        if !pubkey.is_empty() {
            pubkey.parse::<Pubkey>()
                .map_err(|_| ApiError::Validation("Invalid Solana pubkey".to_string()))?;
        }
    }
    
    let user: User = query_as(
        r#"
        UPDATE users
        SET solana_pubkey = $1, updated_at = NOW()
        WHERE id = $2 AND is_active = true
        RETURNING *
        "#
    )
    .bind(&req.solana_pubkey)
    .bind(user.id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
    
    Ok(Json(UserPublic::from(user)))
}
