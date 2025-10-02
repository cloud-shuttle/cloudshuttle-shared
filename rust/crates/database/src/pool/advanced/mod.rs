//! Advanced connection pooling module.
//!
//! This module provides enterprise-grade database connection pooling
//! with monitoring, health checks, and multi-database management.

pub mod types;
pub mod pool;
pub mod manager;

// Re-export public types and functions
pub use types::*;
pub use pool::*;
pub use manager::*;
