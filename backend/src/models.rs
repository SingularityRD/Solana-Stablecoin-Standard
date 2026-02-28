use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub solana_pubkey: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
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

#[derive(Debug, Deserialize)]
pub struct CreateStablecoinRequest {
    pub name: String,
    pub symbol: String,
    pub decimals: Option<u8>,
    pub preset: u8,
    pub asset_mint: String,
    pub authority_keypair: Option<String>, // Base58 encoded keypair (encrypted)
}

#[derive(Debug, Deserialize)]
pub struct UpdateStablecoinRequest {
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
#[derive(Debug, Deserialize)]
pub struct MintRequest {
    pub recipient: String,
    pub amount: u64,
}

#[derive(Debug, Deserialize)]
pub struct BurnRequest {
    pub amount: u64,
    pub from_account: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub tx_signature: String,
    pub status: String,
    pub explorer_url: String,
}

// ==================== Compliance Models ====================
#[derive(Debug, Deserialize)]
pub struct BlacklistAddRequest {
    pub account: String,
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
#[derive(Debug, Deserialize)]
pub struct AssignRoleRequest {
    pub account: String,
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
#[derive(Debug, Deserialize)]
pub struct AddMinterRequest {
    pub account: String,
    pub quota: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct SetQuotaRequest {
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
#[derive(Debug, Deserialize)]
pub struct SeizeRequest {
    pub from_account: String,
    pub to_account: String,
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
#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Vec<String>,
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
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: Option<String>,
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
