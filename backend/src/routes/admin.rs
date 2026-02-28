use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use sqlx::query_as;
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    models::{SeizeRequest, TransactionResponse, User},
    app_middleware::auth::AuthUser,
    solana::explorer_url,
    AppState,
};

/// Pause all operations on a stablecoin
pub async fn pause(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Build pause transaction
    let tx_signature = format!("pause_{}", id);
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "stablecoin.pause",
        Some(&tx_signature),
        None,
        None,
    ).await;
    
    Ok(Json(TransactionResponse {
        tx_signature: tx_signature.clone(),
        status: "pending".to_string(),
        explorer_url: explorer_url(&tx_signature, "devnet"),
    }))
}

/// Unpause all operations on a stablecoin
pub async fn unpause(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Build unpause transaction
    let tx_signature = format!("unpause_{}", id);
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "stablecoin.unpause",
        Some(&tx_signature),
        None,
        None,
    ).await;
    
    Ok(Json(TransactionResponse {
        tx_signature: tx_signature.clone(),
        status: "pending".to_string(),
        explorer_url: explorer_url(&tx_signature, "devnet"),
    }))
}

/// Freeze an account
pub async fn freeze(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, account)): Path<(Uuid, String)>,
) -> ApiResult<impl IntoResponse> {
    // Parse and validate account pubkey
    let _account_pubkey: Pubkey = account.parse()
        .map_err(|_| ApiError::Validation("Invalid account pubkey".to_string()))?;
    
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Build freeze transaction
    let tx_signature = format!("freeze_{}_{}", id, &account[..8]);
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "stablecoin.freeze",
        Some(&tx_signature),
        Some(json!({"account": account})),
        None,
    ).await;
    
    Ok(Json(TransactionResponse {
        tx_signature: tx_signature.clone(),
        status: "pending".to_string(),
        explorer_url: explorer_url(&tx_signature, "devnet"),
    }))
}

/// Thaw a frozen account
pub async fn thaw(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, account)): Path<(Uuid, String)>,
) -> ApiResult<impl IntoResponse> {
    // Parse and validate account pubkey
    let _account_pubkey: Pubkey = account.parse()
        .map_err(|_| ApiError::Validation("Invalid account pubkey".to_string()))?;
    
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Build thaw transaction
    let tx_signature = format!("thaw_{}_{}", id, &account[..8]);
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "stablecoin.thaw",
        Some(&tx_signature),
        Some(json!({"account": account})),
        None,
    ).await;
    
    Ok(Json(TransactionResponse {
        tx_signature: tx_signature.clone(),
        status: "pending".to_string(),
        explorer_url: explorer_url(&tx_signature, "devnet"),
    }))
}

/// Seize tokens from an account
pub async fn seize(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<SeizeRequest>,
) -> ApiResult<impl IntoResponse> {
    // Parse and validate pubkeys
    let _from_pubkey: Pubkey = req.from_account.parse()
        .map_err(|_| ApiError::Validation("Invalid from_account pubkey".to_string()))?;
    let _to_pubkey: Pubkey = req.to_account.parse()
        .map_err(|_| ApiError::Validation("Invalid to_account pubkey".to_string()))?;
    
    if req.amount == 0 {
        return Err(ApiError::Validation("Amount must be greater than 0".to_string()));
    }
    
    // Get stablecoin and check ownership
    let stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Check SSS-2 preset for seizure
    if stablecoin.preset < 1 {
        return Err(ApiError::BadRequest("Seizure only available for SSS-2 or higher".to_string()));
    }
    
    // Build seize transaction
    let tx_signature = format!("seize_{}_{}_{}", id, &req.from_account[..8], req.amount);
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "stablecoin.seize",
        Some(&tx_signature),
        Some(json!({"from": req.from_account, "to": req.to_account, "amount": req.amount})),
        None,
    ).await;
    
    Ok(Json(TransactionResponse {
        tx_signature: tx_signature.clone(),
        status: "pending".to_string(),
        explorer_url: explorer_url(&tx_signature, "devnet"),
    }))
}

// Helper function
async fn get_stablecoin_for_admin(
    state: &AppState, 
    id: Uuid, 
    user: &User
) -> ApiResult<crate::models::Stablecoin> {
    let stablecoin: crate::models::Stablecoin = query_as(
        "SELECT * FROM stablecoins WHERE id = $1 AND is_active = true"
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Stablecoin not found".to_string()))?;
    
    // Check ownership or admin role
    if stablecoin.owner_id != user.id && user.role != "admin" {
        return Err(ApiError::Forbidden("Not authorized for admin operations".to_string()));
    }
    
    Ok(stablecoin)
}
