//! Database connection management

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use cloudshuttle_error_handling::database_error::DatabaseResult;
use crate::DatabaseTransaction;

/// Database connection manager
pub struct DatabaseConnection {
    pool: PgPool,
    config: DatabaseConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost:5432/postgres".to_string(),
            max_connections: 20,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

impl DatabaseConnection {
    /// Create a new database connection with custom config
    pub async fn with_config(config: DatabaseConfig) -> DatabaseResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .connect(&config.url)
            .await
            .map_err(|e| {
                tracing::error!("Failed to connect to database: {}", e);
                cloudshuttle_error_handling::DatabaseError::from(e)
            })?;

        tracing::info!("Connected to database with pool size: {}", config.max_connections);

        Ok(Self { pool, config })
    }

    /// Create a new database connection with default config
    pub async fn new<S: Into<String>>(url: S) -> DatabaseResult<Self> {
        let mut config = DatabaseConfig::default();
        config.url = url.into();
        Self::with_config(config).await
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Execute a transaction with automatic rollback on error
    pub async fn transaction<F, Fut, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(DatabaseTransaction) -> Fut,
        Fut: std::future::Future<Output = DatabaseResult<T>>,
    {
        let tx = self.pool.begin().await?;
        let transaction = DatabaseTransaction::new(tx);

        match f(transaction).await {
            Ok(result) => {
                // Transaction succeeded, commit
                // Note: The transaction is committed when DatabaseTransaction is dropped
                Ok(result)
            }
            Err(e) => {
                // Transaction failed, rollback happens automatically
                tracing::error!("Transaction failed, rolling back: {}", e);
                Err(e)
            }
        }
    }

    /// Check database health
    pub async fn health_check(&self) -> DatabaseResult<DatabaseHealth> {
        let start = std::time::Instant::now();

        let result = sqlx::query("SELECT 1 as health_check")
            .fetch_one(&self.pool)
            .await;

        let response_time = start.elapsed();

        match result {
            Ok(_) => {
                let pool_metrics = self.pool_metrics().await;
                Ok(DatabaseHealth {
                    status: HealthStatus::Healthy,
                    response_time_ms: response_time.as_millis() as u64,
                    connections: Some(pool_metrics),
                    message: None,
                })
            }
            Err(e) => {
                tracing::error!("Database health check failed: {}", e);
                Ok(DatabaseHealth {
                    status: HealthStatus::Unhealthy,
                    response_time_ms: response_time.as_millis() as u64,
                    connections: None,
                    message: Some(format!("Health check failed: {}", e)),
                })
            }
        }
    }

    /// Get current pool metrics
    pub async fn pool_metrics(&self) -> PoolMetrics {
        // Note: sqlx doesn't expose pool metrics directly
        // This is a simplified implementation
        PoolMetrics {
            total_connections: self.config.max_connections,
            idle_connections: 0, // Would need pool introspection
            active_connections: 0, // Would need pool introspection
            pending_connections: 0, // Would need pool introspection
        }
    }

    /// Close the database connection pool
    pub async fn close(&self) {
        self.pool.close().await;
        tracing::info!("Database connection pool closed");
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub connections: Option<PoolMetrics>,
    pub message: Option<String>,
}

/// Health status enum
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Connection pool metrics
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub pending_connections: u32,
}

/// Query helper trait for common database operations
pub trait QueryHelper {
    async fn find_by_id<T>(&self, table: &str, id: &str) -> DatabaseResult<Option<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin;

    async fn exists(&self, table: &str, column: &str, value: &str) -> DatabaseResult<bool>;

    async fn count(&self, table: &str) -> DatabaseResult<i64>;
}

impl QueryHelper for DatabaseConnection {
    async fn find_by_id<T>(&self, table: &str, id: &str) -> DatabaseResult<Option<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let query = format!("SELECT * FROM {} WHERE id = $1", table);
        let result = sqlx::query_as::<_, T>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }

    async fn exists(&self, table: &str, column: &str, value: &str) -> DatabaseResult<bool> {
        let query = format!("SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1)", table, column);
        let result: (bool,) = sqlx::query_as(&query)
            .bind(value)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0)
    }

    async fn count(&self, table: &str) -> DatabaseResult<i64> {
        let query = format!("SELECT COUNT(*) FROM {}", table);
        let result: (i64,) = sqlx::query_as(&query)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0)
    }
}
