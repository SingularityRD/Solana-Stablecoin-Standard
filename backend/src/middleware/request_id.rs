use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // Get or generate request ID
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Add to request extensions
    request.extensions_mut().insert(request_id.clone());
    
    // Process request
    let mut response = next.run(request).await;
    
    // Add request ID to response headers
    response.headers_mut().insert(
        "x-request-id",
        header::HeaderValue::from_str(&request_id).unwrap(),
    );
    
    response
}
