use anyhow::{Context, Result};
use sqlx::{migrate::MigrateDatabase, PgPool, postgres::PgPoolOptions};
use tracing::info;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Creates a new database connection pool with optimized settings
    pub async fn new(database_url: &str) -> Result<Self> {
        // Verify database exists, create if not (for development)
        if !sqlx::Postgres::database_exists(database_url).await.unwrap_or(false) {
            info!("Database does not exist, creating...");
            sqlx::Postgres::create_database(database_url)
                .await
                .context("Failed to create database")?;
        }

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(database_url)
            .await
            .context("Failed to connect to database")?;
        
        Ok(Self { pool })
    }

    /// Returns a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Runs all pending migrations from the migrations folder
    /// Uses sqlx-cli compatible migration system
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");
        
        // Run migrations from the migrations folder
        // This expects migrations to be in ./migrations/ relative to the binary
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("Failed to run database migrations")?;
        
        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Runs migrations from a custom path (useful for testing or embedded migrations)
    pub async fn migrate_from_path(&self, path: &str) -> Result<()> {
        info!("Running database migrations from: {}", path);
        
        sqlx::migrate::Migrator::new(std::path::Path::new(path))
            .await
            .context("Failed to load migrations from path")?
            .run(&self.pool)
            .await
            .context("Failed to run migrations from path")?;
        
        info!("Database migrations from path completed successfully");
        Ok(())
    }

    /// Checks database connection health
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .context("Database health check failed")?;
        Ok(())
    }

    /// Gracefully closes all database connections
    pub async fn close(&self) {
        self.pool.close().await;
        info!("Database connections closed");
    }
}

// Helper functions for database operations
impl Database {
    /// Logs an audit event to the audit_log table
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

    /// Gets the current schema version from _sqlx_migrations table
    pub async fn get_schema_version(&self) -> Result<Option<i64>> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get schema version")?;
        
        Ok(result.map(|r| r.0))
    }

    /// Checks if a specific migration has been applied
    pub async fn is_migration_applied(&self, version: i64) -> Result<bool> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT version FROM _sqlx_migrations WHERE version = $1"
        )
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to check migration status")?;
        
        Ok(result.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        // This test requires a running database
        // Use DATABASE_URL environment variable
        if let Ok(url) = std::env::var("DATABASE_URL") {
            let db = Database::new(&url).await.expect("Failed to connect");
            db.health_check().await.expect("Health check failed");
        }
    }
}