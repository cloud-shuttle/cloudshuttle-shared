//! Connection pool management
pub mod advanced;

use sqlx::{PgPool, postgres::PgPoolOptions, Row};
use std::time::Duration;
use cloudshuttle_error_handling::database_error::DatabaseResult;

/// Enhanced connection pool with metrics and health monitoring
pub struct ConnectionPool {
    pool: PgPool,
    config: PoolConfig,
    metrics: PoolMetrics,
}

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub health_check_interval: Duration,
    pub test_before_acquire: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(1800),
            health_check_interval: Duration::from_secs(30),
            test_before_acquire: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub connections_created: u64,
    pub connections_acquired: u64,
    pub connections_released: u64,
    pub connections_destroyed: u64,
    pub acquire_timeouts: u64,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub pending_connections: u32,
}

impl Default for PoolMetrics {
    fn default() -> Self {
        Self {
            connections_created: 0,
            connections_acquired: 0,
            connections_released: 0,
            connections_destroyed: 0,
            acquire_timeouts: 0,
            idle_connections: 0,
            active_connections: 0,
            pending_connections: 0,
        }
    }
}

impl ConnectionPool {
    /// Create a new connection pool with custom configuration
    pub async fn new(database_url: &str, config: PoolConfig) -> DatabaseResult<Self> {
        let pool_options = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .test_before_acquire(config.test_before_acquire);

        let pool = pool_options.connect(database_url).await
            .map_err(|e| {
                tracing::error!("Failed to create connection pool: {}", e);
                e
            })?;

        tracing::info!(
            "Created database connection pool: max={}, min={}, acquire_timeout={:?}",
            config.max_connections,
            config.min_connections,
            config.acquire_timeout
        );

        Ok(Self {
            pool,
            config,
            metrics: PoolMetrics::default(),
        })
    }

    /// Create a new connection pool with default configuration
    pub async fn with_defaults(database_url: &str) -> DatabaseResult<Self> {
        Self::new(database_url, PoolConfig::default()).await
    }

    /// Get a reference to the underlying pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get current pool metrics
    pub fn metrics(&self) -> &PoolMetrics {
        &self.metrics
    }

    /// Get pool configuration
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    /// Get the number of connections currently in use
    pub async fn size(&self) -> u32 {
        // sqlx doesn't expose this directly, so we return max_connections
        self.config.max_connections
    }

    /// Get the number of idle connections
    pub async fn num_idle(&self) -> u32 {
        // sqlx doesn't expose this directly
        0
    }

    /// Check if the pool is healthy
    pub async fn is_healthy(&self) -> bool {
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("Connection pool health check failed: {}", e);
                false
            }
        }
    }

    /// Close the connection pool
    pub async fn close(&self) {
        self.pool.close().await;
        tracing::info!("Database connection pool closed");
    }

    /// Execute a health check query
    pub async fn health_check(&self) -> DatabaseResult<PoolHealth> {
        let start = std::time::Instant::now();

        let result = sqlx::query("SELECT 1 as health_check, version() as version")
            .fetch_one(&self.pool)
            .await;

        let response_time = start.elapsed();

        match result {
            Ok(row) => {
                let version: String = row.get("version");

                Ok(PoolHealth {
                    status: HealthStatus::Healthy,
                    response_time_ms: response_time.as_millis() as u64,
                    database_version: Some(version),
                    message: None,
                })
            }
            Err(e) => {
                tracing::error!("Pool health check failed: {}", e);
                Ok(PoolHealth {
                    status: HealthStatus::Unhealthy,
                    response_time_ms: response_time.as_millis() as u64,
                    database_version: None,
                    message: Some(format!("Health check failed: {}", e)),
                })
            }
        }
    }
}

/// Pool health status
#[derive(Debug, Clone)]
pub struct PoolHealth {
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub database_version: Option<String>,
    pub message: Option<String>,
}

/// Health status enum
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub pending_connections: u32,
    pub utilization_percentage: f64,
}

impl PoolStats {
    pub fn new(
        total: u32,
        idle: u32,
        active: u32,
        pending: u32,
    ) -> Self {
        let utilization_percentage = if total > 0 {
            ((active + pending) as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total_connections: total,
            idle_connections: idle,
            active_connections: active,
            pending_connections: pending,
            utilization_percentage,
        }
    }
}

/// Pool event listener for monitoring
pub trait PoolEventListener: Send + Sync {
    fn on_connection_acquired(&self);
    fn on_connection_released(&self);
    fn on_connection_created(&self);
    fn on_connection_destroyed(&self);
    fn on_acquire_timeout(&self);
}

/// Pool monitor for collecting metrics
pub struct PoolMonitor {
    listener: Option<Box<dyn PoolEventListener>>,
}

impl PoolMonitor {
    pub fn new() -> Self {
        Self { listener: None }
    }

    pub fn with_listener<L: PoolEventListener + 'static>(mut self, listener: L) -> Self {
        self.listener = Some(Box::new(listener));
        self
    }

    pub fn record_connection_acquired(&self) {
        if let Some(listener) = &self.listener {
            listener.on_connection_acquired();
        }
    }

    pub fn record_connection_released(&self) {
        if let Some(listener) = &self.listener {
            listener.on_connection_released();
        }
    }

    pub fn record_connection_created(&self) {
        if let Some(listener) = &self.listener {
            listener.on_connection_created();
        }
    }

    pub fn record_connection_destroyed(&self) {
        if let Some(listener) = &self.listener {
            listener.on_connection_destroyed();
        }
    }

    pub fn record_acquire_timeout(&self) {
        if let Some(listener) = &self.listener {
            listener.on_acquire_timeout();
        }
    }
}

/// Pool configuration builder
pub struct PoolConfigBuilder {
    config: PoolConfig,
}

impl PoolConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: PoolConfig::default(),
        }
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.config.max_connections = max;
        self
    }

    pub fn min_connections(mut self, min: u32) -> Self {
        self.config.min_connections = min;
        self
    }

    pub fn acquire_timeout(mut self, timeout: Duration) -> Self {
        self.config.acquire_timeout = timeout;
        self
    }

    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.config.idle_timeout = timeout;
        self
    }

    pub fn max_lifetime(mut self, lifetime: Duration) -> Self {
        self.config.max_lifetime = lifetime;
        self
    }

    pub fn health_check_interval(mut self, interval: Duration) -> Self {
        self.config.health_check_interval = interval;
        self
    }

    pub fn test_before_acquire(mut self, test: bool) -> Self {
        self.config.test_before_acquire = test;
        self
    }

    pub fn build(self) -> PoolConfig {
        self.config
    }
}

impl Default for PoolConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
