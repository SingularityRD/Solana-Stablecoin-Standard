use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::info;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .connect(database_url)
            .await
            .context("Failed to connect to database")?;
        
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");
        
        sqlx::query(r#"
            -- Users table
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                solana_pubkey VARCHAR(44),
                role VARCHAR(50) NOT NULL DEFAULT 'user',
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            -- Stablecoins table
            CREATE TABLE IF NOT EXISTS stablecoins (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                owner_id UUID NOT NULL REFERENCES users(id),
                name VARCHAR(64) NOT NULL,
                symbol VARCHAR(16) NOT NULL,
                decimals SMALLINT NOT NULL DEFAULT 6,
                preset SMALLINT NOT NULL,
                asset_mint VARCHAR(44) NOT NULL,
                stablecoin_pda VARCHAR(44) NOT NULL UNIQUE,
                authority_pubkey VARCHAR(44) NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            -- Roles table
            CREATE TABLE IF NOT EXISTS role_assignments (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                stablecoin_id UUID NOT NULL REFERENCES stablecoins(id),
                account_pubkey VARCHAR(44) NOT NULL,
                role VARCHAR(50) NOT NULL,
                assigned_by UUID NOT NULL REFERENCES users(id),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(stablecoin_id, account_pubkey, role)
            );
            
            -- Minter quotas
            CREATE TABLE IF NOT EXISTS minter_quotas (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                stablecoin_id UUID NOT NULL REFERENCES stablecoins(id),
                minter_pubkey VARCHAR(44) NOT NULL,
                quota BIGINT NOT NULL DEFAULT 0,
                minted_amount BIGINT NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(stablecoin_id, minter_pubkey)
            );
            
            -- Blacklist
            CREATE TABLE IF NOT EXISTS blacklist_entries (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                stablecoin_id UUID NOT NULL REFERENCES stablecoins(id),
                account_pubkey VARCHAR(44) NOT NULL,
                reason TEXT NOT NULL,
                blacklisted_by UUID NOT NULL REFERENCES users(id),
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(stablecoin_id, account_pubkey)
            );
            
            -- Audit log
            CREATE TABLE IF NOT EXISTS audit_log (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                stablecoin_id UUID REFERENCES stablecoins(id),
                user_id UUID REFERENCES users(id),
                action VARCHAR(100) NOT NULL,
                tx_signature VARCHAR(88),
                details JSONB,
                ip_address INET,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            -- API Keys
            CREATE TABLE IF NOT EXISTS api_keys (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id),
                key_hash VARCHAR(255) NOT NULL,
                name VARCHAR(100),
                permissions JSONB,
                last_used_at TIMESTAMPTZ,
                expires_at TIMESTAMPTZ,
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            -- Webhooks
            CREATE TABLE IF NOT EXISTS webhooks (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                stablecoin_id UUID NOT NULL REFERENCES stablecoins(id),
                url VARCHAR(500) NOT NULL,
                events JSONB NOT NULL,
                secret VARCHAR(255),
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
            CREATE INDEX IF NOT EXISTS idx_stablecoins_owner ON stablecoins(owner_id);
            CREATE INDEX IF NOT EXISTS idx_stablecoins_pda ON stablecoins(stablecoin_pda);
            CREATE INDEX IF NOT EXISTS idx_role_assignments_stablecoin ON role_assignments(stablecoin_id);
            CREATE INDEX IF NOT EXISTS idx_minter_quotas_stablecoin ON minter_quotas(stablecoin_id);
            CREATE INDEX IF NOT EXISTS idx_blacklist_stablecoin ON blacklist_entries(stablecoin_id);
            CREATE INDEX IF NOT EXISTS idx_audit_log_stablecoin ON audit_log(stablecoin_id);
            CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id);
            CREATE INDEX IF NOT EXISTS idx_webhooks_stablecoin ON webhooks(stablecoin_id);
        "#)
        .execute(&self.pool)
        .await
        .context("Failed to run migrations")?;
        
        info!("Database migrations completed");
        Ok(())
    }
}

// Helper functions for database operations
impl Database {
    pub async fn log_audit(
        &self,
        stablecoin_id: Option<uuid::Uuid>,
        user_id: Option<uuid::Uuid>,
        action: &str,
        tx_signature: Option<&str>,
        details: Option<serde_json::Value>,
        ip_address: Option<&str>,
    ) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO audit_log (stablecoin_id, user_id, action, tx_signature, details, ip_address)
            VALUES ($1, $2, $3, $4, $5, $6::inet)
        "#)
        .bind(stablecoin_id)
        .bind(user_id)
        .bind(action)
        .bind(tx_signature)
        .bind(details)
        .bind(ip_address)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
