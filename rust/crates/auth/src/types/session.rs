//! User session and authentication context structures

use serde::{Deserialize, Serialize};

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl UserSession {
    pub fn new(user_id: String, tenant_id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            user_id,
            tenant_id,
            roles: Vec::new(),
            permissions: Vec::new(),
            session_id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            expires_at: now + chrono::Duration::hours(1),
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn with_expiry(mut self, hours: i64) -> Self {
        self.expires_at = self.created_at + chrono::Duration::hours(hours);
        self
    }

    pub fn with_ip_address(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, agent: String) -> Self {
        self.user_agent = Some(agent);
        self
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    pub fn extend(&mut self, hours: i64) {
        self.expires_at = chrono::Utc::now() + chrono::Duration::hours(hours);
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }

    pub fn can_access_tenant(&self, tenant_id: &str) -> bool {
        self.tenant_id == tenant_id || self.is_admin()
    }

    pub fn time_remaining(&self) -> chrono::Duration {
        self.expires_at.signed_duration_since(chrono::Utc::now())
    }

    pub fn is_near_expiry(&self, threshold_minutes: i64) -> bool {
        let remaining = self.time_remaining();
        remaining.num_minutes() <= threshold_minutes && remaining.num_minutes() > 0
    }
}

/// Authentication context for requests
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
}

impl AuthContext {
    pub fn new(user_id: String, tenant_id: String) -> Self {
        Self {
            user_id,
            tenant_id,
            roles: Vec::new(),
            permissions: Vec::new(),
            session_id: None,
            ip_address: None,
        }
    }

    pub fn from_session(session: &UserSession) -> Self {
        Self {
            user_id: session.user_id.clone(),
            tenant_id: session.tenant_id.clone(),
            roles: session.roles.clone(),
            permissions: session.permissions.clone(),
            session_id: Some(session.session_id.clone()),
            ip_address: session.ip_address.clone(),
        }
    }

    pub fn from_claims(claims: &crate::Claims) -> Self {
        Self {
            user_id: claims.sub.clone(),
            tenant_id: claims.tenant_id.clone(),
            roles: claims.roles.clone(),
            permissions: claims.permissions.clone(),
            session_id: claims.jti.clone(),
            ip_address: None,
        }
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions.iter().all(|perm| self.has_permission(perm))
    }

    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }

    pub fn can_access_tenant(&self, tenant_id: &str) -> bool {
        self.tenant_id == tenant_id || self.is_admin()
    }

    pub fn can_access_resource(&self, resource_tenant_id: &str, required_permissions: &[&str]) -> bool {
        self.can_access_tenant(resource_tenant_id) && self.has_all_permissions(required_permissions)
    }

    /// Get user roles as a comma-separated string
    pub fn roles_string(&self) -> String {
        self.roles.join(",")
    }

    /// Get user permissions as a comma-separated string
    pub fn permissions_string(&self) -> String {
        self.permissions.join(",")
    }
}

/// Session store interface
#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    async fn create_session(&self, session: UserSession) -> Result<String, crate::AuthError>;
    async fn get_session(&self, session_id: &str) -> Result<Option<UserSession>, crate::AuthError>;
    async fn update_session(&self, session: &UserSession) -> Result<(), crate::AuthError>;
    async fn delete_session(&self, session_id: &str) -> Result<(), crate::AuthError>;
    async fn cleanup_expired_sessions(&self) -> Result<usize, crate::AuthError>;
}

/// In-memory session store for testing/development
pub struct InMemorySessionStore {
    sessions: std::sync::RwLock<std::collections::HashMap<String, UserSession>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create_session(&self, session: UserSession) -> Result<String, crate::AuthError> {
        let session_id = session.session_id.clone();
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<UserSession>, crate::AuthError> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions.get(session_id).cloned())
    }

    async fn update_session(&self, session: &UserSession) -> Result<(), crate::AuthError> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.session_id.clone(), session.clone());
        Ok(())
    }

    async fn delete_session(&self, session_id: &str) -> Result<(), crate::AuthError> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(session_id);
        Ok(())
    }

    async fn cleanup_expired_sessions(&self) -> Result<usize, crate::AuthError> {
        let mut sessions = self.sessions.write().unwrap();
        let before = sessions.len();
        sessions.retain(|_, session| !session.is_expired());
        let after = sessions.len();
        Ok(before - after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_session_creation() {
        let session = UserSession::new("user123".to_string(), "tenant1".to_string())
            .with_roles(vec!["user".to_string()])
            .with_permissions(vec!["read".to_string()])
            .with_ip_address("127.0.0.1".to_string());

        assert_eq!(session.user_id, "user123");
        assert_eq!(session.tenant_id, "tenant1");
        assert_eq!(session.roles, vec!["user"]);
        assert_eq!(session.permissions, vec!["read"]);
        assert_eq!(session.ip_address, Some("127.0.0.1".to_string()));
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_expiry() {
        let expired_session = UserSession::new("user123".to_string(), "tenant1".to_string())
            .with_expiry(-1); // Already expired

        assert!(expired_session.is_expired());
    }

    #[test]
    fn test_auth_context() {
        let mut context = AuthContext::new("user123".to_string(), "tenant1".to_string());
        context.roles = vec!["admin".to_string(), "user".to_string()];
        context.permissions = vec!["read".to_string(), "write".to_string()];

        assert!(context.is_admin());
        assert!(context.has_role("user"));
        assert!(context.has_permission("read"));
        assert!(context.can_access_tenant("tenant1"));
        assert!(context.can_access_tenant("tenant2")); // admin can access any tenant
    }

    #[test]
    fn test_session_store() {
        let store = InMemorySessionStore::new();
        let session = UserSession::new("user123".to_string(), "tenant1".to_string());

        // Test synchronous operations (in real async context this would be awaited)
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session_id = store.create_session(session.clone()).await.unwrap();
            let retrieved = store.get_session(&session_id).await.unwrap().unwrap();
            assert_eq!(retrieved.user_id, "user123");

            store.delete_session(&session_id).await.unwrap();
            let deleted = store.get_session(&session_id).await.unwrap();
            assert!(deleted.is_none());
        });
    }

    #[test]
    fn test_session_roles_and_permissions() {
        let session = UserSession::new("user123".to_string(), "tenant1".to_string())
            .with_roles(vec!["admin".to_string(), "moderator".to_string()])
            .with_permissions(vec!["read".to_string(), "write".to_string(), "delete".to_string()]);

        assert!(session.is_admin());
        assert!(session.has_role("moderator"));
        assert!(session.has_permission("delete"));
        assert!(!session.has_role("guest"));
        assert!(!session.has_permission("execute"));
    }
}
