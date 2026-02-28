use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ==================== User Models ====================
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub solana_pubkey: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Custom validator function for password complexity
/// Requires: min 8 chars, uppercase, lowercase, number, special char
pub fn validate_password_complexity(password: &str) -> Result<(), validator::ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;':\",./<>?`~".contains(c));
    
    if !(has_uppercase && has_lowercase && has_digit && has_special) {
        return Err(validator::ValidationError::new("password_complexity")
            .with_message(std::borrow::Cow::Borrowed(
                "Password must contain uppercase, lowercase, number, and special character"
            )));
    }
    Ok(())
}

/// Custom validator for Solana pubkey format
pub fn validate_solana_pubkey(pubkey: &str) -> Result<(), validator::ValidationError> {
    // Base58 encoded Solana pubkeys are typically 32-44 characters
    if pubkey.len() < 32 || pubkey.len() > 44 {
        return Err(validator::ValidationError::new("solana_pubkey")
            .with_message(std::borrow::Cow::Borrowed("Invalid Solana pubkey length")));
    }
    // Check for valid Base58 characters
    const BASE58_CHARS: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    if !pubkey.chars().all(|c| BASE58_CHARS.contains(c)) {
        return Err(validator::ValidationError::new("solana_pubkey")
            .with_message(std::borrow::Cow::Borrowed("Invalid Base58 character in pubkey")));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"), custom = "validate_password_complexity")]
    pub password: String,
    
    #[validate(custom = "validate_solana_pubkey")]
    pub solana_pubkey: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}


#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserPublic,
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub solana_pubkey: Option<String>,
    pub role: String,
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            solana_pubkey: user.solana_pubkey,
            role: user.role,
        }
    }
}

// ==================== Stablecoin Models ====================
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Stablecoin {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub symbol: String,
    pub decimals: i16,
    pub preset: i16,
    pub asset_mint: String,
    pub stablecoin_pda: String,
    pub authority_pubkey: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Custom validator for stablecoin name (alphanumeric with spaces, dashes, underscores)
pub fn validate_stablecoin_name(name: &str) -> Result<(), validator::ValidationError> {
    if !name.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_') {
        return Err(validator::ValidationError::new("stablecoin_name")
            .with_message(std::borrow::Cow::Borrowed(
                "Name can only contain alphanumeric characters, spaces, dashes, and underscores"
            )));
    }
    Ok(())
}

