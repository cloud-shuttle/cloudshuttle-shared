//! Connection pool manager for multiple databases.
//!
//! This module contains the PoolManager struct for managing multiple
//! database connection pools across different databases.

use std::collections::HashMap;
use cloudshuttle_error_handling::database_error::DatabaseResult;

use super::types::*;
use super::pool::*;

/// Connection pool manager for multiple databases
pub struct PoolManager {
    pools: HashMap<String, AdvancedPgPool>,
}

impl PoolManager {
    /// Create a new pool manager
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
        }
    }

    /// Add a database pool
    pub async fn add_pool(
        &mut self,
        name: impl Into<String>,
        database_url: &str,
        config: AdvancedPoolConfig,
    ) -> DatabaseResult<()> {
        let pool = AdvancedPgPool::new(database_url, config).await?;
        self.pools.insert(name.into(), pool);
        Ok(())
    }

    /// Get a pool by name
    pub fn get_pool(&self, name: &str) -> Option<&AdvancedPgPool> {
        self.pools.get(name)
    }

    /// Get all pool metrics
    pub fn all_metrics(&self) -> HashMap<String, PoolMetrics> {
        self.pools
            .iter()
            .map(|(name, pool)| (name.clone(), pool.metrics()))
            .collect()
    }

    /// Check health of all pools
    pub fn health_status(&self) -> HashMap<String, bool> {
        self.pools
            .iter()
            .map(|(name, pool)| (name.clone(), pool.is_healthy()))
            .collect()
    }

    /// Update metrics for all pools
    pub fn update_all_metrics(&self) {
        for pool in self.pools.values() {
            pool.update_metrics();
        }
    }
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_manager() {
        let manager = PoolManager::new();
        let metrics = manager.all_metrics();
        assert!(metrics.is_empty());

        let health = manager.health_status();
        assert!(health.is_empty());
    }
}
