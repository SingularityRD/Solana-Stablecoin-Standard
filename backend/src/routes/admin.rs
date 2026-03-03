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
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::{SeizeRequest, TransactionResponse, User},
    app_middleware::auth::AuthUser,
    solana::explorer_url,
    utils::audit,
    AppState,
};

/// Helper function to convert validation errors to API error
fn validation_error_to_api_error(e: validator::ValidationErrors) -> ApiError {
    let error_messages: Vec<String> = e.field_errors()
        .into_iter()
        .flat_map(|(field, errors)| {
            errors.iter().map(move |err| {
                format!("{}: {}", field, err.message.as_ref().map(|m| m.as_ref()).unwrap_or("invalid"))
            })
        })
        .collect();
    ApiError::Validation(error_messages.join("; "))
}

/// Pause all operations on a stablecoin
pub async fn pause(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    // Get stablecoin and check ownership
    let stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Parse pubkeys
    let stablecoin_pda: Pubkey = stablecoin.stablecoin_pda.parse()
        .map_err(|_| ApiError::Internal("Invalid stablecoin PDA".to_string()))?;
    let authority_pubkey: Pubkey = stablecoin.authority_pubkey.parse()
        .map_err(|_| ApiError::Internal("Invalid authority pubkey".to_string()))?;
    
    // Build and send pause transaction
    let instruction = state.solana.build_pause_instruction(&stablecoin_pda, &authority_pubkey);
    let tx_signature = state.solana.build_and_send_instruction(vec![instruction], &[])
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to pause: {}", e)))?
        .to_string();
    // Log audit
    audit(
        &state.db,
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
    let stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Parse pubkeys
    let stablecoin_pda: Pubkey = stablecoin.stablecoin_pda.parse()
        .map_err(|_| ApiError::Internal("Invalid stablecoin PDA".to_string()))?;
    let authority_pubkey: Pubkey = stablecoin.authority_pubkey.parse()
        .map_err(|_| ApiError::Internal("Invalid authority pubkey".to_string()))?;
    
    // Build and send unpause transaction
    let instruction = state.solana.build_unpause_instruction(&stablecoin_pda, &authority_pubkey);
    let tx_signature = state.solana.build_and_send_instruction(vec![instruction], &[])
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to unpause: {}", e)))?
        .to_string();
    // Log audit
    audit(
        &state.db,
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
    // Validate account pubkey format
    crate::models::validate_solana_pubkey(&account)
        .map_err(|_| ApiError::Validation("Invalid account pubkey".to_string()))?;
    
    // Parse and validate account pubkey (additional validation)
    let _account_pubkey: Pubkey = account.parse()
        .map_err(|_| ApiError::Validation("Invalid account pubkey".to_string()))?;
    
    // Get stablecoin and check ownership
    let stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Parse pubkeys
    let stablecoin_pda: Pubkey = stablecoin.stablecoin_pda.parse()
        .map_err(|_| ApiError::Internal("Invalid stablecoin PDA".to_string()))?;
    let authority_pubkey: Pubkey = stablecoin.authority_pubkey.parse()
        .map_err(|_| ApiError::Internal("Invalid authority pubkey".to_string()))?;
    let asset_mint: Pubkey = stablecoin.asset_mint.parse()
        .map_err(|_| ApiError::Internal("Invalid asset mint".to_string()))?;
    
    // Build and send freeze transaction
    let instruction = state.solana.build_freeze_instruction(
        &stablecoin_pda,
        &asset_mint,
        &authority_pubkey,
        &_account_pubkey,
        None, // role assignment
    );
    let tx_signature = state.solana.build_and_send_instruction(vec![instruction], &[])
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to freeze: {}", e)))?
        .to_string();
    // Log audit
    audit(
        &state.db,
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
    // Validate account pubkey format
    crate::models::validate_solana_pubkey(&account)
        .map_err(|_| ApiError::Validation("Invalid account pubkey".to_string()))?;
    
    // Parse and validate account pubkey (additional validation)
    let _account_pubkey: Pubkey = account.parse()
        .map_err(|_| ApiError::Validation("Invalid account pubkey".to_string()))?;
    
    // Get stablecoin and check ownership
    let stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Parse pubkeys
    let stablecoin_pda: Pubkey = stablecoin.stablecoin_pda.parse()
        .map_err(|_| ApiError::Internal("Invalid stablecoin PDA".to_string()))?;
    let authority_pubkey: Pubkey = stablecoin.authority_pubkey.parse()
        .map_err(|_| ApiError::Internal("Invalid authority pubkey".to_string()))?;
    let asset_mint: Pubkey = stablecoin.asset_mint.parse()
        .map_err(|_| ApiError::Internal("Invalid asset mint".to_string()))?;
    
    // Build and send thaw transaction
    let instruction = state.solana.build_thaw_instruction(
        &stablecoin_pda,
        &asset_mint,
        &authority_pubkey,
        &_account_pubkey,
        None, // role assignment
    );
    let tx_signature = state.solana.build_and_send_instruction(vec![instruction], &[])
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to thaw: {}", e)))?
        .to_string();
    // Log audit
    audit(
        &state.db,
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
    // Validate input using validator crate (includes pubkey and amount validation)
    req.validate().map_err(validation_error_to_api_error)?;
    
    // Parse and validate pubkeys (additional validation)
    let _from_pubkey: Pubkey = req.from_account.parse()
        .map_err(|_| ApiError::Validation("Invalid from_account pubkey".to_string()))?;
    let _to_pubkey: Pubkey = req.to_account.parse()
        .map_err(|_| ApiError::Validation("Invalid to_account pubkey".to_string()))?;
    
    // Get stablecoin and check ownership
    let stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Check SSS-2 preset for seizure
    if stablecoin.preset < 1 {
        return Err(ApiError::BadRequest("Seizure only available for SSS-2 or higher".to_string()));
    }
    
    // Parse pubkeys
    let stablecoin_pda: Pubkey = stablecoin.stablecoin_pda.parse()
        .map_err(|_| ApiError::Internal("Invalid stablecoin PDA".to_string()))?;
    let authority_pubkey: Pubkey = stablecoin.authority_pubkey.parse()
        .map_err(|_| ApiError::Internal("Invalid authority pubkey".to_string()))?;
    let asset_mint: Pubkey = stablecoin.asset_mint.parse()
        .map_err(|_| ApiError::Internal("Invalid asset mint".to_string()))?;
    let from_pubkey: Pubkey = req.from_account.parse()
        .map_err(|_| ApiError::Validation("Invalid from_account pubkey".to_string()))?;
    let to_pubkey: Pubkey = req.to_account.parse()
        .map_err(|_| ApiError::Validation("Invalid to_account pubkey".to_string()))?;
    
    // Build and send seize transaction
    let instruction = state.solana.build_seize_instruction(
        &stablecoin_pda,
        &asset_mint,
        &authority_pubkey,
        &from_pubkey,
        &to_pubkey,
        req.amount,
        None, // role assignment
    );
    let tx_signature = state.solana.build_and_send_instruction(vec![instruction], &[])
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to seize: {}", e)))?
        .to_string();
    // Log audit
    audit(
        &state.db,
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
