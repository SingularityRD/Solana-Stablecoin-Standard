use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::query_as;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::{AuthResponse, LoginRequest, RegisterRequest, User, UserPublic},
    AppState,
};
use crate::utils::{generate_tokens, hash_password, verify_password, is_valid_email};

/// Register a new user
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate input
    if !is_valid_email(&req.email) {
        return Err(ApiError::Validation("Invalid email format".to_string()));
    }
    
    if req.password.len() < 8 {
        return Err(ApiError::Validation("Password must be at least 8 characters".to_string()));
    }
    
    // Check if user already exists
    let existing: Option<User> = query_as(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&req.email)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    if existing.is_some() {
        return Err(ApiError::Conflict("Email already registered".to_string()));
    }
    
    // Hash password
    let password_hash = hash_password(&req.password)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    // Create user
    let user: User = query_as(
        r#"
        INSERT INTO users (email, password_hash, solana_pubkey, role)
        VALUES ($1, $2, $3, 'user')
        RETURNING *
        "#
    )
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.solana_pubkey)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    // Generate tokens
    let tokens = generate_tokens(
        user.id,
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_expiry,
    ).map_err(|e| ApiError::Internal(e.to_string()))?;
    
    // Log audit
    let _ = state.db.log_audit(
        None,
        Some(user.id),
        "user.register",
        None,
        Some(json!({"email": user.email})),
        None,
    ).await;
    
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            token_type: tokens.token_type,
            expires_in: tokens.expires_in,
            user: UserPublic::from(user),
        }),
    ))
}

/// Login with email and password
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<impl IntoResponse> {
    // Find user
    let user: Option<User> = query_as(
        "SELECT * FROM users WHERE email = $1 AND is_active = true"
    )
    .bind(&req.email)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    let user = user.ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;
    
    // Verify password
    let valid = verify_password(&req.password, &user.password_hash)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    if !valid {
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }
    
    // Generate tokens
    let tokens = generate_tokens(
        user.id,
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_expiry,
    ).map_err(|e| ApiError::Internal(e.to_string()))?;
    
    // Log audit
    let _ = state.db.log_audit(
        None,
        Some(user.id),
        "user.login",
        None,
        None,
        None,
    ).await;
    
    Ok(Json(AuthResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: tokens.token_type,
        expires_in: tokens.expires_in,
        user: UserPublic::from(user),
    }))
}

/// Refresh access token
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> ApiResult<impl IntoResponse> {
    use crate::utils::validate_token;
    
    // Validate refresh token
    let claims = validate_token(&req.refresh_token, &state.config.jwt_secret)
        .map_err(|_| ApiError::Unauthorized("Invalid refresh token".to_string()))?;
    
    // Get user
    let user: User = query_as(
        "SELECT * FROM users WHERE id = $1 AND is_active = true"
    )
    .bind(claims.sub)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::Unauthorized("User not found".to_string()))?;
    
    // Generate new tokens
    let tokens = generate_tokens(
        user.id,
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_expiry,
    ).map_err(|e| ApiError::Internal(e.to_string()))?;
    
    Ok(Json(AuthResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: tokens.token_type,
        expires_in: tokens.expires_in,
        user: UserPublic::from(user),
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
