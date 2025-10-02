//! Migration builder implementation.
//!
//! This module contains the MigrationBuilder for creating migrations programmatically.

use super::types::*;

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
}
