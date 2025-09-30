//! Database-specific error types and handling

use crate::ServiceError;
use http::StatusCode;

/// Database operation errors
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection error: {message}")]
    Connection { message: String },

    #[error("Query error: {message}")]
    Query { message: String },

    #[error("Constraint violation: {constraint}")]
    ConstraintViolation { constraint: String },

    #[error("Unique constraint violation: {field}")]
    UniqueViolation { field: String },

    #[error("Foreign key constraint violation: {field}")]
    ForeignKeyViolation { field: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Transaction error: {message}")]
    Transaction { message: String },

    #[error("Migration error: {message}")]
    Migration { message: String },

    #[error("Pool error: {message}")]
    Pool { message: String },
}

impl DatabaseError {
    /// Create a connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// Create a query error
    pub fn query(message: impl Into<String>) -> Self {
        Self::Query {
            message: message.into(),
        }
    }

    /// Create a constraint violation error
    pub fn constraint_violation(constraint: impl Into<String>) -> Self {
        Self::ConstraintViolation {
            constraint: constraint.into(),
        }
    }

    /// Create a unique violation error
    pub fn unique_violation(field: impl Into<String>) -> Self {
        Self::UniqueViolation {
            field: field.into(),
        }
    }

    /// Create a foreign key violation error
    pub fn foreign_key_violation(field: impl Into<String>) -> Self {
        Self::ForeignKeyViolation {
            field: field.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a transaction error
    pub fn transaction(message: impl Into<String>) -> Self {
        Self::Transaction {
            message: message.into(),
        }
    }

    /// Create a migration error
    pub fn migration(message: impl Into<String>) -> Self {
        Self::Migration {
            message: message.into(),
        }
    }

    /// Create a pool error
    pub fn pool(message: impl Into<String>) -> Self {
        Self::Pool {
            message: message.into(),
        }
    }
}

impl ServiceError for DatabaseError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::Connection { .. } => "DATABASE_CONNECTION_ERROR",
            Self::Query { .. } => "DATABASE_QUERY_ERROR",
            Self::ConstraintViolation { .. } => "DATABASE_CONSTRAINT_VIOLATION",
            Self::UniqueViolation { .. } => "DATABASE_UNIQUE_VIOLATION",
            Self::ForeignKeyViolation { .. } => "DATABASE_FOREIGN_KEY_VIOLATION",
            Self::NotFound { .. } => "DATABASE_NOT_FOUND",
            Self::Transaction { .. } => "DATABASE_TRANSACTION_ERROR",
            Self::Migration { .. } => "DATABASE_MIGRATION_ERROR",
            Self::Pool { .. } => "DATABASE_POOL_ERROR",
        }
    }

    fn http_status(&self) -> StatusCode {
        match self {
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::UniqueViolation { .. } | Self::ForeignKeyViolation { .. } | Self::ConstraintViolation { .. } => {
                StatusCode::CONFLICT
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::NotFound { resource } => format!("{} not found", resource),
            Self::UniqueViolation { field } => format!("{} already exists", field),
            Self::ForeignKeyViolation { field } => format!("Invalid reference in {}", field),
            Self::ConstraintViolation { constraint } => format!("Constraint violation: {}", constraint),
            Self::Connection { .. } => "Database connection error. Please try again.".to_string(),
            Self::Query { .. } => "Database query error. Please try again.".to_string(),
            Self::Transaction { .. } => "Database transaction error. Please try again.".to_string(),
            Self::Migration { .. } => "Database migration error. Please try again.".to_string(),
            Self::Pool { .. } => "Database connection pool error. Please try again.".to_string(),
        }
    }
}

#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound {
                resource: "Record".to_string(),
            },
            sqlx::Error::Database(db_err) => {
                let message = db_err.message().to_string();
                if message.contains("unique constraint") || message.contains("UNIQUE constraint") {
                    Self::UniqueViolation {
                        field: "field".to_string(), // Could be parsed from error message
                    }
                } else if message.contains("foreign key constraint") || message.contains("FOREIGN KEY constraint") {
                    Self::ForeignKeyViolation {
                        field: "reference".to_string(), // Could be parsed from error message
                    }
                } else {
                    Self::Query { message }
                }
            }
            sqlx::Error::PoolTimedOut => Self::Pool {
                message: "Connection pool timeout".to_string(),
            },
            _ => Self::Query {
                message: err.to_string(),
            },
        }
    }
}
