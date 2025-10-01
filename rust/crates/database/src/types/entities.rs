//! Base entity types and common functionality

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Common database entity fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl BaseEntity {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_timestamp(mut self) -> Self {
        self.updated_at = chrono::Utc::now();
        self
    }
}

impl Default for BaseEntity {
    fn default() -> Self {
        Self::new()
    }
}

/// Soft delete support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftDeleteEntity {
    #[serde(flatten)]
    pub base: BaseEntity,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl SoftDeleteEntity {
    pub fn new() -> Self {
        Self {
            base: BaseEntity::new(),
            deleted_at: None,
        }
    }

    pub fn soft_delete(mut self) -> Self {
        self.deleted_at = Some(chrono::Utc::now());
        self
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

impl Default for SoftDeleteEntity {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_entity_creation() {
        let entity = BaseEntity::new();
        assert!(!entity.id.is_nil());
        assert_eq!(entity.created_at, entity.updated_at);
    }

    #[test]
    fn test_base_entity_update() {
        let entity = BaseEntity::new();
        let original_time = entity.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(1));
        let updated = entity.update_timestamp();
        assert!(updated.updated_at > original_time);
    }

    #[test]
    fn test_soft_delete_entity() {
        let entity = SoftDeleteEntity::new();
        assert!(!entity.is_deleted());
        assert!(entity.deleted_at.is_none());

        let deleted = entity.soft_delete();
        assert!(deleted.is_deleted());
        assert!(deleted.deleted_at.is_some());
    }
}
