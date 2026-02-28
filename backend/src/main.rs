use axum::{
    extract::Request,
    middleware,
    routing::{get, post, put, delete},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer, AllowOrigin},
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
    set_header::SetResponseHeaderLayer,
    util::option_layer,
};
use http::{header, HeaderValue, Method};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::signal;

mod config;
mod db;
mod error;
#[path = "middleware/mod.rs"]
mod app_middleware;
mod models;
mod routes;
mod services;
mod solana;
mod utils;

#[cfg(test)]
mod tests;

use config::AppConfig;
use db::Database;
use services::{SolanaService, MintBurnService, ComplianceService};

/// Application version - set at compile time
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application start time for uptime calculation
pub static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: Database,
    pub solana: Arc<SolanaService>,
    pub mint_burn: Arc<MintBurnService>,
    pub compliance: Arc<ComplianceService>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sss_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Starting SSS Backend...");

    // Load configuration
    let config = Arc::new(AppConfig::from_env()?);
    tracing::info!("Configuration loaded");

    // Connect to database
    let db = Database::new(&config.database_url).await?;
    tracing::info!("Database connected");

    // Run migrations
    db.migrate().await?;
    tracing::info!("Database migrations completed");

    // Initialize Solana service
    let solana = Arc::new(SolanaService::new(&config.solana_rpc_url, config.program_id).await?);
    tracing::info!("Solana service initialized");

    // Initialize Mint/Burn service
    let mut mint_burn = MintBurnService::new(
        "backend".to_string(),
        solana.clone(),
    );
    mint_burn.set_cluster(config.cluster.clone());
    
    // Initialize Compliance service
    let mut compliance = ComplianceService::new(
        "".to_string(), // API key can be set via environment
        solana.clone(),
    );
    compliance.set_cluster(config.cluster.clone());

    // Load authority keypair if configured
    if let Some(keypair_b58) = &config.authority_keypair {
        match crate::solana::parse_keypair(keypair_b58) {
            Ok(keypair) => {
                tracing::info!("Loaded authority keypair: {}", keypair.pubkey());
                solana.set_keypair(keypair.clone()).await;
                mint_burn.set_authority_keypair(keypair.clone());
                compliance.set_authority_keypair(keypair);
            }
            Err(e) => {
                tracing::warn!("Failed to load authority keypair: {}", e);
            }
        }
    } else {
        tracing::warn!("AUTHORITY_KEYPAIR not set - transactions will fail until loaded via API");
    }

    let mint_burn = Arc::new(mint_burn);
    let compliance = Arc::new(compliance);

    // Create app state
    let state = AppState {
        config: config.clone(),
        db,
        solana,
        mint_burn,
        compliance,
    };

    // Build router with middleware
    let app = Router::new()
        // Health checks (no auth required)
        .route("/health", get(routes::health::handler))
        .route("/health/detail", get(routes::health::detailed_handler))
        .route("/health/ready", get(routes::health::readiness_handler))
        .route("/health/live", get(routes::health::liveness_handler))
        .route("/metrics", get(routes::metrics::handler))
        
        // Public routes
        .route("/api/v1/auth/register", post(routes::auth::register))
        .route("/api/v1/auth/login", post(routes::auth::login))
        .route("/api/v1/auth/refresh", post(routes::auth::refresh))
        
        // Protected routes (require authentication)
        .nest("/api/v1", 
            Router::new()
                // Stablecoin operations
                .route("/stablecoin", post(routes::stablecoin::create))
                .route("/stablecoin/:id", get(routes::stablecoin::get))
                .route("/stablecoin/:id", put(routes::stablecoin::update))
                .route("/stablecoin/:id/status", get(routes::stablecoin::status))
                .route("/stablecoin", get(routes::stablecoin::list))
                
                // Mint/Burn operations
                .route("/stablecoin/:id/mint", post(routes::operations::mint))
                .route("/stablecoin/:id/burn", post(routes::operations::burn))
                .route("/stablecoin/:id/transfer", post(routes::operations::transfer))
                
                // Compliance (SSS-2)
                .route("/stablecoin/:id/blacklist", post(routes::compliance::blacklist_add))
                .route("/stablecoin/:id/blacklist/:account", delete(routes::compliance::blacklist_remove))
                .route("/stablecoin/:id/blacklist", get(routes::compliance::blacklist_list))
                .route("/stablecoin/:id/screen/:address", get(routes::compliance::screen))
                
                // Admin operations
                .route("/stablecoin/:id/pause", post(routes::admin::pause))
                .route("/stablecoin/:id/unpause", post(routes::admin::unpause))
                .route("/stablecoin/:id/freeze/:account", post(routes::admin::freeze))
                .route("/stablecoin/:id/thaw/:account", post(routes::admin::thaw))
                .route("/stablecoin/:id/seize", post(routes::admin::seize))
                
                // Role management
                .route("/stablecoin/:id/roles", post(routes::roles::assign))
                .route("/stablecoin/:id/roles/:account", delete(routes::roles::revoke))
                .route("/stablecoin/:id/roles", get(routes::roles::list))
                
                // Minter management
                .route("/stablecoin/:id/minters", post(routes::minters::add))
                .route("/stablecoin/:id/minters/:account", delete(routes::minters::remove))
                .route("/stablecoin/:id/minters", get(routes::minters::list))
                .route("/stablecoin/:id/minters/:account/quota", put(routes::minters::set_quota))
                
                // Audit logs
                .route("/stablecoin/:id/audit", get(routes::audit::list))
                .route("/audit/:tx_signature", get(routes::audit::get))
                
                // Webhooks
                .route("/stablecoin/:id/webhooks", post(routes::webhooks::create))
                .route("/stablecoin/:id/webhooks", get(routes::webhooks::list))
                .route("/stablecoin/:id/webhooks/:webhook_id", delete(routes::webhooks::delete))
                
                // User management
                .route("/users/me", get(routes::users::me))
                .route("/users/me", put(routes::users::update))
                
                .layer(middleware::from_fn_with_state(state.clone(), app_middleware::auth::auth_middleware))
        )
        
        // Webhook receiver (no auth)
        .route("/webhooks", post(routes::webhooks::handler))
        
        // Global middleware
        .layer(middleware::from_fn(app_middleware::rate_limit::rate_limit_middleware))
        .layer(middleware::from_fn(app_middleware::request_id::request_id_middleware))
        
        // CSRF protection - enabled in staging/production
        .layer(option_layer((!config.environment.is_development()).then(|| {
            middleware::from_fn_with_state(state.clone(), app_middleware::csrf::csrf_middleware)
        })))
        
        // HTTPS enforcement - only in production (must be after other middleware)
        .layer(option_layer(config.enforce_https.then(|| {
            middleware::from_fn(app_middleware::https::https_enforcement_middleware)
        })))
        
        // Security headers (applied in reverse order)
        // X-Content-Type-Options: nosniff
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        
        // X-Frame-Options: DENY
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        
        // X-XSS-Protection: 1; mode=block (legacy but still useful)
        .layer(SetResponseHeaderLayer::overriding(
            HeaderValue::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ))
        
        // Referrer-Policy: strict-origin-when-cross-origin
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        
        // Permissions-Policy (formerly Feature-Policy)
        .layer(SetResponseHeaderLayer::overriding(
            HeaderValue::from_static("permissions-policy"),
            HeaderValue::from_static("geolocation=(), microphone=(), camera=(), payment=()"),
        ))
        
        // HSTS (HTTP Strict Transport Security) - only in production with HTTPS
        .layer(SetResponseHeaderLayer::overriding_if(
            config.environment.is_production(),
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        ))
        
        // Content-Security-Policy
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"),
        ))
        
        // CORS - configured based on environment
        .layer({
            let cors = if config.environment.is_development() {
                // Development: Allow all origins
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS])
                    .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT, header::X_REQUESTED_WITH])
                    .allow_credentials(true)
            } else {
                // Production/Staging: Restrict to configured origins
                let origins: Vec<HeaderValue> = config.cors_origins
                    .iter()
                    .filter_map(|origin| origin.parse().ok())
                    .collect();
                
                CorsLayer::new()
                    .allow_origin(AllowOrigin::list(origins))
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS])
                    .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT, header::X_REQUESTED_WITH])
                    .allow_credentials(true)
                    .max_age(std::time::Duration::from_secs(3600))
            };
            cors
        })
        
        // Request body limit (1MB)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        
        // Tracing
        .layer(TraceLayer::new_for_http())
        
        .with_state(state);

    // Record start time for uptime calculation
    START_TIME.set(Instant::now()).expect("START_TIME already set");

    let addr: SocketAddr = config.server_addr.parse()
        .unwrap_or_else(|_| SocketAddr::from(([0,0,0,0], 3001)));
    
    tracing::info!("Server listening on {}", addr);
    
    // HTTPS enforcement warning
    if config.enforce_https && config.environment.is_production() {
        tracing::info!("HTTPS enforcement is enabled - ensure TLS termination is configured");
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // Clone state for graceful shutdown
    let shutdown_state = state.clone();
    
    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_state))
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

/// Handles graceful shutdown signals (SIGTERM, SIGINT, Ctrl+C)
async fn shutdown_signal(state: AppState) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    #[cfg(unix)]
    let interrupt = async {
        signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let interrupt = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, starting graceful shutdown...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM signal, starting graceful shutdown...");
        },
        _ = interrupt => {
            tracing::info!("Received SIGINT signal, starting graceful shutdown...");
        },
    }

    // Perform cleanup with timeout
    let shutdown_timeout = Duration::from_secs(30);
    tracing::info!("Initiating graceful shutdown with {:?} timeout...", shutdown_timeout);

    let cleanup_result = tokio::time::timeout(shutdown_timeout, async {
        // Close database connections
        tracing::info!("Closing database connections...");
        state.db.close().await;
        
        // Any other cleanup can go here
        
        tracing::info!("Cleanup completed successfully");
    }).await;

    if cleanup_result.is_err() {
        tracing::warn!("Shutdown timeout reached, forcing exit");
    }
}
