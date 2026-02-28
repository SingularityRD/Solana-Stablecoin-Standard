use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,        // User ID
    pub email: String,
    pub role: String,
    pub exp: usize,       // Expiration time
    pub iat: usize,       // Issued at
    pub jti: Uuid,        // JWT ID for revocation
}

/// Token pair response
#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Generate JWT tokens for a user
pub fn generate_tokens(
    user_id: Uuid,
    email: &str,
    role: &str,
    jwt_secret: &str,
    expiry_secs: u64,
) -> Result<TokenPair, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    
    // Access token (short-lived)
    let access_claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        exp: (now + Duration::seconds(expiry_secs as i64)).timestamp() as usize,
        iat: now.timestamp() as usize,
        jti: Uuid::new_v4(),
    };
    
    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    
    // Refresh token (long-lived)
    let refresh_claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        exp: (now + Duration::days(30)).timestamp() as usize,
        iat: now.timestamp() as usize,
        jti: Uuid::new_v4(),
    };
    
    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    
    Ok(TokenPair {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: expiry_secs,
    })
}

/// Validate a JWT token and return claims
pub fn validate_token(token: &str, jwt_secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;
    
    Ok(token_data.claims)
}

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };
    
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(e),
    }
}

/// Generate a random API key
pub fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Validate email format
pub fn is_valid_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

/// Format a timestamp for display
pub fn format_timestamp(timestamp: i64) -> String {
    let dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

use crate::db::Database;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for tracking audit log failures
static AUDIT_FAILURE_COUNT: AtomicU64 = AtomicU64::new(0);

/// Critical actions that require alerting on audit log failure
pub const CRITICAL_ACTIONS: &[&str] = &[
    "stablecoin.seize",
    "stablecoin.freeze",
    "stablecoin.blacklist_add",
    "role.assign",
    "role.revoke",
    "minter.add",
    "minter.remove",
];

/// Audit log entry for structured logging
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub stablecoin_id: Option<uuid::Uuid>,
    pub user_id: Option<uuid::Uuid>,
    pub action: String,
    pub tx_signature: Option<String>,
    pub details: Option<Value>,
    pub ip_address: Option<String>,
}

/// Log an audit entry with proper error handling
/// 
/// This function:
/// - Attempts to write the audit log to the database
/// - On failure, logs the error at ERROR level via tracing
/// - For critical actions, sends an alert (logged at WARN level)
/// - Tracks the total number of audit failures globally
/// - Never fails the main operation
pub async fn log_audit(db: &Database, entry: AuditEntry) {
    let is_critical = CRITICAL_ACTIONS.contains(&entry.action.as_str());
    
    match db.log_audit(
        entry.stablecoin_id,
        entry.user_id,
        &entry.action,
        entry.tx_signature.as_deref(),
        entry.details.clone(),
        entry.ip_address.as_deref(),
    ).await {
        Ok(()) => {
            tracing::debug!(
                stablecoin_id = ?entry.stablecoin_id,
                user_id = ?entry.user_id,
                action = %entry.action,
                tx_signature = ?entry.tx_signature,
                "Audit log recorded successfully"
            );
        }
        Err(e) => {
            // Increment failure counter
            let failure_count = AUDIT_FAILURE_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            
            // Log the failure at ERROR level
            tracing::error!(
                stablecoin_id = ?entry.stablecoin_id,
                user_id = ?entry.user_id,
                action = %entry.action,
                tx_signature = ?entry.tx_signature,
                details = ?entry.details,
                error = %e,
                total_failures = failure_count,
                "Failed to write audit log to database"
            );
            
            // For critical actions, also log at WARN level for alerting systems
            if is_critical {
                tracing::warn!(
                    stablecoin_id = ?entry.stablecoin_id,
                    user_id = ?entry.user_id,
                    action = %entry.action,
                    tx_signature = ?entry.tx_signature,
                    details = ?entry.details,
                    error = %e,
                    "CRITICAL: Audit log failure for sensitive operation - immediate attention required"
                );
            }
        }
    }
}

/// Convenience function to create an audit entry and log it
pub async fn audit(
    db: &Database,
    stablecoin_id: Option<uuid::Uuid>,
    user_id: Option<uuid::Uuid>,
    action: &str,
    tx_signature: Option<&str>,
    details: Option<Value>,
    ip_address: Option<&str>,
) {
    log_audit(db, AuditEntry {
        stablecoin_id,
        user_id,
        action: action.to_string(),
        tx_signature: tx_signature.map(|s| s.to_string()),
        details,
        ip_address: ip_address.map(|s| s.to_string()),
    }).await;
}

/// Get the total number of audit log failures since server start
pub fn get_audit_failure_count() -> u64 {
    AUDIT_FAILURE_COUNT.load(Ordering::Relaxed)
}

/// Reset the audit failure counter (useful for monitoring/alerting)
pub fn reset_audit_failure_count() -> u64 {
    AUDIT_FAILURE_COUNT.swap(0, Ordering::Relaxed)
}