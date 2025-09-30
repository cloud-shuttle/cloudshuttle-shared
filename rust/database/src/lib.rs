//! # CloudShuttle Database Layer
//!
//! Common database utilities, connection management, and query helpers for CloudShuttle services.
//!
//! This crate provides:
//! - Database connection management
//! - Connection pool configuration
//! - Query builder helpers
//! - Transaction management
//! - Migration utilities
//! - Common database types
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_database::{DatabaseConnection, QueryHelper};
//!
//! // Initialize database connection
//! let db = DatabaseConnection::new("postgresql://user:pass@localhost/db").await?;
//!
//! // Use query helpers
//! let user = db.find_by_id::<User>("users", user_id).await?;
//!
//! // Use transactions
//! let result = db.transaction(|tx| async move {
//!     // Perform multiple operations in a transaction
//!     tx.execute("INSERT INTO users ...").await?;
//!     tx.execute("INSERT INTO profiles ...").await?;
//!     Ok(())
//! }).await;
//! ```

pub mod connection;
pub mod migrations;
pub mod query;
pub mod transaction;
pub mod pool;
pub mod types;

// Re-export main types for convenience
pub use connection::DatabaseConnection;
pub use query::QueryHelper;
pub use transaction::DatabaseTransaction;
pub use pool::ConnectionPool;
pub use types::*;
