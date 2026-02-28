use axum::{extract::State, http::StatusCode, response::IntoResponse};
use prometheus::{Encoder, TextEncoder};
use crate::AppState;

/// Prometheus metrics endpoint
pub async fn handler(State(_state): State<AppState>) -> impl IntoResponse {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    
    // Gather all registered metrics
    let metric_families = prometheus::gather();
    
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!("Failed to encode metrics: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics".to_string());
    }
    
    match String::from_utf8(buffer) {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(e) => {
            tracing::error!("Failed to convert metrics to string: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to convert metrics".to_string())
        }
    }
}
