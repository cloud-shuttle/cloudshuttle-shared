//! Connection pool management

use crate::DatabaseConfig;
use cloudshuttle_error_handling::DatabaseError;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// Connection pool wrapper with configuration
#[derive(Clone)]
pub struct ConnectionPool {
    pool: PgPool,
}

impl ConnectionPool {
    /// Create a new connection pool with default configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        Self::with_options(config, PoolOptions::default()).await
    }

    /// Create a new connection pool with custom options
    pub async fn with_options(
        config: DatabaseConfig,
        options: PoolOptions,
    ) -> Result<Self, DatabaseError> {
        let pool_options = PgPoolOptions::new()
            .max_connections(options.max_connections)
            .min_connections(options.min_connections)
            .acquire_timeout(Duration::from_secs(options.acquire_timeout_seconds))
            .idle_timeout(Duration::from_secs(options.idle_timeout_seconds))
            .max_lifetime(Duration::from_secs(options.max_lifetime_seconds));

        let pool = pool_options
            .connect(&config.database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Get a reference to the underlying pool
    pub fn inner(&self) -> &PgPool {
        &self.pool
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        // Note: sqlx doesn't provide direct access to pool stats,
        // but we can track this in a real implementation
        PoolStats {
            size: 0, // Would need to track this
            idle: 0,
            used: 0,
        }
    }
}

impl std::ops::Deref for ConnectionPool {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

/// Connection pool options
#[derive(Debug, Clone)]
pub struct PoolOptions {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
}

impl Default for PoolOptions {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_connections: 5,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 600, // 10 minutes
            max_lifetime_seconds: 1800, // 30 minutes
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: u32,
    pub used: u32,
}