/// Custom validator for stablecoin symbol (alphanumeric only)
pub fn validate_stablecoin_symbol(symbol: &str) -> Result<(), validator::ValidationError> {
    if !symbol.chars().all(|c| c.is_alphanumeric()) {
        return Err(validator::ValidationError::new("stablecoin_symbol")
            .with_message(std::borrow::Cow::Borrowed(
                "Symbol can only contain alphanumeric characters"
            )));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateStablecoinRequest {
    #[validate(length(min = 1, max = 64, message = "Name must be 1-64 characters"), custom = "validate_stablecoin_name")]
    pub name: String,
    
    #[validate(length(min = 1, max = 16, message = "Symbol must be 1-16 characters"), custom = "validate_stablecoin_symbol")]
    pub symbol: String,
    
    #[validate(range(min = 0, max = 9, message = "Decimals must be between 0 and 9"))]
    pub decimals: Option<u8>,
    
    #[validate(range(min = 0, max = 2, message = "Preset must be 0 (SSS-1), 1 (SSS-2), or 2 (SSS-3)"))]
    pub preset: u8,
    
    #[validate(custom = "validate_solana_pubkey")]
    pub asset_mint: String,
    
    pub authority_keypair: Option<String>, // Base58 encoded keypair (encrypted)
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateStablecoinRequest {
    #[validate(length(min = 1, max = 64, message = "Name must be 1-64 characters"), custom = "validate_stablecoin_name")]
    pub name: Option<String>,
    
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct StablecoinStatus {
    pub stablecoin: Stablecoin,
    pub total_supply: u64,
    pub paused: bool,
    pub compliance_enabled: bool,
    pub holder_count: u64,
}

// ==================== Operation Models ====================

/// Maximum allowed amount for mint/burn/transfer operations (protects against overflow)
pub const MAX_OPERATION_AMOUNT: u64 = 1_000_000_000_000_000; // 1 quadrillion (10^15)

/// Custom validator for amount (ensures non-zero and within safe bounds)
pub fn validate_amount(amount: &u64) -> Result<(), validator::ValidationError> {
    if *amount == 0 {
        return Err(validator::ValidationError::new("amount")
            .with_message(std::borrow::Cow::Borrowed("Amount must be greater than 0")));
    }
    if *amount > MAX_OPERATION_AMOUNT {
        return Err(validator::ValidationError::new("amount")
            .with_message(std::borrow::Cow::Borrowed(
                "Amount exceeds maximum allowed value (1 quadrillion)"
            )));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
pub struct MintRequest {
    #[validate(custom = "validate_solana_pubkey")]
    pub recipient: String,
    
    #[validate(custom = "validate_amount")]
    pub amount: u64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct BurnRequest {
    #[validate(custom = "validate_amount")]
    pub amount: u64,
    
    #[validate(custom = "validate_solana_pubkey")]
    pub from_account: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct TransferRequest {
    #[validate(custom = "validate_solana_pubkey")]
    pub from: String,
    
    #[validate(custom = "validate_solana_pubkey")]
    pub to: String,
    
    #[validate(custom = "validate_amount")]
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub tx_signature: String,
    pub status: String,
    pub explorer_url: String,
}

// ==================== Compliance Models ====================
#[derive(Debug, Deserialize, Validate)]
pub struct BlacklistAddRequest {
    #[validate(custom = "validate_solana_pubkey")]
    pub account: String,
    
    #[validate(length(min = 1, max = 500, message = "Reason must be 1-500 characters"))]
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlacklistEntry {
    pub id: Uuid,
    pub stablecoin_id: Uuid,
    pub account_pubkey: String,
    pub reason: String,
    pub blacklisted_by: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ScreeningResult {
    pub address: String,
    pub risk_score: u8,
    pub is_sanctioned: bool,
    pub is_blacklisted: bool,
    pub recommendation: String,
}

// ==================== Role Models ====================

/// Valid roles for assignment
pub const VALID_ROLES: &[&str] = &["admin", "minter", "freezer", "compliance", "auditor"];

/// Custom validator for role
pub fn validate_role(role: &str) -> Result<(), validator::ValidationError> {
    if !VALID_ROLES.contains(&role.to_lowercase().as_str()) {
        return Err(validator::ValidationError::new("role")
            .with_message(std::borrow::Cow::Borrowed(
                "Invalid role. Must be one of: admin, minter, freezer, compliance, auditor"
            )));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
pub struct AssignRoleRequest {
    #[validate(custom = "validate_solana_pubkey")]
    pub account: String,
    
    #[validate(custom = "validate_role")]
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleAssignment {
    pub id: Uuid,
    pub stablecoin_id: Uuid,
    pub account_pubkey: String,
    pub role: String,
    pub assigned_by: Uuid,
    pub created_at: DateTime<Utc>,
}

// ==================== Minter Models ====================
#[derive(Debug, Deserialize, Validate)]
pub struct AddMinterRequest {
    #[validate(custom = "validate_solana_pubkey")]
    pub account: String,
    
    #[validate(range(min = 0, message = "Quota cannot be negative"))]
    pub quota: Option<u64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SetQuotaRequest {
    #[validate(range(min = 0, message = "Quota cannot be negative"))]
    pub quota: u64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MinterQuota {
    pub id: Uuid,
    pub stablecoin_id: Uuid,
    pub minter_pubkey: String,
    pub quota: i64,
    pub minted_amount: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== Admin Models ====================
#[derive(Debug, Deserialize, Validate)]
pub struct SeizeRequest {
    #[validate(custom = "validate_solana_pubkey")]
    pub from_account: String,
    
    #[validate(custom = "validate_solana_pubkey")]
    pub to_account: String,
    
    #[validate(custom = "validate_amount")]
    pub amount: u64,
}

// ==================== Audit Models ====================
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub stablecoin_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub tx_signature: Option<String>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ==================== Webhook Models ====================

/// Valid webhook events
pub const VALID_WEBHOOK_EVENTS: &[&str] = &[
    "mint", "burn", "transfer", "freeze", "thaw", "blacklist", 
    "role_assigned", "role_revoked", "compliance_alert"
];

/// Custom validator for webhook URL
pub fn validate_webhook_url(url: &str) -> Result<(), validator::ValidationError> {
    // Parse URL
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return Err(validator::ValidationError::new("url")
            .with_message(std::borrow::Cow::Borrowed("Invalid URL format"))),
    };
    
    // Must be HTTP or HTTPS
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(validator::ValidationError::new("url")
            .with_message(std::borrow::Cow::Borrowed("URL must use HTTP or HTTPS protocol")));
    }
    
    // In production, require HTTPS (skip for localhost)
    #[cfg(not(debug_assertions))]
    {
        if scheme != "https" && parsed.host_str() != Some("localhost") {
            return Err(validator::ValidationError::new("url")
                .with_message(std::borrow::Cow::Borrowed(
                    "HTTPS is required for production webhook URLs"
                )));
        }
    }
    
    Ok(())
}

/// Custom validator for webhook events
pub fn validate_webhook_events(events: &[String]) -> Result<(), validator::ValidationError> {
    if events.is_empty() {
        return Err(validator::ValidationError::new("events")
            .with_message(std::borrow::Cow::Borrowed("At least one event must be specified")));
    }
    
    for event in events {
        if !VALID_WEBHOOK_EVENTS.contains(&event.to_lowercase().as_str()) {
            return Err(validator::ValidationError::new("events")
                .with_message(std::borrow::Cow::Borrowed(
                    format!("Invalid event '{}'. Valid events: {}", event, VALID_WEBHOOK_EVENTS.join(", ")).into()
                )));
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWebhookRequest {
    #[validate(length(min = 1, max = 2048, message = "URL must be 1-2048 characters"), custom = "validate_webhook_url")]
    pub url: String,
    
    #[validate(custom = "validate_webhook_events")]
    pub events: Vec<String>,
    
    #[validate(length(max = 128, message = "Secret must be at most 128 characters"))]
    pub secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub stablecoin_id: Uuid,
    pub url: String,
    pub events: serde_json::Value,
    pub secret: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ==================== API Key Models ====================

/// Valid API key permissions
pub const VALID_PERMISSIONS: &[&str] = &[
    "read", "write", "mint", "burn", "admin"
];

/// Custom validator for API key permissions
pub fn validate_permissions(permissions: &[String]) -> Result<(), validator::ValidationError> {
    for perm in permissions {
        if !VALID_PERMISSIONS.contains(&perm.to_lowercase().as_str()) {
            return Err(validator::ValidationError::new("permissions")
                .with_message(std::borrow::Cow::Borrowed(
                    format!("Invalid permission '{}'. Valid permissions: {}", perm, VALID_PERMISSIONS.join(", ")).into()
                )));
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateApiKeyRequest {
    #[validate(length(max = 64, message = "Name must be at most 64 characters"))]
    pub name: Option<String>,
    
    #[validate(custom = "validate_permissions")]
    pub permissions: Option<Vec<String>>,
    
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub key: String, // Only shown once!
    pub name: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}
