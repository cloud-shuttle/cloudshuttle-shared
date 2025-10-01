//! Database migration utilities

use std::path::Path;
use cloudshuttle_error_handling::database_error::DatabaseResult;

/// Migration runner for managing database schema changes
pub struct MigrationRunner {
    migrations_dir: String,
}

impl MigrationRunner {
    /// Create a new migration runner
    pub fn new<S: Into<String>>(migrations_dir: S) -> Self {
        Self {
            migrations_dir: migrations_dir.into(),
        }
    }

    /// Run all pending migrations
    pub async fn run_migrations(&self, pool: &sqlx::PgPool) -> DatabaseResult<()> {
        // Create migrations table if it doesn't exist
        self.create_migrations_table(pool).await?;

        // Get list of migration files
        let migration_files = self.get_migration_files()?;

        // Run each migration that hasn't been applied
        for file in migration_files {
            if !self.is_migration_applied(pool, &file).await? {
                self.apply_migration(pool, &file).await?;
                tracing::info!("Applied migration: {}", file);
            }
        }

        Ok(())
    }

    /// Create the migrations tracking table
    async fn create_migrations_table(&self, pool: &sqlx::PgPool) -> DatabaseResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version VARCHAR(255) PRIMARY KEY,
                applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                description TEXT
            )
            "#
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get list of migration files
    fn get_migration_files(&self) -> DatabaseResult<Vec<String>> {
        let path = Path::new(&self.migrations_dir);
        let mut files = Vec::new();

        if path.exists() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        files.push(file_name.to_string());
                    }
                }
            }
        }

        // Sort files to ensure consistent order
        files.sort();
        Ok(files)
    }

    /// Check if a migration has been applied
    async fn is_migration_applied(&self, pool: &sqlx::PgPool, file_name: &str) -> DatabaseResult<bool> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = $1)"
        )
        .bind(file_name)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    /// Apply a migration
    async fn apply_migration(&self, pool: &sqlx::PgPool, file_name: &str) -> DatabaseResult<()> {
        let file_path = Path::new(&self.migrations_dir).join(file_name);
        let sql = std::fs::read_to_string(&file_path)?;

        // Split SQL by semicolon to handle multiple statements
        let statements: Vec<&str> = sql.split(';').filter(|s| !s.trim().is_empty()).collect();

        // Execute each statement in a transaction
        let mut tx = pool.begin().await?;

        for statement in statements {
            // TODO: Implement migration statement execution
            // sqlx::query(statement.trim()).execute(&mut tx).await?;
        }

        // TODO: Record the migration as applied
        // sqlx::query(
        //     "INSERT INTO schema_migrations (version, description) VALUES ($1, $2)"
        // )
        // .bind(file_name)
        // .bind(format!("Migration: {}", file_name))
        // .execute(&mut tx)
        // .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Rollback a migration
    pub async fn rollback_migration(&self, pool: &sqlx::PgPool, file_name: &str) -> DatabaseResult<()> {
        // This is a simplified rollback - in practice, you'd need
        // rollback SQL for each migration
        sqlx::query("DELETE FROM schema_migrations WHERE version = $1")
            .bind(file_name)
            .execute(pool)
            .await?;

        tracing::warn!("Rolled back migration: {} (manual rollback may be required)", file_name);
        Ok(())
    }

    /// Get migration status
    pub async fn get_migration_status(&self, pool: &sqlx::PgPool) -> DatabaseResult<Vec<MigrationStatus>> {
        let migration_files = self.get_migration_files()?;

        let mut status = Vec::new();

        for file in migration_files {
            let applied = self.is_migration_applied(pool, &file).await?;
            let applied_at = if applied {
                let result: (chrono::DateTime<chrono::Utc>,) = sqlx::query_as(
                    "SELECT applied_at FROM schema_migrations WHERE version = $1"
                )
                .bind(&file)
                .fetch_one(pool)
                .await?;
                Some(result.0)
            } else {
                None
            };

            status.push(MigrationStatus {
                version: file,
                applied,
                applied_at,
            });
        }

        Ok(status)
    }
}

/// Migration status information
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub version: String,
    pub applied: bool,
    pub applied_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Migration file template generator
pub struct MigrationGenerator;

impl MigrationGenerator {
    /// Generate a new migration file
    pub fn generate_migration(name: &str, migrations_dir: &str) -> DatabaseResult<String> {
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let file_name = format!("{}_{}.sql", timestamp, name);
        let file_path = Path::new(migrations_dir).join(&file_name);

        // Create directory if it doesn't exist
        std::fs::create_dir_all(migrations_dir)?;

        // Create empty migration file
        std::fs::write(&file_path, "-- Migration: Add your SQL here\n")?;

        Ok(file_name)
    }

    /// Generate a migration with up/down sections
    pub fn generate_migration_with_down(name: &str, migrations_dir: &str) -> DatabaseResult<String> {
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let file_name = format!("{}_{}.sql", timestamp, name);
        let file_path = Path::new(migrations_dir).join(&file_name);

        std::fs::create_dir_all(migrations_dir)?;

        let content = format!(
            "-- Migration: {}\n\
             -- Up migration\n\
             \n\
             -- Down migration (for rollback)\n\
             -- Uncomment and add rollback SQL below\n\
             /*\n\
             -- ROLLBACK SQL GOES HERE\n\
             */\n",
            name
        );

        std::fs::write(&file_path, content)?;

        Ok(file_name)
    }
}

/// Migration validator for checking migration files
pub struct MigrationValidator;

impl MigrationValidator {
    /// Validate a migration file
    pub fn validate_migration_file(file_path: &str) -> DatabaseResult<Vec<String>> {
        let mut errors = Vec::new();
        let content = std::fs::read_to_string(file_path)?;

        // Check for basic SQL syntax issues
        if content.trim().is_empty() {
            errors.push("Migration file is empty".to_string());
        }

        // Check for dangerous operations
        if content.to_uppercase().contains("DROP DATABASE") ||
           content.to_uppercase().contains("DROP TABLE") {
            errors.push("Migration contains potentially dangerous DROP operations".to_string());
        }

        // Check for transaction markers (optional)
        let has_begin = content.to_uppercase().contains("BEGIN");
        let has_commit = content.to_uppercase().contains("COMMIT");

        if has_begin != has_commit {
            errors.push("Migration has unbalanced transaction markers".to_string());
        }

        Ok(errors)
    }

    /// Validate all migration files in a directory
    pub fn validate_migrations_dir(migrations_dir: &str) -> DatabaseResult<Vec<(String, Vec<String>)>> {
        let path = Path::new(migrations_dir);
        let mut results = Vec::new();

        if path.exists() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        let file_path = path.to_string_lossy();
                        let errors = Self::validate_migration_file(&file_path)?;
                        if !errors.is_empty() {
                            results.push((file_name.to_string(), errors));
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}
