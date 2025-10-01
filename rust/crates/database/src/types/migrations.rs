//! Migration types and utilities

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Migration record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub id: Uuid,
    pub version: String,
    pub name: String,
    pub executed_at: DateTime<Utc>,
    pub success: bool,
    pub execution_time_ms: Option<i64>,
    pub checksum: Option<String>,
}

impl MigrationRecord {
    pub fn new(version: String, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            version,
            name,
            executed_at: Utc::now(),
            success: false,
            execution_time_ms: None,
            checksum: None,
        }
    }

    pub fn mark_successful(mut self, execution_time_ms: i64, checksum: String) -> Self {
        self.success = true;
        self.execution_time_ms = Some(execution_time_ms);
        self.checksum = Some(checksum);
        self
    }

    pub fn mark_failed(mut self) -> Self {
        self.success = false;
        self.execution_time_ms = None;
        self
    }
}

/// Migration status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MigrationStatus {
    Pending,
    Executing,
    Completed,
    Failed,
    RolledBack,
}

/// Migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    pub table_name: String,
    pub directory: String,
    pub allow_out_of_order: bool,
    pub validate_checksums: bool,
    pub create_table_if_missing: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            table_name: "schema_migrations".to_string(),
            directory: "migrations".to_string(),
            allow_out_of_order: false,
            validate_checksums: true,
            create_table_if_missing: true,
        }
    }
}

/// Migration file representation
#[derive(Debug, Clone)]
pub struct MigrationFile {
    pub version: String,
    pub name: String,
    pub up_sql: String,
    pub down_sql: Option<String>,
    pub checksum: String,
}

impl MigrationFile {
    pub fn new(version: String, name: String, up_sql: String) -> Self {
        let checksum = Self::calculate_checksum(&up_sql);
        Self {
            version,
            name,
            up_sql,
            down_sql: None,
            checksum,
        }
    }

    pub fn with_down_sql(mut self, down_sql: String) -> Self {
        self.down_sql = Some(down_sql);
        self
    }

    fn calculate_checksum(sql: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    pub fn validate_checksum(&self, expected: &str) -> bool {
        self.checksum == expected
    }
}

/// Migration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub migration_id: Uuid,
    pub version: String,
    pub status: MigrationStatus,
    pub message: String,
    pub execution_time_ms: Option<i64>,
    pub timestamp: DateTime<Utc>,
}

impl MigrationResult {
    pub fn success(migration_id: Uuid, version: String, execution_time_ms: i64) -> Self {
        Self {
            migration_id,
            version,
            status: MigrationStatus::Completed,
            message: "Migration completed successfully".to_string(),
            execution_time_ms: Some(execution_time_ms),
            timestamp: Utc::now(),
        }
    }

    pub fn failure(migration_id: Uuid, version: String, error: &str) -> Self {
        Self {
            migration_id,
            version,
            status: MigrationStatus::Failed,
            message: format!("Migration failed: {}", error),
            execution_time_ms: None,
            timestamp: Utc::now(),
        }
    }
}

/// Migration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStats {
    pub total_migrations: usize,
    pub applied_migrations: usize,
    pub pending_migrations: usize,
    pub failed_migrations: usize,
    pub last_migration_at: Option<DateTime<Utc>>,
    pub total_execution_time_ms: i64,
}

impl Default for MigrationStats {
    fn default() -> Self {
        Self {
            total_migrations: 0,
            applied_migrations: 0,
            pending_migrations: 0,
            failed_migrations: 0,
            last_migration_at: None,
            total_execution_time_ms: 0,
        }
    }
}

/// Migration direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MigrationDirection {
    Up,
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_record_creation() {
        let record = MigrationRecord::new("001".to_string(), "create_users".to_string());

        assert!(!record.id.is_nil());
        assert_eq!(record.version, "001");
        assert_eq!(record.name, "create_users");
        assert!(!record.success);
        assert!(record.execution_time_ms.is_none());
        assert!(record.checksum.is_none());
    }

    #[test]
    fn test_migration_record_mark_successful() {
        let record = MigrationRecord::new("001".to_string(), "create_users".to_string())
            .mark_successful(150, "abc123".to_string());

        assert!(record.success);
        assert_eq!(record.execution_time_ms, Some(150));
        assert_eq!(record.checksum, Some("abc123".to_string()));
    }

    #[test]
    fn test_migration_config_default() {
        let config = MigrationConfig::default();
        assert_eq!(config.table_name, "schema_migrations");
        assert_eq!(config.directory, "migrations");
        assert!(!config.allow_out_of_order);
        assert!(config.validate_checksums);
        assert!(config.create_table_if_missing);
    }

    #[test]
    fn test_migration_file() {
        let sql = "CREATE TABLE users (id SERIAL PRIMARY KEY);";
        let file = MigrationFile::new("001".to_string(), "create_users".to_string(), sql.to_string());

        assert_eq!(file.version, "001");
        assert_eq!(file.name, "create_users");
        assert_eq!(file.up_sql, sql);
        assert!(file.down_sql.is_none());
        assert!(!file.checksum.is_empty());

        // Test with down SQL
        let file_with_down = file.with_down_sql("DROP TABLE users;".to_string());
        assert_eq!(file_with_down.down_sql, Some("DROP TABLE users;".to_string()));
    }

    #[test]
    fn test_migration_result() {
        let success = MigrationResult::success(Uuid::new_v4(), "001".to_string(), 100);
        assert_eq!(success.status, MigrationStatus::Completed);
        assert!(success.execution_time_ms.is_some());

        let failure = MigrationResult::failure(Uuid::new_v4(), "001".to_string(), "SQL error");
        assert_eq!(failure.status, MigrationStatus::Failed);
        assert!(failure.execution_time_ms.is_none());
        assert!(failure.message.contains("SQL error"));
    }

    #[test]
    fn test_migration_stats() {
        let stats = MigrationStats::default();
        assert_eq!(stats.total_migrations, 0);
        assert_eq!(stats.applied_migrations, 0);
        assert!(stats.last_migration_at.is_none());
    }
}
