//! Repository traits for database operations

use async_trait::async_trait;
use cloudshuttle_error_handling::database_error::DatabaseResult;
use uuid::Uuid;

/// Basic CRUD operations
#[async_trait]
pub trait Repository<T, ID = Uuid> {
    async fn find_by_id(&self, id: ID) -> DatabaseResult<Option<T>>;
    async fn save(&self, entity: T) -> DatabaseResult<T>;
    async fn delete(&self, id: ID) -> DatabaseResult<bool>;
}

/// Query operations
#[async_trait]
pub trait QueryRepository<T> {
    async fn find_all(&self) -> DatabaseResult<Vec<T>>;
    async fn find_by_criteria(&self, criteria: crate::types::models::QueryCriteria) -> DatabaseResult<Vec<T>>;
    async fn count(&self) -> DatabaseResult<i64>;
}

/// Batch operations
#[async_trait]
pub trait BatchRepository<T> {
    async fn save_batch(&self, entities: Vec<T>) -> DatabaseResult<Vec<T>>;
    async fn delete_batch(&self, ids: Vec<Uuid>) -> DatabaseResult<usize>;
}

/// Transaction operations
#[async_trait]
pub trait TransactionalRepository<T> {
    async fn execute_in_transaction<F, Fut, R>(&self, f: F) -> DatabaseResult<R>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = DatabaseResult<R>> + Send;
}

/// Soft delete operations
#[async_trait]
pub trait SoftDeleteRepository<T> {
    async fn soft_delete(&self, id: Uuid) -> DatabaseResult<bool>;
    async fn restore(&self, id: Uuid) -> DatabaseResult<bool>;
    async fn find_deleted(&self) -> DatabaseResult<Vec<T>>;
}

/// Versioned entity operations
#[async_trait]
pub trait VersionedRepository<T, ID = Uuid> {
    async fn save_versioned(&self, entity: T, expected_version: i32) -> DatabaseResult<T>;
    async fn get_version(&self, id: ID) -> DatabaseResult<i32>;
}

/// Tenant-aware repository trait
#[async_trait]
pub trait TenantRepository<T, ID = Uuid> {
    async fn find_by_tenant(&self, tenant_id: Uuid) -> DatabaseResult<Vec<T>>;
    async fn find_by_tenant_and_id(&self, tenant_id: Uuid, id: ID) -> DatabaseResult<Option<T>>;
}

/// Audit logging repository trait
#[async_trait]
pub trait AuditableRepository<T> {
    async fn find_with_audit(&self, id: Uuid) -> DatabaseResult<Option<(T, Vec<AuditEntry>)>>;
    async fn get_audit_trail(&self, id: Uuid) -> DatabaseResult<Vec<AuditEntry>>;
}

/// Generic audit entry for audit logging
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: String,
    pub action: AuditAction,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub user_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<String>,
}

/// Audit actions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuditAction {
    Created,
    Updated,
    Deleted,
    SoftDeleted,
    Restored,
}

/// Connection management trait
#[async_trait]
pub trait ConnectionManager {
    async fn get_connection(&self) -> DatabaseResult<sqlx::PgConnection>;
    async fn get_pool(&self) -> DatabaseResult<&sqlx::PgPool>;
    async fn health_check(&self) -> DatabaseResult<crate::types::models::DatabaseHealth>;
}

/// Migration management trait
#[async_trait]
pub trait MigrationManager {
    async fn run_migrations(&self) -> DatabaseResult<()>;
    async fn rollback_migration(&self, version: &str) -> DatabaseResult<()>;
    async fn get_migration_status(&self) -> DatabaseResult<Vec<crate::types::migrations::MigrationRecord>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_action_serialization() {
        let action = AuditAction::Created;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"Created\"");

        let deserialized: AuditAction = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, AuditAction::Created));
    }

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            entity_id: Uuid::new_v4(),
            entity_type: "TestEntity".to_string(),
            action: AuditAction::Updated,
            old_values: None,
            new_values: Some(serde_json::json!({"field": "value"})),
            user_id: Some(Uuid::new_v4()),
            timestamp: chrono::Utc::now(),
            ip_address: Some("127.0.0.1".to_string()),
        };

        assert_eq!(entry.entity_type, "TestEntity");
        assert!(matches!(entry.action, AuditAction::Updated));
        assert!(entry.new_values.is_some());
        assert!(entry.user_id.is_some());
        assert!(entry.ip_address.is_some());
    }
}
