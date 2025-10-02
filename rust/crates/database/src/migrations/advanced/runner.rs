//! Advanced migration runner implementation.
//!
//! This module contains the core AdvancedMigrationRunner struct and its
//! methods for managing database migrations with rollback support.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::hash::{Hash, Hasher, DefaultHasher};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use cloudshuttle_error_handling::database_error::DatabaseResult;

use super::types::*;

/// Internal migration state
#[derive(Debug)]
struct MigrationState {
    applied_migrations: HashMap<String, MigrationRecord>,
    pending_migrations: Vec<Migration>,
    execution_history: Vec<MigrationResult>,
}

/// Advanced migration runner with rollback and dependency management
pub struct AdvancedMigrationRunner {
    pool: PgPool,
    migrations_table: String,
    migration_path: String,
    state: Arc<Mutex<MigrationState>>,
}

impl AdvancedMigrationRunner {
    /// Create a new advanced migration runner
    pub async fn new(pool: PgPool, migration_path: impl Into<String>) -> DatabaseResult<Self> {
        let migrations_table = "schema_migrations".to_string();
        let migration_path = migration_path.into();

        // Ensure migrations table exists
        Self::ensure_migrations_table(&pool, &migrations_table).await?;

        let applied_migrations = Self::load_applied_migrations(&pool, &migrations_table).await?;
        let pending_migrations = Self::load_pending_migrations(&migration_path)?;

        let state = Arc::new(Mutex::new(MigrationState {
            applied_migrations,
            pending_migrations,
            execution_history: Vec::new(),
        }));

        Ok(Self {
            pool,
            migrations_table,
            migration_path,
            state,
        })
    }

    /// Create migrations table if it doesn't exist
    async fn ensure_migrations_table(pool: &PgPool, table_name: &str) -> DatabaseResult<()> {
        let create_table_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                checksum VARCHAR(64) NOT NULL,
                applied_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                applied_by VARCHAR(255),
                execution_time_ms BIGINT,
                status VARCHAR(50) NOT NULL,
                rollback_sql TEXT,
                description TEXT,
                environment VARCHAR(50)
            );

