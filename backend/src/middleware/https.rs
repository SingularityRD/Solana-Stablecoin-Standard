//! HTTPS enforcement middleware for production environments
//! 
//! This middleware ensures all requests are made over HTTPS in production.
//! It checks for the X-Forwarded-Proto header set by load balancers/proxies.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// HTTPS enforcement middleware
/// 
/// In production environments behind a reverse proxy (nginx, load balancer),
/// this middleware checks the X-Forwarded-Proto header to ensure the original
/// request was made over HTTPS.
/// 
/// If the request is not HTTPS, returns 403 Forbidden.
pub async fn https_enforcement_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check X-Forwarded-Proto header (set by reverse proxy)
    if let Some(proto) = request.headers().get("x-forwarded-proto") {
        if proto != "https" {
            tracing::warn!(
                path = %request.uri().path(),
                "Rejected non-HTTPS request"
            );
            return Err(StatusCode::FORBIDDEN);
        }
    }
    
    // Also check X-Forwarded-Ssl (used by some proxies)
    if let Some(ssl) = request.headers().get("x-forwarded-ssl") {
        if ssl != "on" {
            tracing::warn!(
                path = %request.uri().path(),
                "Rejected non-SSL request"
            );
            return Err(StatusCode::FORBIDDEN);
        }
    }
    
    // Check Front-End-Https header (Microsoft IIS/Azure)
    if let Some(https) = request.headers().get("front-end-https") {
        if https != "on" {
            tracing::warn!(
                path = %request.uri().path(),
                "Rejected non-HTTPS request (Front-End-Https)"
            );
            return Err(StatusCode::FORBIDDEN);
        }
    }
    
    Ok(next.run(request).await)
}

/// Security context extractor
/// 
/// Extracts security-relevant information from the request for logging/auditing.
pub struct SecurityContext {
    pub is_secure: bool,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

impl SecurityContext {
    pub fn from_request(request: &Request) -> Self {
        let is_secure = request.headers()
            .get("x-forwarded-proto")
            .map(|v| v == "https")
            .unwrap_or(false);
        
        let client_ip = request.headers()
            .get("x-forwarded-for")
            .or_else(|| request.headers().get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .map(|s| {
                // Take only the first IP if there are multiple
                s.split(',').next().unwrap_or(s).trim().to_string()
            });
        
        let user_agent = request.headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        Self {
            is_secure,
            client_ip,
            user_agent,
        }
    }
}
