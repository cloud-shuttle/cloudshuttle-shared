//! # CloudShuttle Database Utilities
//!
//! Common database utilities, connection management, and query helpers used across CloudShuttle services.
//!
//! ## Features
//!
//! - Database connection management and pooling
//! - Query builder utilities and helpers
//! - Transaction management
//! - Migration support
//! - Health monitoring
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_database::{DatabaseConnection, QueryHelper};
//!
//! // Create database connection
//! let db = DatabaseConnection::new("postgresql://...").await?;
//!
//! // Use query helpers
//! let user = db.find_by_id::<User>("users", user_id).await?;
//!
//! // Use transactions
//! let result = db.transaction(|tx| async move {
//!     // Perform database operations within transaction
//!     tx.execute("INSERT INTO users ...", &[]).await?;
//!     Ok(())
//! }).await;
//! ```

pub mod connection;
pub mod migrations;
pub mod query;
pub mod transaction;
pub mod pool;
pub mod types;

// Re-export main types
pub use connection::{DatabaseConnection, DatabaseConfig};
pub use migrations::MigrationRunner;
pub use connection::QueryHelper;
pub use transaction::{DatabaseTransaction, TransactionResult};
pub use pool::{ConnectionPool, PoolConfig};

// Re-export advanced pool types
pub use pool::advanced::{AdvancedPgPool, AdvancedPoolConfig, PoolMetrics, PoolManager, HealthCheckConfig};

// Re-export advanced migration types
pub use migrations::advanced::{
    AdvancedMigrationRunner, Migration, MigrationStatus, MigrationRecord,
    MigrationResult, MigrationPlan, MigrationBuilder, MigrationStatusSummary
};

// Re-export commonly used types from modular structure
pub use types::entities::{BaseEntity, SoftDeleteEntity};
pub use types::models::{DatabaseHealth, HealthStatus, QueryCriteria, Pagination};
pub use types::traits::{Repository, QueryRepository, TransactionalRepository};
