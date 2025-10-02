//! Connection pool types and data structures.
//!
//! This module contains all the fundamental types used by the advanced
//! connection pooling system including configurations, metrics, and state.

use std::time::{Duration, Instant};

/// Connection pool metrics for monitoring and optimization
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    /// Total number of connections in the pool
    pub total_connections: u32,

    /// Number of idle connections
    pub idle_connections: u32,

    /// Number of active connections
    pub active_connections: u32,

    /// Number of connections waiting for acquisition
    pub pending_connections: u32,

    /// Average connection acquisition time in milliseconds
    pub avg_acquire_time_ms: f64,

    /// Number of connection timeouts
    pub timeout_count: u64,

    /// Number of connection errors
    pub error_count: u64,

    /// Pool health score (0.0 to 1.0)
    pub health_score: f64,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,

    /// Health check interval
    pub interval: Duration,

    /// Connection timeout for health checks
    pub timeout: Duration,

    /// Maximum number of failed health checks before marking pool unhealthy
    pub max_failures: u32,

    /// SQL query to use for health checks
    pub query: String,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            max_failures: 3,
            query: "SELECT 1".to_string(),
        }
    }
}

/// Connection pool configuration with advanced options
#[derive(Debug, Clone)]
pub struct AdvancedPoolConfig {
    /// Maximum number of connections
    pub max_connections: u32,

    /// Minimum number of connections to maintain
    pub min_connections: u32,

    /// Maximum time to wait for a connection
    pub acquire_timeout: Duration,

    /// Maximum connection lifetime
    pub max_lifetime: Duration,

    /// Maximum idle time before connection is closed
    pub idle_timeout: Duration,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Connection test query
    pub test_query: Option<String>,

    /// Enable connection validation on checkout
    pub test_on_checkout: bool,

    /// Enable connection validation on idle
    pub test_on_idle: bool,
}

impl Default for AdvancedPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(30),
            max_lifetime: Duration::from_secs(1800), // 30 minutes
            idle_timeout: Duration::from_secs(600),  // 10 minutes
            health_check: HealthCheckConfig::default(),
            test_query: Some("SELECT 1".to_string()),
            test_on_checkout: false,
            test_on_idle: true,
        }
    }
}

/// Connection pool state tracking
#[derive(Debug)]
pub struct PoolState {
    pub config: AdvancedPoolConfig,
    pub metrics: PoolMetrics,
    pub created_at: Instant,
    pub last_health_check: Instant,
    pub consecutive_failures: u32,
    pub acquire_times: Vec<Duration>,
}

impl PoolState {
    pub fn new(config: AdvancedPoolConfig) -> Self {
        Self {
            config,
            metrics: PoolMetrics {
                total_connections: 0,
                idle_connections: 0,
                active_connections: 0,
                pending_connections: 0,
                avg_acquire_time_ms: 0.0,
                timeout_count: 0,
                error_count: 0,
                health_score: 1.0,
            },
            created_at: Instant::now(),
            last_health_check: Instant::now(),
            consecutive_failures: 0,
            acquire_times: Vec::with_capacity(100),
        }
    }

    pub fn record_acquire_time(&mut self, duration: Duration) {
        self.acquire_times.push(duration);
    }

    pub fn calculate_health_score(&mut self) {
        let mut health_score = 1.0;

        // Penalize high error rates
        let error_rate = if self.metrics.total_connections > 0 {
            self.metrics.error_count as f64 / self.metrics.total_connections as f64
        } else {
            0.0
        };
        health_score *= 1.0 - error_rate.min(0.5); // Cap at 50% penalty

        // Penalize high timeout rates
        let timeout_rate = if self.metrics.total_connections > 0 {
            self.metrics.timeout_count as f64 / self.metrics.total_connections as f64
        } else {
            0.0
        };
        health_score *= 1.0 - timeout_rate.min(0.3); // Cap at 30% penalty

        // Penalize low connection utilization
        let utilization = if self.metrics.total_connections > 0 {
            self.metrics.active_connections as f64 / self.metrics.total_connections as f64
        } else {
            0.0
        };
        if utilization < 0.1 {
            health_score *= 0.8; // 20% penalty for very low utilization
        }

        self.metrics.health_score = health_score.max(0.0);
    }
}
