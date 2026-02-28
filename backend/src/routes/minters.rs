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
    models::{AddMinterRequest, MinterQuota, SetQuotaRequest, User},
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

/// Add a minter with optional quota
pub async fn add(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AddMinterRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate input using validator crate
    req.validate().map_err(validation_error_to_api_error)?;
    
    // Parse and validate minter pubkey (additional validation)
    let minter_pubkey: Pubkey = req.account.parse()
        .map_err(|_| ApiError::Validation("Invalid minter pubkey".to_string()))?;
    
    // Get stablecoin and check ownership
    let stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Parse stablecoin PDA
    let stablecoin_pda: Pubkey = stablecoin.stablecoin_pda.parse()
        .map_err(|_| ApiError::Internal("Invalid stablecoin PDA".to_string()))?;
    
    // Find minter PDA
    let (minter_pda, _bump) = state.solana.find_minter_pda(&stablecoin_pda, &minter_pubkey);
    
    // Create minter quota entry
    let quota = req.quota.unwrap_or(0) as i64;
    let minter: MinterQuota = query_as(
        r#"
        INSERT INTO minter_quotas (stablecoin_id, minter_pubkey, quota, minted_amount)
        VALUES ($1, $2, $3, 0)
        ON CONFLICT (stablecoin_id, minter_pubkey)
        DO UPDATE SET quota = $3, updated_at = NOW()
        RETURNING *
        "#
    )
    .bind(id)
    .bind(&req.account)
    .bind(quota)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    // Log audit
    audit(
        &state.db,
        Some(id),
        Some(user.id),
        "minter.add",
        None,
        Some(json!({"minter": req.account, "quota": quota, "pda": minter_pda.to_string()})),
        None,
    ).await;
    
    Ok((StatusCode::CREATED, Json(minter)))
}

/// Remove a minter
pub async fn remove(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, account)): Path<(Uuid, String)>,
) -> ApiResult<impl IntoResponse> {
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Delete minter quota
    let result = sqlx::query(
        "DELETE FROM minter_quotas WHERE stablecoin_id = $1 AND minter_pubkey = $2"
    )
    .bind(id)
    .bind(&account)
    .execute(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Minter not found".to_string()));
    }
    
    // Log audit
    audit(
        &state.db,
        Some(id),
        Some(user.id),
        "minter.remove",
        None,
        Some(json!({"minter": account})),
        None,
    ).await;
    
    Ok(StatusCode::NO_CONTENT)
}

/// List all minters for a stablecoin
pub async fn list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    let minters: Vec<MinterQuota> = query_as(
        "SELECT * FROM minter_quotas WHERE stablecoin_id = $1 ORDER BY created_at DESC"
    )
    .bind(id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    Ok(Json(minters))
}

/// Set or update minter quota
pub async fn set_quota(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, account)): Path<(Uuid, String)>,
    Json(req): Json<SetQuotaRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate input using validator crate
    req.validate().map_err(validation_error_to_api_error)?;
    
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Update quota
    let minter: MinterQuota = query_as(
        r#"
        UPDATE minter_quotas
        SET quota = $1, updated_at = NOW()
        WHERE stablecoin_id = $2 AND minter_pubkey = $3
        RETURNING *
        "#
    )
    .bind(req.quota as i64)
    .bind(id)
    .bind(&account)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Minter not found".to_string()))?;
    
    // Log audit
    audit(
        &state.db,
        Some(id),
        Some(user.id),
        "minter.set_quota",
        None,
        Some(json!({"minter": account, "quota": req.quota})),
        None,
    ).await;
    
    Ok(Json(minter))
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
        return Err(ApiError::Forbidden("Not authorized for minter management".to_string()));
    }
    
    Ok(stablecoin)
}
