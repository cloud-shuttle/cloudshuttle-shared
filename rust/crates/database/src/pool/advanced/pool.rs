//! Advanced PostgreSQL connection pool implementation.
//!
//! This module contains the core AdvancedPgPool struct and its methods
//! for managing database connections with monitoring and health checks.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use sqlx::{PgPool, postgres::PgPoolOptions};
use cloudshuttle_error_handling::database_error::DatabaseResult;

use super::types::*;

/// Advanced PostgreSQL connection pool with monitoring and health checks
#[derive(Clone)]
pub struct AdvancedPgPool {
    pool: PgPool,
    state: Arc<Mutex<PoolState>>,
    semaphore: Arc<Semaphore>,
}

impl AdvancedPgPool {
    /// Create a new advanced connection pool
    pub async fn new(database_url: &str, config: AdvancedPoolConfig) -> DatabaseResult<Self> {
        let mut pool_options = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .max_lifetime(config.max_lifetime)
            .idle_timeout(config.idle_timeout);

        if let Some(ref _query) = config.test_query {
            pool_options = pool_options.test_before_acquire(true);
        }

        let pool = pool_options.connect(database_url).await?;
        let state = Arc::new(Mutex::new(PoolState::new(config.clone())));
        let semaphore = Arc::new(Semaphore::new(config.max_connections as usize));

        let pool_instance = Self {
            pool,
            state,
            semaphore,
        };

        // Start background health check task if enabled
        if pool_instance.get_config().health_check.enabled {
            pool_instance.start_health_monitor();
        }

        Ok(pool_instance)
    }

    /// Get a connection from the pool with metrics tracking
    pub async fn acquire(&self) -> DatabaseResult<sqlx::pool::PoolConnection<sqlx::Postgres>> {
        let _permit = self.semaphore.acquire().await.map_err(|_| {
            let mut state = self.state.lock().unwrap();
            state.metrics.timeout_count += 1;
            state.calculate_health_score();
            sqlx::Error::PoolTimedOut
        })?;

        let start_time = Instant::now();

        let conn = self.pool.acquire().await.map_err(|e| {
            let mut state = self.state.lock().unwrap();
            state.metrics.error_count += 1;
            state.calculate_health_score();
            e
        })?;

        let acquire_time = start_time.elapsed();
        {
            let mut state = self.state.lock().unwrap();
            state.record_acquire_time(acquire_time);
        }

        Ok(conn)
    }

    /// Get current pool metrics
    pub fn metrics(&self) -> PoolMetrics {
        self.state.lock().unwrap().metrics.clone()
    }

    /// Get pool configuration
    pub fn get_config(&self) -> AdvancedPoolConfig {
        self.state.lock().unwrap().config.clone()
    }

    /// Check if pool is healthy
    pub fn is_healthy(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.metrics.health_score > 0.7 && state.consecutive_failures == 0
    }

    /// Get pool utilization as a percentage (0.0 to 1.0)
    pub fn utilization(&self) -> f64 {
        let metrics = self.metrics();
        if metrics.total_connections == 0 {
            0.0
        } else {
            metrics.active_connections as f64 / metrics.total_connections as f64
        }
    }

    /// Force a health check
    pub async fn health_check(&self) -> DatabaseResult<bool> {
        let config = self.get_config();
        let query = &config.health_check.query;

        let mut conn = self.pool.acquire().await?;
        let result = sqlx::query(query)
            .execute(&mut *conn)
            .await;

        let mut state = self.state.lock().unwrap();

        match result {
            Ok(_) => {
                state.consecutive_failures = 0;
                state.last_health_check = Instant::now();
                Ok(true)
            }
            Err(_) => {
                state.consecutive_failures += 1;
                state.metrics.error_count += 1;
                state.calculate_health_score();
                Ok(state.consecutive_failures < config.health_check.max_failures)
            }
        }
    }

    /// Update metrics (should be called periodically)
    pub fn update_metrics(&self) {
        let mut state = self.state.lock().unwrap();
        // Update basic metrics from the actual pool
        // Note: sqlx doesn't expose all internal metrics, so we work with what we have
        state.calculate_health_score();
    }

    /// Start background health monitoring task
    pub fn start_health_monitor(&self) {
        let state: Arc<Mutex<PoolState>> = Arc::clone(&self.state);
        let pool = self.pool.clone();
        let config = self.get_config();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(config.health_check.interval).await;

                let health_check_result = {
                    let config = {
                        let state_guard = state.lock().unwrap();
                        state_guard.config.clone()
                    };
                    let query = &config.health_check.query;
                    let result = tokio::time::timeout(
                        config.health_check.timeout,
                        async {
                            let mut conn = pool.acquire().await?;
                            sqlx::query(query).execute(&mut *conn).await
                        }
                    ).await;

                    match result {
                        Ok(Ok(_)) => Ok(()),
                        _ => Err(()),
                    }
                };

                let mut state_guard = state.lock().unwrap();
                match health_check_result {
                    Ok(_) => {
                        state_guard.consecutive_failures = 0;
                        state_guard.last_health_check = Instant::now();
                    }
                    Err(_) => {
                        state_guard.consecutive_failures += 1;
                        state_guard.metrics.error_count += 1;
                        state_guard.calculate_health_score();
                    }
                }
            }
        });
    }
}
