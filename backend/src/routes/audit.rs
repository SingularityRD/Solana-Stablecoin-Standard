use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::query_as;
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    models::{AuditLogEntry, User},
    app_middleware::auth::AuthUser,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub action: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List audit logs for a stablecoin
pub async fn list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Query(query): Query<AuditQuery>,
) -> ApiResult<impl IntoResponse> {
    // Check stablecoin ownership
    let stablecoin: crate::models::Stablecoin = query_as(
        "SELECT * FROM stablecoins WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Stablecoin not found".to_string()))?;
    
    if stablecoin.owner_id != user.id && user.role != "admin" {
        return Err(ApiError::Forbidden("Not authorized to view audit logs".to_string()));
    }
    
    let limit = query.limit.unwrap_or(100).min(1000);
    let offset = query.offset.unwrap_or(0);
    
    let logs: Vec<AuditLogEntry> = if let Some(action) = &query.action {
        query_as(
            r#"
            SELECT * FROM audit_log 
            WHERE stablecoin_id = $1 AND action = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#
        )
        .bind(id)
        .bind(action)
        .bind(limit)
        .bind(offset)
        .fetch_all(state.db.pool())
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?
    } else {
        query_as(
            r#"
            SELECT * FROM audit_log 
            WHERE stablecoin_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(id)
        .bind(limit)
        .bind(offset)
        .fetch_all(state.db.pool())
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?
    };
    
    Ok(Json(logs))
}

/// Get a specific audit log entry by transaction signature
pub async fn get(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(tx_signature): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let log: AuditLogEntry = query_as(
        "SELECT * FROM audit_log WHERE tx_signature = $1"
    )
    .bind(&tx_signature)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Audit log entry not found".to_string()))?;
    
    // Check if user has access to this stablecoin's logs
    if let Some(stablecoin_id) = log.stablecoin_id {
        let stablecoin: crate::models::Stablecoin = query_as(
            "SELECT * FROM stablecoins WHERE id = $1"
        )
        .bind(stablecoin_id)
        .fetch_one(state.db.pool())
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;
        
        if stablecoin.owner_id != user.id && user.role != "admin" {
            return Err(ApiError::Forbidden("Not authorized to view this audit log".to_string()));
        }
    }
    
    Ok(Json(log))
}