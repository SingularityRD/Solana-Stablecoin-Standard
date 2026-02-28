use axum::{
    extract::Request,
    middleware,
    routing::{get, post, put, delete},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

use config::AppConfig;
use db::Database;
use services::SolanaService;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: Database,
    pub solana: Arc<SolanaService>,
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

    // Create app state
    let state = AppState {
        config: config.clone(),
        db,
        solana,
    };

    // Build router with middleware
    let app = Router::new()
        // Health check (no auth required)
        .route("/health", get(routes::health::handler))
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
        
        // CORS
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        
        // Request body limit (1MB)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        
        // Tracing
        .layer(TraceLayer::new_for_http())
        
        .with_state(state);

    let addr: SocketAddr = config.server_addr.parse()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 3001)));
    
    tracing::info!("🚀 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
