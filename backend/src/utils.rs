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