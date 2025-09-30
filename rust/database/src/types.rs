//! Common database types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub connection_timeout_seconds: Option<u64>,
    pub idle_timeout_seconds: Option<u64>,
    pub max_lifetime_seconds: Option<u64>,
}

impl DatabaseConfig {
    /// Create config from database URL
    pub fn from_url(database_url: &str) -> Result<Self, DatabaseConfigError> {
        if database_url.trim().is_empty() {
            return Err(DatabaseConfigError::InvalidUrl("Database URL cannot be empty".to_string()));
        }

        // Basic URL validation for PostgreSQL
        if !database_url.starts_with("postgresql://") && !database_url.starts_with("postgres://") {
            return Err(DatabaseConfigError::InvalidUrl(
                "Database URL must start with postgresql:// or postgres://".to_string()
            ));
        }

        Ok(Self {
            database_url: database_url.to_string(),
            max_connections: None,
            min_connections: None,
            connection_timeout_seconds: None,
            idle_timeout_seconds: None,
            max_lifetime_seconds: None,
        })
    }

    /// Create config from environment variables
    pub fn from_env() -> Result<Self, DatabaseConfigError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| DatabaseConfigError::MissingEnvVar("DATABASE_URL".to_string()))?;

        let mut config = Self::from_url(&database_url)?;

        if let Ok(max_conn) = std::env::var("DATABASE_MAX_CONNECTIONS") {
            config.max_connections = max_conn.parse().ok();
        }

        if let Ok(min_conn) = std::env::var("DATABASE_MIN_CONNECTIONS") {
            config.min_connections = min_conn.parse().ok();
        }

        if let Ok(conn_timeout) = std::env::var("DATABASE_CONNECTION_TIMEOUT_SECONDS") {
            config.connection_timeout_seconds = conn_timeout.parse().ok();
        }

        if let Ok(idle_timeout) = std::env::var("DATABASE_IDLE_TIMEOUT_SECONDS") {
            config.idle_timeout_seconds = idle_timeout.parse().ok();
        }

        if let Ok(max_lifetime) = std::env::var("DATABASE_MAX_LIFETIME_SECONDS") {
            config.max_lifetime_seconds = max_lifetime.parse().ok();
        }

        Ok(config)
    }

    /// Set max connections
    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = Some(max_connections);
        self
    }

    /// Set min connections
    pub fn with_min_connections(mut self, min_connections: u32) -> Self {
        self.min_connections = Some(min_connections);
        self
    }

    /// Set connection timeout
    pub fn with_connection_timeout(mut self, seconds: u64) -> Self {
        self.connection_timeout_seconds = Some(seconds);
        self
    }

    /// Set idle timeout
    pub fn with_idle_timeout(mut self, seconds: u64) -> Self {
        self.idle_timeout_seconds = Some(seconds);
        self
    }

    /// Set max lifetime
    pub fn with_max_lifetime(mut self, seconds: u64) -> Self {
        self.max_lifetime_seconds = Some(seconds);
        self
    }
}

/// Database configuration error
#[derive(Debug, thiserror::Error)]
pub enum DatabaseConfigError {
    #[error("Invalid database URL: {0}")]
    InvalidUrl(String),

    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}

/// Common database entity fields
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BaseEntity {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Soft delete entity fields
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SoftDeleteEntity {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Tenant-scoped entity fields
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TenantEntity {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::Utc,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Versioned entity fields
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VersionedEntity {
    pub id: Uuid,
    pub version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Audit fields for tracking changes
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditEntity {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Uuid,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub updated_by: Uuid,
}

/// Pagination parameters
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_per_page")]
    pub per_page: u32,

    #[serde(default)]
    pub sort_by: Option<String>,

    #[serde(default)]
    pub sort_order: SortOrder,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

/// Sort order
#[derive(Debug, Clone, Deserialize)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Ascending,

    #[serde(rename = "desc")]
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Ascending
    }
}

/// Filter parameters
#[derive(Debug, Clone, Deserialize)]
pub struct FilterParams {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// Filter operators
#[derive(Debug, Clone, Deserialize)]
pub enum FilterOperator {
    #[serde(rename = "eq")]
    Equal,

    #[serde(rename = "ne")]
    NotEqual,

    #[serde(rename = "gt")]
    GreaterThan,

    #[serde(rename = "gte")]
    GreaterThanOrEqual,

    #[serde(rename = "lt")]
    LessThan,

    #[serde(rename = "lte")]
    LessThanOrEqual,

    #[serde(rename = "like")]
    Like,

    #[serde(rename = "in")]
    In,

    #[serde(rename = "nin")]
    NotIn,

    #[serde(rename = "null")]
    IsNull,

    #[serde(rename = "nnull")]
    IsNotNull,
}

/// Query result wrapper
#[derive(Debug, Clone, Serialize)]
pub struct QueryResult<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Single record result
#[derive(Debug, Clone, Serialize)]
pub struct SingleResult<T> {
    pub data: Option<T>,
}
