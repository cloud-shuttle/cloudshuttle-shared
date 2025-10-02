//! Migration types and data structures.
//!
//! This module contains all the fundamental types used by the migration system
//! including status enums, records, plans, and execution results.

use chrono::{DateTime, Utc};

/// Migration status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationStatus {
    /// Migration is pending execution
    Pending,
    /// Migration has been successfully applied
    Applied,
    /// Migration failed to apply
    Failed,
    /// Migration has been rolled back
    RolledBack,
}

/// Migration metadata stored in database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MigrationRecord {
    /// Unique migration ID
    pub id: String,
    /// Migration name/version
    pub name: String,
    /// Migration checksum for integrity verification
    pub checksum: String,
    /// When the migration was applied
    pub applied_at: DateTime<Utc>,
    /// Who applied the migration
    pub applied_by: String,
    /// Execution time in milliseconds
    pub execution_time_ms: i64,
    /// Migration status
    pub status: String,
    /// Rollback SQL (if available)
    pub rollback_sql: Option<String>,
    /// Migration description
    pub description: Option<String>,
    /// Environment where migration was applied
    pub environment: Option<String>,
}

/// Migration definition
#[derive(Debug, Clone)]
pub struct Migration {
    /// Migration ID (should be unique and sortable)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Up migration SQL
    pub up_sql: String,
    /// Down migration SQL (optional, for rollback)
    pub down_sql: Option<String>,
    /// Migration description
    pub description: Option<String>,
    /// Dependencies (other migration IDs this migration depends on)
    pub dependencies: Vec<String>,
    /// Whether this migration is destructive (cannot be safely rolled back)
    pub destructive: bool,
}

/// Migration execution result
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Migration that was executed
    pub migration: Migration,
    /// Execution status
    pub status: MigrationStatus,
    /// Execution time in milliseconds
    pub execution_time_ms: i64,
    /// Error message if execution failed
    pub error_message: Option<String>,
    /// Number of affected rows (for informational purposes)
    pub affected_rows: Option<i64>,
}

/// Migration plan for execution
#[derive(Debug, Clone)]
pub struct MigrationPlan {
    /// Migrations to be applied
    pub to_apply: Vec<Migration>,
    /// Migrations to be rolled back
    pub to_rollback: Vec<Migration>,
    /// Whether this is a dry run
    pub dry_run: bool,
    /// Target migration ID (for partial migrations)
    pub target_id: Option<String>,
}

/// Migration status summary
#[derive(Debug, Clone)]
pub struct MigrationStatusSummary {
    /// Number of applied migrations
    pub applied_count: usize,
    /// Number of pending migrations
    pub pending_count: usize,
    /// Number of failed migrations
    pub failed_count: usize,
    /// Last applied migration
    pub last_applied: Option<MigrationRecord>,
}
