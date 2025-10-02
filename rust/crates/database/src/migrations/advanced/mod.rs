//! Advanced migrations module - enterprise-grade database migrations.
//!
//! This module provides comprehensive database migration capabilities
//! including rollback support, dependency management, and integrity verification.

pub mod types;
pub mod runner;
pub mod builder;

// Re-export public types and functions
pub use types::*;
pub use runner::*;
pub use builder::*;
