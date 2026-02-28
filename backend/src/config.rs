use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_addr: String,
    pub database_url: String,
    pub redis_url: Option<String>,
    pub solana_rpc_url: String,
    pub program_id: Pubkey,
    pub jwt_secret: String,
    pub jwt_expiry: u64,
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u64,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let server_addr = env::var("SERVER_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:3001".to_string());
        
        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;
        
        let redis_url = env::var("REDIS_URL").ok();
        
        let solana_rpc_url = env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        
        let program_id_str = env::var("PROGRAM_ID")
            .unwrap_or_else(|_| "SSSToken11111111111111111111111111111111111".to_string());
        
        let program_id = program_id_str
            .parse::<Pubkey>()
            .with_context(|| format!("Invalid PROGRAM_ID: {}", program_id_str))?;
        
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| {
                tracing::warn!("JWT_SECRET not set, using default (NOT SECURE FOR PRODUCTION!)");
                "super-secret-key-change-in-production".to_string()
            });
        
        let jwt_expiry = env::var("JWT_EXPIRY_SECS")
            .unwrap_or_else(|_| "86400".to_string())
            .parse()
            .unwrap_or(86400);
        
        let rate_limit_requests = env::var("RATE_LIMIT_REQUESTS")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);
        
        let rate_limit_window_secs = env::var("RATE_LIMIT_WINDOW_SECS")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .unwrap_or(60);
        
        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());
        
        Ok(Self {
            server_addr,
            database_url,
            redis_url,
            solana_rpc_url,
            program_id,
            jwt_secret,
            jwt_expiry,
            rate_limit_requests,
            rate_limit_window_secs,
            log_level,
        })
    }
}
