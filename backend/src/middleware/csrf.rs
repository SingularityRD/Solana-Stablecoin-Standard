//! CSRF (Cross-Site Request Forgery) protection middleware
//! 
//! This module provides CSRF protection for state-changing operations.
//! It uses the double-submit cookie pattern combined with origin/referer validation.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::config::AppConfig;
use crate::AppState;

/// CSRF token header name
pub const CSRF_HEADER: &str = "x-csrf-token";

/// Safe methods that don't require CSRF protection
const SAFE_METHODS: [Method; 3] = [Method::GET, Method::HEAD, Method::OPTIONS];

/// CSRF protection middleware
/// 
/// This middleware implements CSRF protection using:
/// 1. Origin/Referer header validation for all state-changing requests
/// 2. Optional CSRF token validation (double-submit cookie pattern)
/// 
/// In production, all POST, PUT, DELETE, PATCH requests must have:
/// - A valid Origin or Referer header matching allowed origins
/// - OR a valid CSRF token in the request header
pub async fn csrf_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    
    // Skip CSRF check for safe methods
    if SAFE_METHODS.contains(&method) {
        return Ok(next.run(request).await);
    }
    
    // Skip CSRF for health/metrics endpoints
    let path = request.uri().path();
    if path.starts_with("/health") || path.starts_with("/metrics") {
        return Ok(next.run(request).await);
    }
    
    // In development, log but don't enforce
    if state.config.environment.is_development() {
        let origin = get_origin_or_referer(&request.headers());
        tracing::debug!(
            method = %method,
            path = %path,
            origin = ?origin,
            "CSRF check (development mode - not enforced)"
        );
        return Ok(next.run(request).await);
    }
    
    // Validate Origin/Referer header
    let origin = get_origin_or_referer(&request.headers());
    
    match origin {
        Some(ref origin_str) => {
            // Check if origin is in allowed list
            if !is_origin_allowed(&origin_str, &state.config.cors_origins) {
                tracing::warn!(
                    method = %method,
                    path = %path,
                    origin = %origin_str,
                    allowed_origins = ?state.config.cors_origins,
                    "CSRF validation failed: origin not allowed"
                );
                return Err(StatusCode::FORBIDDEN);
            }
            
            tracing::trace!(
                method = %method,
                path = %path,
                origin = %origin_str,
                "CSRF validation passed"
            );
        }
        None => {
            // No Origin or Referer header - check for CSRF token
            if !validate_csrf_token(&request, &state.config) {
                tracing::warn!(
                    method = %method,
                    path = %path,
                    "CSRF validation failed: no origin/referer and no valid CSRF token"
                );
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// Extract Origin or Referer header from request
fn get_origin_or_referer(headers: &HeaderMap) -> Option<String> {
    headers
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get("referer")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| url::Url::parse(s).ok())
                .map(|url| {
                    format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""))
                })
        })
}

/// Check if origin is in allowed list
fn is_origin_allowed(origin: &str, allowed_origins: &[String]) -> bool {
    allowed_origins.iter().any(|allowed| {
        // Exact match
        if origin == allowed {
            return true;
        }
        // Allow subdomain matching for *.domain.com pattern
        if allowed.starts_with("*.") {
            let domain = &allowed[2..];
            if let Ok(origin_url) = url::Url::parse(origin) {
                if let Some(host) = origin_url.host_str() {
                    return host.ends_with(domain) || host == domain;
                }
            }
        }
        false
    })
}

/// Validate CSRF token from request
fn validate_csrf_token(request: &Request, config: &AppConfig) -> bool {
    // Check for CSRF token in header
    let token = request
        .headers()
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok());
    
    match token {
        Some(token) => {
            // Validate token structure (should be timestamped hash)
            validate_token_structure(token, &config.csrf_secret)
        }
        None => false,
    }
}

/// Validate CSRF token structure
fn validate_token_structure(token: &str, secret: &str) -> bool {
    // Token format: base64(timestamp:hash)
    // where hash = sha256(timestamp:secret)
    let decoded = match URL_SAFE_NO_PAD.decode(token) {
        Ok(d) => d,
        Err(_) => return false,
    };
    
    let decoded_str = match std::str::from_utf8(&decoded) {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    let parts: Vec<&str> = decoded_str.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let timestamp = parts[0];
    let provided_hash = parts[1];
    
    // Verify hash
    let expected_hash = generate_hash(timestamp, secret);
    
    // Constant-time comparison to prevent timing attacks
    constant_time_eq(&expected_hash, provided_hash)
}

/// Generate hash for CSRF token
fn generate_hash(data: &str, secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", data, secret));
    format!("{:x}", hasher.finalize())
}

/// Constant-time string comparison
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    
    let mut result = 0u8;
    for (x, y) in a_bytes.iter().zip(b_bytes.iter()) {
        result |= x ^ y;
    }
    
    result == 0
}

/// Generate a new CSRF token
pub fn generate_csrf_token(secret: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let data = timestamp.to_string();
    let hash = generate_hash(&data, secret);
    
    let token = format!("{}:{}", data, hash);
    URL_SAFE_NO_PAD.encode(token.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_and_validate_token() {
        let secret = "test-secret-key";
        let token = generate_csrf_token(secret);
        
        assert!(validate_token_structure(&token, secret));
        assert!(!validate_token_structure(&token, "wrong-secret"));
    }
    
    #[test]
    fn test_origin_allowed() {
        let origins = vec![
            "https://example.com".to_string(),
            "*.trusted.com".to_string(),
        ];
        
        assert!(is_origin_allowed("https://example.com", &origins));
        assert!(is_origin_allowed("https://sub.trusted.com", &origins));
        assert!(is_origin_allowed("https://trusted.com", &origins));
        assert!(!is_origin_allowed("https://evil.com", &origins));
    }
    
    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq("abc", "abc"));
        assert!(!constant_time_eq("abc", "abd"));
        assert!(!constant_time_eq("abc", "ab"));
    }
}
