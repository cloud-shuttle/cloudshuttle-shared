//! Advanced database migration framework with rollback support.
//!
//! This module provides enterprise-grade database migration capabilities
//! including version control, rollback support, dependency management,
//! and migration testing.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use cloudshuttle_error_handling::database_error::DatabaseResult;

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

/// Advanced migration runner with rollback and dependency management
pub struct AdvancedMigrationRunner {
    pool: PgPool,
    migrations_table: String,
    migration_path: String,
    state: Arc<Mutex<MigrationState>>,
}

#[derive(Debug)]
struct MigrationState {
    applied_migrations: HashMap<String, MigrationRecord>,
    pending_migrations: Vec<Migration>,
    execution_history: Vec<MigrationResult>,
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

    /// Plan migrations to reach target state
    pub fn plan_migrations(&self, target_id: Option<&str>) -> DatabaseResult<MigrationPlan> {
        let state = self.state.lock().unwrap();

        let mut to_apply = Vec::new();
        let mut to_rollback = Vec::new();

        if let Some(target) = target_id {
            // Find target migration
            if let Some(target_migration) = state.pending_migrations.iter().find(|m| m.id == target) {
                // Apply all migrations up to and including target
                for migration in &state.pending_migrations {
                    if migration.id <= target_migration.id {
                        to_apply.push(migration.clone());
                    }
                }
            } else if let Some(applied_record) = state.applied_migrations.get(target) {
                if applied_record.status == "applied" {
                    // Rollback to target (exclusive)
                    let applied_ids: HashSet<String> = state.applied_migrations.keys().cloned().collect();

                    for migration in &state.pending_migrations {
                        if applied_ids.contains(&migration.id) && migration.id > target.to_string() {
                            to_rollback.push(migration.clone());
                        }
                    }
                }
            }
        } else {
            // Apply all pending migrations
            to_apply = state.pending_migrations.clone();
        }

        Ok(MigrationPlan {
            to_apply,
            to_rollback,
            dry_run: false,
            target_id: target_id.map(|s| s.to_string()),
        })
    }

