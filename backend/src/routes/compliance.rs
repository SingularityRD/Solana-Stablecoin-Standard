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
    models::{BlacklistAddRequest, BlacklistEntry, ScreeningResult, User},
    app_middleware::auth::AuthUser,
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

/// Add an account to the blacklist
pub async fn blacklist_add(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<BlacklistAddRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate input using validator crate
    req.validate().map_err(validation_error_to_api_error)?;
    
    // Parse and validate account pubkey (additional validation)
    let account_pubkey: Pubkey = req.account.parse()
        .map_err(|_| ApiError::Validation("Invalid account pubkey".to_string()))?;
    
    // Get stablecoin and check ownership
    let stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Parse stablecoin PDA
    let stablecoin_pda: Pubkey = stablecoin.stablecoin_pda.parse()
        .map_err(|_| ApiError::Internal("Invalid stablecoin PDA".to_string()))?;
    
    // Find blacklist PDA
    let (blacklist_pda, _bump) = state.solana.find_blacklist_pda(&stablecoin_pda, &account_pubkey);
    
    // Add to blacklist in database
    let entry: BlacklistEntry = query_as(
        r#"
        INSERT INTO blacklist_entries (stablecoin_id, account_pubkey, reason, blacklisted_by)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (stablecoin_id, account_pubkey)
        DO UPDATE SET is_active = true, reason = $3
        RETURNING *
        "#
    )
    .bind(id)
    .bind(&req.account)
    .bind(&req.reason)
    .bind(user.id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    // Log audit
    audit(
        &state.db,
        Some(id),
        Some(user.id),
        "blacklist.add",
        None,
        Some(json!({"account": req.account, "reason": req.reason, "pda": blacklist_pda.to_string()})),
        None,
    ).await;
    
    Ok((StatusCode::CREATED, Json(entry)))
}

/// Remove an account from the blacklist
pub async fn blacklist_remove(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, account)): Path<(Uuid, String)>,
) -> ApiResult<impl IntoResponse> {
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Deactivate blacklist entry
    let result = sqlx::query(
        "UPDATE blacklist_entries SET is_active = false WHERE stablecoin_id = $1 AND account_pubkey = $2"
    )
    .bind(id)
    .bind(&account)
    .execute(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Blacklist entry not found".to_string()));
    }
    
    // Log audit
    audit(
        &state.db,
        Some(id),
        Some(user.id),
        "blacklist.remove",
        None,
        Some(json!({"account": account})),
        None,
    ).await;
    
    Ok(StatusCode::NO_CONTENT)
}

/// List all blacklisted accounts for a stablecoin
pub async fn blacklist_list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    let entries: Vec<BlacklistEntry> = query_as(
        "SELECT * FROM blacklist_entries WHERE stablecoin_id = $1 AND is_active = true ORDER BY created_at DESC"
    )
    .bind(id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    Ok(Json(entries))
}

/// Screen an address for compliance
pub async fn screen(
    State(state): State<AppState>,
    Path((id, address)): Path<(Uuid, String)>,
) -> ApiResult<impl IntoResponse> {
    // Validate address using the model's pubkey validation
    crate::models::validate_solana_pubkey(&address)
        .map_err(|_| ApiError::Validation("Invalid address pubkey".to_string()))?;
    
    // Parse and validate address (additional validation)
    let _address_pubkey: Pubkey = address.parse()
        .map_err(|_| ApiError::Validation("Invalid address pubkey".to_string()))?;
    
    // Check if address is in blacklist
    let blacklisted: Option<BlacklistEntry> = query_as(
        "SELECT * FROM blacklist_entries WHERE stablecoin_id = $1 AND account_pubkey = $2 AND is_active = true"
    )
    .bind(id)
    .bind(&address)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    let result = ScreeningResult {
        address: address.clone(),
        risk_score: if blacklisted.is_some() { 100 } else { 0 },
        is_sanctioned: false, // Would call external API in production
        is_blacklisted: blacklisted.is_some(),
        recommendation: if blacklisted.is_some() { "block".to_string() } else { "allow".to_string() },
    };
    
    Ok(Json(result))
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
        return Err(ApiError::Forbidden("Not authorized for compliance operations".to_string()));
    }
    
    Ok(stablecoin)
}

/// Legacy handler for compatibility
pub async fn handler() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({"compliance": "enabled", "blacklist_count": 0})))
}