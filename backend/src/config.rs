use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;
use std::env;

/// Application environment
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }
    
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_addr: String,
    pub database_url: String,
    pub redis_url: Option<String>,
    pub solana_rpc_url: String,
    pub program_id: Pubkey,
    /// Authority keypair in base58 format (optional - can be set via API)
    pub authority_keypair: Option<String>,
    pub jwt_secret: String,
    pub jwt_expiry: u64,
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u64,
    pub log_level: String,
    /// Cluster name for explorer URLs (devnet, testnet, mainnet)
    pub cluster: String,
    /// Application environment
    pub environment: Environment,
    /// Allowed CORS origins (comma-separated)
    pub cors_origins: Vec<String>,
    /// Whether to enforce HTTPS
    pub enforce_https: bool,
    /// CSRF secret for token generation
    pub csrf_secret: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        // Parse environment first
        let environment = match env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            _ => Environment::Development,
        };
        
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
        
        // Authority keypair is optional - can be loaded dynamically
        let authority_keypair = env::var("AUTHORITY_KEYPAIR").ok();
        
        // JWT_SECRET is MANDATORY in production
        let jwt_secret = match env::var("JWT_SECRET") {
            Ok(secret) => {
                // Validate minimum length in production
                if environment.is_production() && secret.len() < 32 {
                    panic!("JWT_SECRET must be at least 32 characters in production environment");
                }
                secret
            }
            Err(_) => {
                if environment.is_production() {
                    panic!("JWT_SECRET environment variable is required in production environment");
                }
                tracing::warn!("JWT_SECRET not set, using default (NOT SECURE FOR PRODUCTION!)");
                "super-secret-key-change-in-production".to_string()
            }
        };
        
        // CSRF secret - mandatory in production
        let csrf_secret = match env::var("CSRF_SECRET") {
            Ok(secret) => {
                if environment.is_production() && secret.len() < 32 {
                    panic!("CSRF_SECRET must be at least 32 characters in production environment");
                }
                secret
            }
            Err(_) => {
                if environment.is_production() {
                    panic!("CSRF_SECRET environment variable is required in production environment");
                }
                tracing::warn!("CSRF_SECRET not set, using JWT secret as fallback");
                jwt_secret.clone()
            }
        };
        
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
        
        // Determine cluster from RPC URL
        let cluster = if solana_rpc_url.contains("mainnet") {
            "mainnet".to_string()
        } else if solana_rpc_url.contains("testnet") {
            "testnet".to_string()
        } else {
            "devnet".to_string()
        };
        
        // Parse CORS origins
        let cors_origins: Vec<String> = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| {
                if environment.is_development() {
                    "http://localhost:3000,http://localhost:3001,http://127.0.0.1:3000".to_string()
                } else {
                    String::new()
                }
            })
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // HTTPS enforcement - on by default in production
        let enforce_https = env::var("ENFORCE_HTTPS")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or_else(|_| environment.is_production());
        
        // Validate production configuration
        if environment.is_production() {
            if cors_origins.is_empty() {
                panic!("CORS_ORIGINS must be set in production environment");
            }
            tracing::info!("Running in PRODUCTION mode - security features enabled");
        } else if environment.is_staging() {
            tracing::info!("Running in STAGING mode");
        } else {
            tracing::warn!("Running in DEVELOPMENT mode - NOT FOR PRODUCTION USE");
        }
        
        Ok(Self {
            server_addr,
            database_url,
            redis_url,
            solana_rpc_url,
            program_id,
            authority_keypair,
            jwt_secret,
            jwt_expiry,
            rate_limit_requests,
            rate_limit_window_secs,
            log_level,
            cluster,
            environment,
            cors_origins,
            enforce_https,
            csrf_secret,
        })
    }
}