    /// Execute migration plan
    pub async fn execute_plan(&self, plan: &MigrationPlan) -> DatabaseResult<Vec<MigrationResult>> {
        let mut results = Vec::new();

        // Execute rollbacks first (in reverse order)
        for migration in plan.to_rollback.iter().rev() {
            let result = self.rollback_migration(migration).await?;
            results.push(result);
        }

        // Execute migrations
        for migration in &plan.to_apply {
            let result = self.apply_migration(migration, "system").await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Apply a single migration
    async fn apply_migration(&self, migration: &Migration, applied_by: &str) -> DatabaseResult<MigrationResult> {
        let start_time = std::time::Instant::now();

        let result = sqlx::query(&migration.up_sql)
            .execute(&self.pool)
            .await;

        let execution_time = start_time.elapsed();
        let execution_time_ms = execution_time.as_millis() as i64;

        let (status, error_message, affected_rows) = match &result {
            Ok(result) => {
                // Record successful migration
                self.record_migration_applied(migration, applied_by, execution_time_ms, Some(result.rows_affected() as i64)).await?;
                (MigrationStatus::Applied, None, Some(result.rows_affected() as i64))
            }
            Err(e) => {
                // Record failed migration
                let error_msg = e.to_string();
                self.record_migration_failed(migration, applied_by, execution_time_ms, &error_msg).await?;
                (MigrationStatus::Failed, Some(error_msg), None)
            }
        };

        let migration_result = MigrationResult {
            migration: migration.clone(),
            status,
            execution_time_ms,
            error_message,
            affected_rows,
        };

        // Update state
        let mut state = self.state.lock().unwrap();
        state.execution_history.push(migration_result.clone());

        Ok(migration_result)
    }

    /// Rollback a single migration
    async fn rollback_migration(&self, migration: &Migration) -> DatabaseResult<MigrationResult> {
        let start_time = std::time::Instant::now();

        let result = if let Some(down_sql) = &migration.down_sql {
            sqlx::query(down_sql).execute(&self.pool).await
        } else {
            return Err(sqlx::Error::Configuration(
                format!("Cannot rollback migration {}: no rollback SQL provided", migration.id).into()
            ).into());
        };

        let execution_time = start_time.elapsed();
        let execution_time_ms = execution_time.as_millis() as i64;

        let (status, error_message, affected_rows) = match &result {
            Ok(result) => {
                self.record_migration_rolled_back(migration, execution_time_ms, Some(result.rows_affected() as i64)).await?;
                (MigrationStatus::RolledBack, None, Some(result.rows_affected() as i64))
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.record_migration_rollback_failed(migration, execution_time_ms, &error_msg).await?;
                (MigrationStatus::Failed, Some(error_msg), None)
            }
        };

        let migration_result = MigrationResult {
            migration: migration.clone(),
            status,
            execution_time_ms,
            error_message,
            affected_rows,
        };

        let mut state = self.state.lock().unwrap();
        state.execution_history.push(migration_result.clone());

        Ok(migration_result)
    }

    /// Record successful migration application
    async fn record_migration_applied(
        &self,
        migration: &Migration,
        applied_by: &str,
        execution_time_ms: i64,
        affected_rows: Option<i64>,
    ) -> DatabaseResult<()> {
        let checksum = Self::calculate_checksum(&migration.up_sql);

        let query = format!(
            r#"
            INSERT INTO {} (
                id, name, checksum, applied_by, execution_time_ms, status,
                rollback_sql, description, environment
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            self.migrations_table
        );

        sqlx::query(&query)
            .bind(&migration.id)
            .bind(&migration.name)
            .bind(&checksum)
            .bind(applied_by)
            .bind(execution_time_ms)
            .bind("applied")
            .bind(&migration.down_sql)
            .bind(&migration.description)
            .bind("production")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Record failed migration
    async fn record_migration_failed(
        &self,
        migration: &Migration,
        applied_by: &str,
        execution_time_ms: i64,
        error_message: &str,
    ) -> DatabaseResult<()> {
        let checksum = Self::calculate_checksum(&migration.up_sql);

        let query = format!(
            r#"
            INSERT INTO {} (
                id, name, checksum, applied_by, execution_time_ms, status, description
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            self.migrations_table
        );

        sqlx::query(&query)
            .bind(&migration.id)
            .bind(&migration.name)
            .bind(&checksum)
            .bind(applied_by)
            .bind(execution_time_ms)
            .bind("failed")
            .bind(&format!("Migration failed: {}", error_message))
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Record successful rollback
    async fn record_migration_rolled_back(
        &self,
        migration: &Migration,
        execution_time_ms: i64,
        affected_rows: Option<i64>,
    ) -> DatabaseResult<()> {
        let query = format!(
            r#"
            UPDATE {} SET status = 'rolled_back', execution_time_ms = $1
            WHERE id = $2
            "#,
            self.migrations_table
        );

        sqlx::query(&query)
            .bind(execution_time_ms)
            .bind(&migration.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Record failed rollback
    async fn record_migration_rollback_failed(
        &self,
        migration: &Migration,
        execution_time_ms: i64,
        error_message: &str,
    ) -> DatabaseResult<()> {
        let query = format!(
            r#"
            UPDATE {} SET status = 'rollback_failed',
                          description = CONCAT(description, '; Rollback failed: ', $1),
                          execution_time_ms = $2
            WHERE id = $3
            "#,
            self.migrations_table
        );

        sqlx::query(&query)
            .bind(error_message)
            .bind(execution_time_ms)
            .bind(&migration.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Calculate migration checksum for integrity verification
    fn calculate_checksum(sql: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get current migration status
    pub fn status(&self) -> MigrationStatusSummary {
        let state = self.state.lock().unwrap();

        let applied_count = state.applied_migrations.len();
        let pending_count = state.pending_migrations.len();
        let failed_count = state.applied_migrations.values()
            .filter(|r| r.status == "failed")
            .count();

        MigrationStatusSummary {
            applied_count,
            pending_count,
            failed_count,
            last_applied: state.applied_migrations.values()
                .max_by_key(|r| r.applied_at)
                .cloned(),
        }
    }

    /// Get migration execution history
    pub fn history(&self) -> Vec<MigrationResult> {
        self.state.lock().unwrap().execution_history.clone()
    }

    /// Validate migration dependencies
    pub fn validate_dependencies(&self) -> DatabaseResult<Vec<String>> {
        let state = self.state.lock().unwrap();
        let applied_ids: HashSet<String> = state.applied_migrations.keys().cloned().collect();

        let mut missing_deps = Vec::new();

        for migration in &state.pending_migrations {
            for dep in &migration.dependencies {
                if !applied_ids.contains(dep) {
                    missing_deps.push(format!("Migration {} depends on {} which is not applied", migration.id, dep));
                }
            }
        }

        Ok(missing_deps)
    }
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

/// Migration builder for creating migrations programmatically
pub struct MigrationBuilder {
    migration: Migration,
}

impl MigrationBuilder {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            migration: Migration {
                id: id.into(),
                name: name.into(),
                up_sql: String::new(),
                down_sql: None,
                description: None,
                dependencies: Vec::new(),
                destructive: false,
            },
        }
    }

    pub fn up_sql(mut self, sql: impl Into<String>) -> Self {
        self.migration.up_sql = sql.into();
        self
    }

    pub fn down_sql(mut self, sql: impl Into<String>) -> Self {
        self.migration.down_sql = Some(sql.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.migration.description = Some(desc.into());
        self
    }

    pub fn depends_on(mut self, dep: impl Into<String>) -> Self {
        self.migration.dependencies.push(dep.into());
        self
    }

    pub fn destructive(mut self, destructive: bool) -> Self {
        self.migration.destructive = destructive;
        self
    }

    pub fn build(self) -> Migration {
        self.migration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_builder() {
        let migration = MigrationBuilder::new("001", "create_users_table")
            .up_sql("CREATE TABLE users (id SERIAL PRIMARY KEY)")
            .down_sql("DROP TABLE users")
            .description("Create users table")
            .build();

        assert_eq!(migration.id, "001");
        assert_eq!(migration.name, "create_users_table");
        assert!(migration.up_sql.contains("CREATE TABLE"));
        assert!(migration.down_sql.is_some());
        assert_eq!(migration.description, Some("Create users table".to_string()));
    }

    #[test]
    fn test_checksum_calculation() {
        let sql = "CREATE TABLE test (id INT)";
        let checksum1 = AdvancedMigrationRunner::calculate_checksum(sql);
        let checksum2 = AdvancedMigrationRunner::calculate_checksum(sql);

        // Same SQL should produce same checksum
        assert_eq!(checksum1, checksum2);

        let different_sql = "CREATE TABLE test (id INT, name TEXT)";
        let checksum3 = AdvancedMigrationRunner::calculate_checksum(different_sql);

        // Different SQL should produce different checksum
        assert_ne!(checksum1, checksum3);
    }

    #[tokio::test]
    async fn test_migration_status() {
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