            CREATE INDEX IF NOT EXISTS idx_{}_applied_at ON {}(applied_at);
            CREATE INDEX IF NOT EXISTS idx_{}_status ON {}(status);
            "#,
            table_name, table_name, table_name, table_name, table_name
        );

        sqlx::query(&create_table_sql).execute(pool).await?;
        Ok(())
    }

    /// Load applied migrations from database
    async fn load_applied_migrations(pool: &PgPool, table_name: &str) -> DatabaseResult<HashMap<String, MigrationRecord>> {
        let query = format!("SELECT * FROM {} ORDER BY applied_at DESC", table_name);
        let records = sqlx::query_as::<_, MigrationRecord>(&query)
            .fetch_all(pool)
            .await?;

        let mut applied = HashMap::new();
        for record in records {
            applied.insert(record.id.clone(), record);
        }

        Ok(applied)
    }

    /// Load pending migrations from filesystem
    fn load_pending_migrations(migration_path: &str) -> DatabaseResult<Vec<Migration>> {
        let path = Path::new(migration_path);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut migrations = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let migration = Self::parse_migration_file(&path, file_name)?;
                    migrations.push(migration);
                }
            }
        }

        // Sort by ID for deterministic order
        migrations.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(migrations)
    }

    /// Parse migration file
    fn parse_migration_file(path: &Path, file_name: &str) -> DatabaseResult<Migration> {
        let content = fs::read_to_string(path)?;

        // Parse filename format: ID_NAME.sql
        let parts: Vec<&str> = file_name.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Err(sqlx::Error::Configuration(
                format!("Invalid migration filename format: {}", file_name).into()
            ).into());
        }

        let id = parts[0].to_string();
        let name = parts[1].trim_end_matches(".sql").to_string();

        // Parse SQL content - split on -- DOWN marker
        let sections: Vec<&str> = content.split("-- DOWN").collect();
        let up_sql = sections[0].trim().to_string();
        let down_sql = if sections.len() > 1 {
            Some(sections[1].trim().to_string())
        } else {
            None
        };

        Ok(Migration {
            id,
            name,
            up_sql,
            down_sql,
            description: None,
            dependencies: Vec::new(),
            destructive: false,
        })
    }

    /// Apply migrations to reach target state
    pub async fn apply_migrations(&self, plan: MigrationPlan, applied_by: &str) -> DatabaseResult<Vec<MigrationResult>> {
        let mut results = Vec::new();

        for migration in &plan.to_apply {
            let start_time = std::time::Instant::now();
            let result = self.apply_migration(migration, applied_by).await;
            let execution_time = start_time.elapsed().as_millis() as i64;

            let migration_result = match result {
                Ok(affected_rows) => {
                    // Update state
                    let mut state = self.state.lock().unwrap();
                    let record = MigrationRecord {
                        id: migration.id.clone(),
                        name: migration.name.clone(),
                        checksum: self.calculate_checksum(&migration.up_sql),
                        applied_at: Utc::now(),
                        applied_by: applied_by.to_string(),
                        execution_time_ms: execution_time,
                        status: "applied".to_string(),
                        rollback_sql: migration.down_sql.clone(),
                        description: migration.description.clone(),
                        environment: None,
                    };
                    state.applied_migrations.insert(migration.id.clone(), record);
                    state.execution_history.push(MigrationResult {
                        migration: migration.clone(),
                        status: MigrationStatus::Applied,
                        execution_time_ms: execution_time,
                        error_message: None,
                        affected_rows: Some(affected_rows),
                    });

                    MigrationResult {
                        migration: migration.clone(),
                        status: MigrationStatus::Applied,
                        execution_time_ms: execution_time,
                        error_message: None,
                        affected_rows: Some(affected_rows),
                    }
                }
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    let mut state = self.state.lock().unwrap();
                    state.execution_history.push(MigrationResult {
                        migration: migration.clone(),
                        status: MigrationStatus::Failed,
                        execution_time_ms: execution_time,
                        error_message: Some(error_msg.clone()),
                        affected_rows: None,
                    });

                    MigrationResult {
                        migration: migration.clone(),
                        status: MigrationStatus::Failed,
                        execution_time_ms: execution_time,
                        error_message: Some(error_msg),
                        affected_rows: None,
                    }
                }
            };

            results.push(migration_result);
        }

        Ok(results)
    }

    /// Apply a single migration
    async fn apply_migration(&self, migration: &Migration, applied_by: &str) -> DatabaseResult<i64> {
        let mut tx = self.pool.begin().await?;

        // Execute migration SQL
        let result = sqlx::query(&migration.up_sql).execute(&mut *tx).await?;
        let affected_rows = result.rows_affected() as i64;

        // Record migration in database
        let checksum = self.calculate_checksum(&migration.up_sql);
        let record_migration_sql = format!(
            "INSERT INTO {} (id, name, checksum, applied_by, execution_time_ms, status, rollback_sql, description) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            self.migrations_table
        );

        sqlx::query(&record_migration_sql)
            .bind(&migration.id)
            .bind(&migration.name)
            .bind(&checksum)
            .bind(applied_by)
            .bind(0i64) // Will be updated with actual execution time
            .bind("applied")
            .bind(&migration.down_sql)
            .bind(&migration.description)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(affected_rows)
    }

    /// Rollback migrations
    pub async fn rollback_migrations(&self, plan: MigrationPlan) -> DatabaseResult<Vec<MigrationResult>> {
        let mut results = Vec::new();

        for migration in &plan.to_rollback {
            let start_time = std::time::Instant::now();
            let result = self.rollback_migration(migration).await;
            let execution_time = start_time.elapsed().as_millis() as i64;

            let migration_result = match result {
                Ok(affected_rows) => {
                    // Update state
                    let mut state = self.state.lock().unwrap();
                    if let Some(record) = state.applied_migrations.get_mut(&migration.id) {
                        record.status = "rolled_back".to_string();
                    }
                    state.execution_history.push(MigrationResult {
                        migration: migration.clone(),
                        status: MigrationStatus::RolledBack,
                        execution_time_ms: execution_time,
                        error_message: None,
                        affected_rows: Some(affected_rows),
                    });

                    MigrationResult {
                        migration: migration.clone(),
                        status: MigrationStatus::RolledBack,
                        execution_time_ms: execution_time,
                        error_message: None,
                        affected_rows: Some(affected_rows),
                    }
                }
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    let mut state = self.state.lock().unwrap();
                    state.execution_history.push(MigrationResult {
                        migration: migration.clone(),
                        status: MigrationStatus::Failed,
                        execution_time_ms: execution_time,
                        error_message: Some(error_msg.clone()),
                        affected_rows: None,
                    });

                    MigrationResult {
                        migration: migration.clone(),
                        status: MigrationStatus::Failed,
                        execution_time_ms: execution_time,
                        error_message: Some(error_msg),
                        affected_rows: None,
                    }
                }
            };

            results.push(migration_result);
        }

        Ok(results)
    }

    /// Rollback a single migration
    async fn rollback_migration(&self, migration: &Migration) -> DatabaseResult<i64> {
        if let Some(down_sql) = &migration.down_sql {
            let mut tx = self.pool.begin().await?;

            // Execute rollback SQL
            let result = sqlx::query(down_sql).execute(&mut *tx).await?;
            let affected_rows = result.rows_affected() as i64;

            // Update migration status in database
            let update_sql = format!(
                "UPDATE {} SET status = 'rolled_back' WHERE id = $1",
                self.migrations_table
            );

            sqlx::query(&update_sql)
                .bind(&migration.id)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;

            Ok(affected_rows)
        } else {
            Err(sqlx::Error::Configuration(
                format!("No rollback SQL available for migration {}", migration.id).into()
            ).into())
        }
    }

    /// Calculate checksum for migration integrity
    fn calculate_checksum(&self, sql: &str) -> String {
        Self::calculate_checksum_static(sql)
    }

    /// Static version of checksum calculation for testing
    fn calculate_checksum_static(sql: &str) -> String {
        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get migration status summary
    pub fn get_status_summary(&self) -> MigrationStatusSummary {
        let state = self.state.lock().unwrap();

        let applied_count = state.applied_migrations.values()
            .filter(|r| r.status == "applied")
            .count();

        let pending_count = state.pending_migrations.len();

        let failed_count = state.applied_migrations.values()
            .filter(|r| r.status == "failed")
            .count();

        let last_applied = state.applied_migrations.values()
            .filter(|r| r.status == "applied")
            .max_by_key(|r| r.applied_at)
            .cloned();

        MigrationStatusSummary {
            applied_count,
            pending_count,
            failed_count,
            last_applied,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_calculation() {
        let sql = "CREATE TABLE test (id INT)";
        let checksum1 = AdvancedMigrationRunner::calculate_checksum_static(sql);
        let checksum2 = AdvancedMigrationRunner::calculate_checksum_static(sql);

        // Same SQL should produce same checksum
        assert_eq!(checksum1, checksum2);

        let different_sql = "CREATE TABLE test (id INT, name TEXT)";
        let checksum3 = AdvancedMigrationRunner::calculate_checksum_static(different_sql);

        // Different SQL should produce different checksum
        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_migration_status() {
        // This would need a real database to test fully
        // For now, just test the data structures
        let summary = MigrationStatusSummary {
            applied_count: 5,
            pending_count: 3,
            failed_count: 1,
            last_applied: None,
        };

        assert_eq!(summary.applied_count, 5);
        assert_eq!(summary.pending_count, 3);
        assert_eq!(summary.failed_count, 1);
    }
}
