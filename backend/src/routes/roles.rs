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
    models::{AssignRoleRequest, RoleAssignment, User},
    app_middleware::auth::AuthUser,
    AppState,
};

/// Assign a role to an account
pub async fn assign(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AssignRoleRequest>,
) -> ApiResult<impl IntoResponse> {
    // Parse and validate account pubkey
    let account_pubkey: Pubkey = req.account.parse()
        .map_err(|_| ApiError::Validation("Invalid account pubkey".to_string()))?;
    
    // Validate role
    let valid_roles = ["minter", "burner", "freezer", "blacklister", "seizer", "pauser"];
    if !valid_roles.contains(&req.role.as_str()) {
        return Err(ApiError::Validation(format!(
            "Invalid role. Must be one of: {}", 
            valid_roles.join(", ")
        )));
    }
    
    // Get stablecoin and check ownership
    let stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Parse stablecoin PDA
    let stablecoin_pda: Pubkey = stablecoin.stablecoin_pda.parse()
        .map_err(|_| ApiError::Internal("Invalid stablecoin PDA".to_string()))?;
    
    // Find role PDA
    let (role_pda, _bump) = state.solana.find_role_pda(
        &stablecoin_pda, 
        &account_pubkey, 
        req.role.as_bytes()
    );
    
    // Create role assignment in database
    let role_assignment: RoleAssignment = query_as(
        r#"
        INSERT INTO role_assignments (stablecoin_id, account_pubkey, role, assigned_by)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (stablecoin_id, account_pubkey, role) 
        DO UPDATE SET assigned_by = $4
        RETURNING *
        "#
    )
    .bind(id)
    .bind(&req.account)
    .bind(&req.role)
    .bind(user.id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "role.assign",
        None,
        Some(json!({"account": req.account, "role": req.role, "pda": role_pda.to_string()})),
        None,
    ).await;
    
    Ok((StatusCode::CREATED, Json(role_assignment)))
}

/// Revoke a role from an account
pub async fn revoke(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, account)): Path<(Uuid, String)>,
) -> ApiResult<impl IntoResponse> {
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    // Delete role assignment
    let result = sqlx::query(
        "DELETE FROM role_assignments WHERE stablecoin_id = $1 AND account_pubkey = $2"
    )
    .bind(id)
    .bind(&account)
    .execute(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Role assignment not found".to_string()));
    }
    
    // Log audit
    let _ = state.db.log_audit(
        Some(id),
        Some(user.id),
        "role.revoke",
        None,
        Some(json!({"account": account})),
        None,
    ).await;
    
    Ok(StatusCode::NO_CONTENT)
}

/// List all role assignments for a stablecoin
pub async fn list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    // Get stablecoin and check ownership
    let _stablecoin = get_stablecoin_for_admin(&state, id, &user).await?;
    
    let roles: Vec<RoleAssignment> = query_as(
        "SELECT * FROM role_assignments WHERE stablecoin_id = $1 ORDER BY created_at DESC"
    )
    .bind(id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    Ok(Json(roles))
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
        return Err(ApiError::Forbidden("Not authorized for role management".to_string()));
    }
    
    Ok(stablecoin)
}
