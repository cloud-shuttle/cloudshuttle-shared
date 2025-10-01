//! Domain models for database operations

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Query criteria for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCriteria {
    pub filters: Vec<Filter>,
    pub sorting: Vec<SortOrder>,
    pub pagination: Option<Pagination>,
}

impl Default for QueryCriteria {
    fn default() -> Self {
        Self {
            filters: Vec::new(),
            sorting: Vec::new(),
            pagination: None,
        }
    }
}

/// Filter for query criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// Filter operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Like,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

impl std::fmt::Display for FilterOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterOperator::Equal => write!(f, "="),
            FilterOperator::NotEqual => write!(f, "!="),
            FilterOperator::GreaterThan => write!(f, ">"),
            FilterOperator::LessThan => write!(f, "<"),
            FilterOperator::GreaterThanOrEqual => write!(f, ">="),
            FilterOperator::LessThanOrEqual => write!(f, "<="),
            FilterOperator::Like => write!(f, "LIKE"),
            FilterOperator::In => write!(f, "IN"),
            FilterOperator::NotIn => write!(f, "NOT IN"),
            FilterOperator::IsNull => write!(f, "IS NULL"),
            FilterOperator::IsNotNull => write!(f, "IS NOT NULL"),
        }
    }
}

/// Sort order for query criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOrder {
    pub field: String,
    pub direction: SortDirection,
}

/// Sort directions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
    pub offset: Option<u64>,
}

impl Pagination {
    pub fn new(page: u32, page_size: u32) -> Self {
        Self {
            page,
            page_size,
            offset: Some(((page - 1) * page_size) as u64),
        }
    }

    pub fn offset(&self) -> u64 {
        self.offset.unwrap_or(0)
    }

    pub fn limit(&self) -> u32 {
        self.page_size
    }
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u32,
    pub idle_timeout_seconds: Option<u32>,
    pub max_lifetime_seconds: Option<u32>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost:5432/default".to_string(),
            max_connections: 10,
            min_connections: 1,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: Some(300),
            max_lifetime_seconds: Some(3600),
        }
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metrics: Option<HealthMetrics>,
}

impl DatabaseHealth {
    pub fn healthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: message.into(),
            timestamp: chrono::Utc::now(),
            metrics: None,
        }
    }

    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: message.into(),
            timestamp: chrono::Utc::now(),
            metrics: None,
        }
    }

    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: message.into(),
            timestamp: chrono::Utc::now(),
            metrics: None,
        }
    }
}

/// Health status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub connection_count: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_queries: u64,
    pub average_query_time_ms: f64,
}

/// Pool metrics for connection monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMetrics {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub acquired_connections: u32,
    pub pending_connections: u32,
    pub max_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_criteria_default() {
        let criteria = QueryCriteria::default();
        assert!(criteria.filters.is_empty());
        assert!(criteria.sorting.is_empty());
        assert!(criteria.pagination.is_none());
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(2, 20);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.page_size, 20);
        assert_eq!(pagination.offset(), 20); // (2-1) * 20
        assert_eq!(pagination.limit(), 20);
    }

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
        assert_eq!(config.acquire_timeout_seconds, 30);
        assert_eq!(config.idle_timeout_seconds, Some(300));
        assert_eq!(config.max_lifetime_seconds, Some(3600));
    }

    #[test]
    fn test_database_health_constructors() {
        let healthy = DatabaseHealth::healthy("All systems operational");
        assert_eq!(healthy.status, HealthStatus::Healthy);
        assert_eq!(healthy.message, "All systems operational");

        let degraded = DatabaseHealth::degraded("High latency detected");
        assert_eq!(degraded.status, HealthStatus::Degraded);
        assert_eq!(degraded.message, "High latency detected");

        let unhealthy = DatabaseHealth::unhealthy("Database unreachable");
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
        assert_eq!(unhealthy.message, "Database unreachable");
    }

    #[test]
    fn test_filter_creation() {
        let filter = Filter {
            field: "name".to_string(),
            operator: FilterOperator::Equal,
            value: serde_json::json!("test"),
        };

        assert_eq!(filter.field, "name");
        assert!(matches!(filter.operator, FilterOperator::Equal));
        assert_eq!(filter.value, serde_json::json!("test"));
    }

    #[test]
    fn test_sort_order() {
        let sort = SortOrder {
            field: "created_at".to_string(),
            direction: SortDirection::Descending,
        };

        assert_eq!(sort.field, "created_at");
        assert!(matches!(sort.direction, SortDirection::Descending));
    }
}
