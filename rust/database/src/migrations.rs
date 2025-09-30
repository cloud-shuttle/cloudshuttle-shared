//! Migration utilities

use crate::DatabaseConnection;
use cloudshuttle_error_handling::DatabaseError;
use sqlx::{PgPool, Postgres, Row};
use std::path::Path;

/// Migration runner for managing database schema changes
pub struct MigrationRunner {
    connection: DatabaseConnection,
}

impl MigrationRunner {
    /// Create a new migration runner
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    /// Run all pending migrations
    pub async fn run_migrations(&self, migrations_dir: &Path) -> Result<(), DatabaseError> {
        self.ensure_migrations_table().await?;

        let migrations = self.load_migrations(migrations_dir)?;
        let applied_migrations = self.get_applied_migrations().await?;

        for migration in migrations {
            if !applied_migrations.contains(&migration.version) {
                println!("Running migration: {}", migration.name);
                self.connection
                    .pool()
                    .execute(sqlx::query(&migration.up_sql))
                    .await?;

                self.record_migration(&migration).await?;
                println!("Migration {} completed", migration.name);
            }
        }

        Ok(())
    }

    /// Rollback the last migration
    pub async fn rollback_last(&self, migrations_dir: &Path) -> Result<(), DatabaseError> {
        let applied_migrations = self.get_applied_migrations().await?;

        if let Some(last_migration) = applied_migrations.last() {
            let migrations = self.load_migrations(migrations_dir)?;
            let migration = migrations
                .iter()
                .find(|m| m.version == *last_migration)
                .ok_or_else(|| DatabaseError::migration("Migration file not found"))?;

            println!("Rolling back migration: {}", migration.name);
            self.connection
                .pool()
                .execute(sqlx::query(&migration.down_sql))
                .await?;

            self.remove_migration_record(last_migration).await?;
            println!("Migration {} rolled back", migration.name);
        }

        Ok(())
    }

    /// Get the status of all migrations
    pub async fn status(&self, migrations_dir: &Path) -> Result<MigrationStatus, DatabaseError> {
        let migrations = self.load_migrations(migrations_dir)?;
        let applied_migrations = self.get_applied_migrations().await?;

        let mut pending = Vec::new();
        let mut applied = Vec::new();

        for migration in migrations {
            if applied_migrations.contains(&migration.version) {
                applied.push(migration);
            } else {
                pending.push(migration);
            }
        }

        Ok(MigrationStatus { pending, applied })
    }

    /// Create the migrations table if it doesn't exist
    async fn ensure_migrations_table(&self) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
        )
        .execute(self.connection.pool())
        .await?;

        Ok(())
    }

    /// Get list of applied migration versions
    async fn get_applied_migrations(&self) -> Result<Vec<String>, DatabaseError> {
        let rows = sqlx::query("SELECT version FROM schema_migrations ORDER BY applied_at")
            .fetch_all(self.connection.pool())
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| row.get::<String, _>("version"))
            .collect())
    }

    /// Record a migration as applied
    async fn record_migration(&self, migration: &Migration) -> Result<(), DatabaseError> {
        sqlx::query("INSERT INTO schema_migrations (version, name) VALUES ($1, $2)")
            .bind(&migration.version)
            .bind(&migration.name)
            .execute(self.connection.pool())
            .await?;

        Ok(())
    }

    /// Remove a migration record
    async fn remove_migration_record(&self, version: &str) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM schema_migrations WHERE version = $1")
            .bind(version)
            .execute(self.connection.pool())
            .await?;

        Ok(())
    }

    /// Load migrations from directory
    fn load_migrations(&self, migrations_dir: &Path) -> Result<Vec<Migration>, DatabaseError> {
        if !migrations_dir.exists() {
            return Ok(Vec::new());
        }

        let mut migrations = Vec::new();

        // In a real implementation, you would read SQL files from the directory
        // For now, this is a placeholder

        Ok(migrations)
    }
}

/// Migration information
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: String,
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
}

/// Migration status
#[derive(Debug)]
pub struct MigrationStatus {
    pub pending: Vec<Migration>,
    pub applied: Vec<Migration>,
}

/// Migration builder for creating migrations programmatically
pub struct MigrationBuilder {
    version: String,
    name: String,
    up_sql: String,
    down_sql: String,
}

impl MigrationBuilder {
    /// Create a new migration builder
    pub fn new(version: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            name: name.into(),
            up_sql: String::new(),
            down_sql: String::new(),
        }
    }

    /// Add up migration SQL
    pub fn up(mut self, sql: impl Into<String>) -> Self {
        self.up_sql = sql.into();
        self
    }

    /// Add down migration SQL
    pub fn down(mut self, sql: impl Into<String>) -> Self {
        self.down_sql = sql.into();
        self
    }

    /// Build the migration
    pub fn build(self) -> Migration {
        Migration {
            version: self.version,
            name: self.name,
            up_sql: self.up_sql,
            down_sql: self.down_sql,
        }
    }
}
