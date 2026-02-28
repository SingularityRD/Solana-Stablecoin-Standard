use axum::{
    async_trait,
    extract::{FromRequestParts, Request, State},
    http::{header, request::Parts, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::User;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,       // User ID
    pub email: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,       // JWT ID for revocation
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
}

pub fn generate_access_token(user: &User, jwt_secret: &str, expiry_secs: u64) -> Result<String, ApiError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.clone(),
        exp: now + expiry_secs as usize,
        iat: now,
        jti: Uuid::new_v4().to_string(),
    };
    
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(ApiError::from)
}

pub fn generate_refresh_token(user_id: Uuid, jwt_secret: &str) -> Result<String, ApiError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    
    let claims = RefreshClaims {
        sub: user_id.to_string(),
        exp: now + 604800, // 7 days
        iat: now,
        jti: Uuid::new_v4().to_string(),
    };
    
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(ApiError::from)
}

pub fn verify_token(token: &str, jwt_secret: &str) -> Result<Claims, ApiError> {
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_err(ApiError::from)?;
    
    Ok(token_data.claims)
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing authorization header".to_string()))?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Invalid authorization header format".to_string()))?;
    
    let claims = verify_token(token, &state.config.jwt_secret)?;
    
    // Check if user is still active
    let user: User = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1 AND is_active = true"
    )
    .bind(Uuid::parse_str(&claims.sub).map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?)
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::Unauthorized("User not found or inactive".to_string()))?;
    
    // Add user to request extensions
    request.extensions_mut().insert(user);
    request.extensions_mut().insert(claims);
    
    Ok(next.run(request).await)
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

impl From<Claims> for CurrentUser {
    fn from(claims: Claims) -> Self {
        Self {
            id: Uuid::parse_str(&claims.sub).unwrap(),
            email: claims.email,
            role: claims.role,
        }
    }
}

/// Extractor for authenticated user in route handlers
#[derive(Debug, Clone)]
pub struct AuthUser(pub User);

impl std::ops::Deref for AuthUser {
    type Target = User;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Implement FromRequestParts for AuthUser
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<serde_json::Value>);
    
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<User>()
            .cloned()
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::json!({"error": "User not found in request"})),
                )
            })?;
        
        Ok(AuthUser(user))
    }
}
