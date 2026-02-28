use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::query_as;
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    models::{CreateWebhookRequest, User, Webhook},
    app_middleware::auth::AuthUser,
    AppState,
};

/// Handle incoming webhook events (from external services)
pub async fn handler(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> ApiResult<impl IntoResponse> {
    // Process incoming webhook
    tracing::info!("Received webhook: {:?}", payload);
    
    // In production, this would verify the webhook signature and process events
    // from external services like Chainalysis, banking APIs, etc.
    
    Ok((StatusCode::OK, Json(json!({"status": "received"}))))
}

/// Create a new webhook subscription
pub async fn create(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateWebhookRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate URL
    if req.url.is_empty() || !req.url.starts_with("http") {
        return Err(ApiError::Validation("Invalid webhook URL".to_string()));
    }
    
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Create webhook
    let webhook: Webhook = query_as(
        r#"
        INSERT INTO webhooks (stablecoin_id, url, events, secret)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#
    )
    .bind(id)
    .bind(&req.url)
    .bind(serde_json::to_value(&req.events).unwrap_or(json!([])))
    .bind(&req.secret)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "webhook.create",
        None,
        Some(json!({"url": req.url})),
        None,
    ).await;
    
    Ok((StatusCode::CREATED, Json(webhook)))
}

/// List all webhooks for a stablecoin
pub async fn list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    let webhooks: Vec<Webhook> = query_as(
        "SELECT * FROM webhooks WHERE stablecoin_id = $1 AND is_active = true ORDER BY created_at DESC"
    )
    .bind(id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    Ok(Json(webhooks))
}

/// Delete a webhook
pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, webhook_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<impl IntoResponse> {
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Delete webhook
    let result = sqlx::query(
        "UPDATE webhooks SET is_active = false WHERE id = $1 AND stablecoin_id = $2"
    )
    .bind(webhook_id)
    .bind(id)
    .execute(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "webhook.delete",
        None,
        Some(json!({"webhook_id": webhook_id})),
        None,
    ).await;
    
    Ok(StatusCode::NO_CONTENT)
}

// Helper function
async fn get_stablecoin_for_admin(
    state: &AppState, 
    id: Uuid, 
    user: &User
) -> ApiResult<crate::models::Stablecoin> {
    let stablecoin: crate::models::Stablecoin = sqlx::query_as(
        "SELECT * FROM stablecoins WHERE id = $1 AND is_active = true"
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Stablecoin not found".to_string()))?;
    
    // Check ownership or admin role
    if stablecoin.owner_id != user.id && user.role != "admin" {
        return Err(ApiError::Forbidden("Not authorized for webhook management".to_string()));
    }
    
    Ok(stablecoin)
}