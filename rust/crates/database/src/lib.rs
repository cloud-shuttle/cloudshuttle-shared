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
pub use connection::DatabaseConnection;
pub use migrations::MigrationRunner;
pub use query::QueryHelper;
pub use transaction::{DatabaseTransaction, TransactionResult};
pub use pool::ConnectionPool;
pub use types::*;
