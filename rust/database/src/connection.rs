//! Database connection management

use crate::pool::ConnectionPool;
use crate::transaction::DatabaseTransaction;
use crate::DatabaseConfig;
use cloudshuttle_error_handling::DatabaseError;
use sqlx::{PgPool, Postgres, Transaction};
use std::time::Duration;

/// Database connection wrapper with common utilities
#[derive(Clone)]
pub struct DatabaseConnection {
    pool: PgPool,
    config: DatabaseConfig,
}

impl DatabaseConnection {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        let config = DatabaseConfig::from_url(database_url)?;
        Self::with_config(config).await
    }

    /// Create a new database connection with custom config
    pub async fn with_config(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        let pool = ConnectionPool::new(config.clone()).await?;
        Ok(Self { pool, config })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get the database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Execute a transaction
    pub async fn transaction<F, Fut, T>(&self, f: F) -> Result<T, DatabaseError>
    where
        F: FnOnce(DatabaseTransaction) -> Fut,
        Fut: std::future::Future<Output = Result<T, DatabaseError>>,
    {
        let tx = self.pool.begin().await?;
        let db_tx = DatabaseTransaction::new(tx);

        match f(db_tx).await {
            Ok(result) => {
                // Transaction will be committed when db_tx is dropped
                Ok(result)
            }
            Err(err) => {
                // Transaction will be rolled back when db_tx is dropped
                Err(err)
            }
        }
    }

    /// Check database connectivity
    pub async fn ping(&self) -> Result<(), DatabaseError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get database statistics
    pub async fn stats(&self) -> Result<DatabaseStats, DatabaseError> {
        let row = sqlx::query!(
            r#"
            SELECT
                count(*) as total_connections,
                count(*) filter (where state = 'idle') as idle_connections,
                count(*) filter (where state = 'busy') as busy_connections
            FROM pg_stat_activity
            WHERE datname = current_database()
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            total_connections: row.total_connections.unwrap_or(0),
            idle_connections: row.idle_connections.unwrap_or(0),
            busy_connections: row.busy_connections.unwrap_or(0),
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_connections: i64,
    pub idle_connections: i64,
    pub busy_connections: i64,
}
