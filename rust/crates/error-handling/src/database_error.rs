//! Database-specific error handling

use serde::{Deserialize, Serialize};

/// Database operation errors
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection error: {message}")]
    Connection { message: String },

    #[error("Query error: {message}")]
    Query { message: String },

    #[error("Transaction error: {message}")]
    Transaction { message: String },

    #[error("Migration error: {message}")]
    Migration { message: String },

    #[error("Constraint violation: {constraint} - {message}")]
    ConstraintViolation { constraint: String, message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Duplicate key: {key}")]
    DuplicateKey { key: String },

    #[error("Timeout: {operation}")]
    Timeout { operation: String },

    #[error("Pool exhausted: {message}")]
    PoolExhausted { message: String },
}

impl DatabaseError {
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Self::Connection { message: message.into() }
    }

    pub fn query<S: Into<String>>(message: S) -> Self {
        Self::Query { message: message.into() }
    }

    pub fn transaction<S: Into<String>>(message: S) -> Self {
        Self::Transaction { message: message.into() }
    }

    pub fn migration<S: Into<String>>(message: S) -> Self {
        Self::Migration { message: message.into() }
    }

    pub fn constraint_violation<S: Into<String>>(constraint: S, message: S) -> Self {
        Self::ConstraintViolation {
            constraint: constraint.into(),
            message: message.into(),
        }
    }

    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        Self::NotFound { resource: resource.into() }
    }

    pub fn duplicate_key<S: Into<String>>(key: S) -> Self {
        Self::DuplicateKey { key: key.into() }
    }

    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        Self::Timeout { operation: operation.into() }
    }

    pub fn pool_exhausted<S: Into<String>>(message: S) -> Self {
        Self::PoolExhausted { message: message.into() }
    }

    pub fn http_status(&self) -> http::StatusCode {
        match self {
            Self::Connection { .. } => http::StatusCode::SERVICE_UNAVAILABLE,
            Self::Query { .. } => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::Transaction { .. } => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::Migration { .. } => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::ConstraintViolation { .. } => http::StatusCode::BAD_REQUEST,
            Self::NotFound { .. } => http::StatusCode::NOT_FOUND,
            Self::DuplicateKey { .. } => http::StatusCode::CONFLICT,
            Self::Timeout { .. } => http::StatusCode::GATEWAY_TIMEOUT,
            Self::PoolExhausted { .. } => http::StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Connection { .. } => "DATABASE_CONNECTION_ERROR",
            Self::Query { .. } => "DATABASE_QUERY_ERROR",
            Self::Transaction { .. } => "DATABASE_TRANSACTION_ERROR",
            Self::Migration { .. } => "DATABASE_MIGRATION_ERROR",
            Self::ConstraintViolation { .. } => "DATABASE_CONSTRAINT_VIOLATION",
            Self::NotFound { .. } => "DATABASE_NOT_FOUND",
            Self::DuplicateKey { .. } => "DATABASE_DUPLICATE_KEY",
            Self::Timeout { .. } => "DATABASE_TIMEOUT",
            Self::PoolExhausted { .. } => "DATABASE_POOL_EXHAUSTED",
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::Connection { .. } => "Database connection failed".to_string(),
            Self::Query { .. } => "Database query failed".to_string(),
            Self::Transaction { .. } => "Database transaction failed".to_string(),
            Self::Migration { .. } => "Database migration failed".to_string(),
            Self::ConstraintViolation { constraint, .. } => format!("Data constraint violation: {}", constraint),
            Self::NotFound { resource } => format!("{} not found", resource),
            Self::DuplicateKey { key } => format!("{} already exists", key),
            Self::Timeout { operation } => format!("Database {} timed out", operation),
            Self::PoolExhausted { .. } => "Database connection pool exhausted".to_string(),
        }
    }
}

/// Database operation result
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Database transaction result
pub type TransactionResult<T> = Result<T, DatabaseError>;

/// Migration result
pub type MigrationResult = Result<(), DatabaseError>;

/// Connection pool metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct PoolMetrics {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub pending_connections: u32,
    pub max_connections: u32,
}

impl PoolMetrics {
    pub fn utilization_percentage(&self) -> f64 {
        if self.max_connections == 0 {
            0.0
        } else {
            (self.active_connections as f64 / self.max_connections as f64) * 100.0
        }
    }

    pub fn available_connections(&self) -> u32 {
        self.max_connections.saturating_sub(self.active_connections)
    }
}

/// Database health status
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub connections: Option<PoolMetrics>,
    pub last_check: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl DatabaseHealth {
    pub fn healthy(response_time_ms: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            response_time_ms,
            connections: None,
            last_check: chrono::Utc::now().to_rfc3339(),
            message: None,
        }
    }

    pub fn degraded<S: Into<String>>(response_time_ms: u64, message: S) -> Self {
        Self {
            status: HealthStatus::Degraded,
            response_time_ms,
            connections: None,
            last_check: chrono::Utc::now().to_rfc3339(),
            message: Some(message.into()),
        }
    }

    pub fn unhealthy<S: Into<String>>(message: S) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            response_time_ms: 0,
            connections: None,
            last_check: chrono::Utc::now().to_rfc3339(),
            message: Some(message.into()),
        }
    }

    pub fn with_connections(mut self, connections: PoolMetrics) -> Self {
        self.connections = Some(connections);
        self
    }
}


