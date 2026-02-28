use axum::{
    extract::{Request, ConnectInfo},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::ApiError;

#[derive(Debug, Clone)]
struct RateLimitEntry {
    requests: u32,
    window_start: Instant,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    max_requests: u32,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_duration: Duration::from_secs(window_secs),
        }
    }
    
    pub async fn check(&self, key: &str) -> Result<(), ApiError> {
        let mut entries = self.entries.write().await;
        let now = Instant::now();
        
        let entry = entries.entry(key.to_string()).or_insert(RateLimitEntry {
            requests: 0,
            window_start: now,
        });
        
        // Reset window if expired
        if now.duration_since(entry.window_start) > self.window_duration {
            entry.requests = 0;
            entry.window_start = now;
        }
        
        // Check limit
        if entry.requests >= self.max_requests {
            return Err(ApiError::RateLimited);
        }
        
        entry.requests += 1;
        Ok(())
    }
    
    // Cleanup old entries periodically
    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        let now = Instant::now();
        
        entries.retain(|_, entry| {
            now.duration_since(entry.window_start) < self.window_duration * 2
        });
    }
}

// Global rate limiter
static RATE_LIMITER: once_cell::sync::Lazy<RateLimiter> = once_cell::sync::Lazy::new(|| {
    let max_requests = std::env::var("RATE_LIMIT_REQUESTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    
    let window_secs = std::env::var("RATE_LIMIT_WINDOW_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    
    RateLimiter::new(max_requests, window_secs)
});

pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Get client IP from various sources
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
        })
        .unwrap_or("unknown");
    
    // Rate limit by IP
    RATE_LIMITER.check(client_ip).await?;
    
    // Also check by API key if present
    if let Some(api_key) = request.headers().get("x-api-key") {
        if let Ok(key) = api_key.to_str() {
            RATE_LIMITER.check(&format!("api:{}", key)).await?;
        }
    }
    
    Ok(next.run(request).await)
}
