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
    models::{BurnRequest, MintRequest, TransactionResponse, TransferRequest, User},
    app_middleware::auth::AuthUser,
    solana::explorer_url,
    AppState,
};

/// Mint tokens to a recipient
pub async fn mint(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<MintRequest>,
) -> ApiResult<impl IntoResponse> {
    // Parse and validate recipient pubkey
    let recipient: Pubkey = req.recipient.parse()
        .map_err(|_| ApiError::Validation("Invalid recipient pubkey".to_string()))?;
    
    if req.amount == 0 {
        return Err(ApiError::Validation("Amount must be greater than 0".to_string()));
    }
    
    // Get stablecoin
    let stablecoin = get_stablecoin(&state, id).await?;
    
    // Check if user has minter role
    let has_role = check_role(&state, id, &user).await?;
    if !has_role && user.role != "admin" {
        return Err(ApiError::Forbidden("Not authorized to mint".to_string()));
    }
    
    // Parse stablecoin PDA
    let stablecoin_pda: Pubkey = stablecoin.stablecoin_pda.parse()
        .map_err(|_| ApiError::Internal("Invalid stablecoin PDA".to_string()))?;
    
    // Build mint transaction
    // In production, this would use Anchor client to build and send the transaction
    let tx_signature = format!("mint_{}_{}_{}", id, recipient, req.amount);
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "stablecoin.mint",
        Some(&tx_signature),
        Some(json!({"recipient": req.recipient, "amount": req.amount})),
        None,
    ).await;
    
    Ok(Json(TransactionResponse {
        tx_signature: tx_signature.clone(),
        status: "pending".to_string(),
        explorer_url: explorer_url(&tx_signature, "devnet"),
    }))
}

/// Burn tokens from an account
pub async fn burn(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<BurnRequest>,
) -> ApiResult<impl IntoResponse> {
    if req.amount == 0 {
        return Err(ApiError::Validation("Amount must be greater than 0".to_string()));
    }
    
    // Get stablecoin
    let _stablecoin = get_stablecoin(&state, id).await?;
    
    // Build burn transaction
    let tx_signature = format!("burn_{}_{}", id, req.amount);
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "stablecoin.burn",
        Some(&tx_signature),
        Some(json!({"amount": req.amount, "from_account": req.from_account})),
        None,
    ).await;
    
    Ok(Json(TransactionResponse {
        tx_signature: tx_signature.clone(),
        status: "pending".to_string(),
        explorer_url: explorer_url(&tx_signature, "devnet"),
    }))
}

/// Transfer tokens between accounts
pub async fn transfer(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<TransferRequest>,
) -> ApiResult<impl IntoResponse> {
    // Parse and validate pubkeys
    let _from: Pubkey = req.from.parse()
        .map_err(|_| ApiError::Validation("Invalid from pubkey".to_string()))?;
    let _to: Pubkey = req.to.parse()
        .map_err(|_| ApiError::Validation("Invalid to pubkey".to_string()))?;
    
    if req.amount == 0 {
        return Err(ApiError::Validation("Amount must be greater than 0".to_string()));
    }
    
    // Get stablecoin
    let _stablecoin = get_stablecoin(&state, id).await?;
    
    // Build transfer transaction
    let tx_signature = format!("transfer_{}_{}_{}", id, &req.from[..8], &req.to[..8]);
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "stablecoin.transfer",
        Some(&tx_signature),
        Some(json!({"from": req.from, "to": req.to, "amount": req.amount})),
        None,
    ).await;
    
    Ok(Json(TransactionResponse {
        tx_signature: tx_signature.clone(),
        status: "pending".to_string(),
        explorer_url: explorer_url(&tx_signature, "devnet"),
    }))
}

// Helper functions
async fn get_stablecoin(state: &AppState, id: Uuid) -> ApiResult<crate::models::Stablecoin> {
    query_as(
        "SELECT * FROM stablecoins WHERE id = $1 AND is_active = true"
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Stablecoin not found".to_string()))
}

async fn check_role(state: &AppState, stablecoin_id: Uuid, user: &User) -> ApiResult<bool> {
    let pubkey = match user.solana_pubkey.as_deref() {
        Some(pk) => pk,
        None => return Ok(false),
    };
    
    let role: Option<crate::models::RoleAssignment> = query_as(
        "SELECT * FROM role_assignments WHERE stablecoin_id = $1 AND account_pubkey = $2 AND role = 'minter'"
    )
    .bind(stablecoin_id)
    .bind(pubkey)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    Ok(role.is_some())
}
