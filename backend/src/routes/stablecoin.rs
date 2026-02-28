use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use sqlx::query_as;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{ApiError, ApiResult},
    models::{
        CreateStablecoinRequest, Stablecoin, StablecoinStatus, UpdateStablecoinRequest,
    },
    app_middleware::auth::AuthUser,
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

/// Create a new stablecoin
pub async fn create(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateStablecoinRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate input using validator crate
    req.validate().map_err(validation_error_to_api_error)?;
    
    // Parse and validate asset mint (additional validation beyond base58 format)
    let asset_mint: Pubkey = req.asset_mint.parse()
        .map_err(|_| ApiError::Validation("Invalid asset_mint pubkey".to_string()))?;
    
    // Find stablecoin PDA
    let (stablecoin_pda, _bump) = state.solana.find_stablecoin_pda(&asset_mint);
    
    // Check if stablecoin already exists
    let existing: Option<Stablecoin> = query_as(
        "SELECT * FROM stablecoins WHERE stablecoin_pda = $1"
    )
    .bind(stablecoin_pda.to_string())
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    if existing.is_some() {
        return Err(ApiError::Conflict("Stablecoin already exists for this asset".to_string()));
    }
    
    // Generate authority keypair
    let authority_keypair = solana_sdk::signature::Keypair::new();
    let authority_pubkey = authority_keypair.pubkey().to_string();
    
    // Create stablecoin in database
    let stablecoin: Stablecoin = query_as(
        r#"
        INSERT INTO stablecoins (
            owner_id, name, symbol, decimals, preset, asset_mint, 
            stablecoin_pda, authority_pubkey
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#
    )
    .bind(user.id)
    .bind(&req.name)
    .bind(&req.symbol)
    .bind(req.decimals.unwrap_or(6) as i16)
    .bind(req.preset as i16)
    .bind(&req.asset_mint)
    .bind(stablecoin_pda.to_string())
    .bind(&authority_pubkey)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    // Log audit
    let _ = state.db.log_audit(
        Some(stablecoin.id),
        Some(user.id),
        "stablecoin.create",
        None,
        Some(json!({"name": req.name, "symbol": req.symbol, "preset": req.preset})),
        None,
    ).await;
    
    Ok((StatusCode::CREATED, Json(stablecoin)))
}

/// Get stablecoin by ID
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    let stablecoin: Stablecoin = query_as(
        "SELECT * FROM stablecoins WHERE id = $1 AND is_active = true"
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Stablecoin not found".to_string()))?;
    
    Ok(Json(stablecoin))
}

/// Update stablecoin
pub async fn update(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStablecoinRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate input using validator crate
    req.validate().map_err(validation_error_to_api_error)?;
    
    // Check ownership
    let existing: Stablecoin = query_as(
        "SELECT * FROM stablecoins WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Stablecoin not found".to_string()))?;
    
    if existing.owner_id != user.id && user.role != "admin" {
        return Err(ApiError::Forbidden("Not authorized to update this stablecoin".to_string()));
    }
    
    // Update
    let stablecoin: Stablecoin = query_as(
        r#"
        UPDATE stablecoins
        SET name = COALESCE($1, name),
            is_active = COALESCE($2, is_active),
            updated_at = NOW()
        WHERE id = $3
        RETURNING *
        "#
    )
    .bind(&req.name)
    .bind(req.is_active)
    .bind(id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    Ok(Json(stablecoin))
}

/// List all stablecoins for user
pub async fn list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> ApiResult<impl IntoResponse> {
    let stablecoins: Vec<Stablecoin> = if user.role == "admin" {
        query_as("SELECT * FROM stablecoins WHERE is_active = true ORDER BY created_at DESC")
            .fetch_all(state.db.pool())
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?
    } else {
        query_as("SELECT * FROM stablecoins WHERE owner_id = $1 AND is_active = true ORDER BY created_at DESC")
            .bind(user.id)
            .fetch_all(state.db.pool())
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?
    };
    
    Ok(Json(stablecoins))
}

/// Get stablecoin status (with on-chain data)
pub async fn status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    let stablecoin: Stablecoin = query_as(
        "SELECT * FROM stablecoins WHERE id = $1 AND is_active = true"
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Stablecoin not found".to_string()))?;
    
    // Parse stablecoin PDA
    let stablecoin_pda: Pubkey = stablecoin.stablecoin_pda.parse()
        .map_err(|_| ApiError::Internal("Invalid stablecoin PDA".to_string()))?;
    
    // Fetch on-chain state
    let account_info = state.solana.rpc_client()
        .get_account(&stablecoin_pda)
        .ok();
    
    let (total_supply, paused, holder_count) = if let Some(account) = account_info {
        // Parse on-chain state (simplified)
        let data = &account.data;
        let paused = data.len() > 0 && data[0] == 1;
        let total_supply = if data.len() >= 12 {
            u64::from_le_bytes(data[4..12].try_into().unwrap_or([0u8; 8]))
        } else {
            0
        };
        (total_supply, paused, 0)
    } else {
        (0, false, 0)
    };
    
    let status = StablecoinStatus {
        stablecoin,
        total_supply,
        paused,
        compliance_enabled: false,
        holder_count,
    };
    
    Ok(Json(status))
}
