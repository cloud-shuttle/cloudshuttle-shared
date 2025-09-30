//! JWT claims structure and management

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Standard JWT claims for CloudShuttle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// Tenant ID
    pub tenant_id: String,

    /// User roles
    #[serde(default)]
    pub roles: Vec<String>,

    /// Permissions
    #[serde(default)]
    pub permissions: Vec<String>,

    /// Token type (access, refresh, etc.)
    pub token_type: Option<String>,

    /// Issuer
    pub iss: Option<String>,

    /// Audience
    pub aud: Option<String>,

    /// Expiration time
    pub exp: u64,

    /// Issued at
    pub iat: u64,

    /// Not before
    pub nbf: Option<u64>,

    /// JWT ID
    pub jti: Option<String>,

    /// Custom claims
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl Claims {
    /// Create new claims with basic subject and tenant
    pub fn new<S: Into<String>>(subject: S, tenant_id: S) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            sub: subject.into(),
            tenant_id: tenant_id.into(),
            roles: Vec::new(),
            permissions: Vec::new(),
            token_type: Some("access".to_string()),
            iss: None,
            aud: None,
            exp: now + 3600, // 1 hour default
            iat: now,
            nbf: None,
            jti: Some(Uuid::new_v4().to_string()),
            custom: std::collections::HashMap::new(),
        }
    }

    /// Set expiry time in seconds from now
    pub fn with_expiry(mut self, seconds_from_now: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.exp = now + seconds_from_now;
        self
    }

    /// Set expiry timestamp directly
    pub fn with_expiry_timestamp(mut self, timestamp: u64) -> Self {
        self.exp = timestamp;
        self
    }

    /// Set issued at timestamp
    pub fn with_issued_at(mut self, timestamp: u64) -> Self {
        self.iat = timestamp;
        self
    }

    /// Set not before timestamp
    pub fn with_not_before(mut self, timestamp: u64) -> Self {
        self.nbf = Some(timestamp);
        self
    }

    /// Set issuer
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.iss = Some(issuer.into());
        self
    }

    /// Set audience
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.aud = Some(audience.into());
        self
    }

    /// Set roles
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    /// Add a role
    pub fn add_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// Set permissions
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    /// Add a permission
    pub fn add_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.push(permission.into());
        self
    }

    /// Set token type
    pub fn with_token_type(mut self, token_type: impl Into<String>) -> Self {
        self.token_type = Some(token_type.into());
        self
    }

    /// Set JWT ID
    pub fn with_jwt_id(mut self, jti: impl Into<String>) -> Self {
        self.jti = Some(jti.into());
        self
    }

    /// Add custom claim
    pub fn with_custom_claim<T: serde::Serialize>(
        mut self,
        key: impl Into<String>,
        value: T,
    ) -> serde_json::Result<Self> {
        let json_value = serde_json::to_value(value)?;
        self.custom.insert(key.into(), json_value);
        Ok(self)
    }

    /// Get custom claim
    pub fn get_custom_claim<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Option<T> {
        self.custom.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Check if user has role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Check if user has any of the roles
    pub fn has_any_role(&self, roles: &[String]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Check if user has permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Check if user has any of the permissions
    pub fn has_any_permission(&self, permissions: &[String]) -> bool {
        permissions.iter().any(|perm| self.has_permission(perm))
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.exp < now
    }

    /// Check if token is valid (not expired and not before time passed)
    pub fn is_valid(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check expiry
        if self.exp < now {
            return false;
        }

        // Check not before
        if let Some(nbf) = self.nbf {
            if now < nbf {
                return false;
            }
        }

        true
    }

    /// Get time until expiry in seconds
    pub fn seconds_until_expiry(&self) -> i64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.exp as i64 - now
    }

    /// Get user ID (alias for subject)
    pub fn user_id(&self) -> &str {
        &self.sub
    }

    /// Get tenant ID
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Get roles
    pub fn roles(&self) -> &[String] {
        &self.roles
    }

    /// Get permissions
    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }
}

/// Builder pattern for creating claims
pub struct ClaimsBuilder {
    claims: Claims,
}

impl ClaimsBuilder {
    pub fn new<S: Into<String>>(subject: S, tenant_id: S) -> Self {
        Self {
            claims: Claims::new(subject, tenant_id),
        }
    }

    pub fn with_expiry(mut self, seconds: u64) -> Self {
        self.claims = self.claims.with_expiry(seconds);
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.claims = self.claims.with_roles(roles);
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.claims = self.claims.with_permissions(permissions);
        self
    }

    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.claims = self.claims.with_issuer(issuer);
        self
    }

    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.claims = self.claims.with_audience(audience);
        self
    }

    pub fn with_token_type(mut self, token_type: impl Into<String>) -> Self {
        self.claims = self.claims.with_token_type(token_type);
        self
    }

    pub fn with_custom_claim<T: serde::Serialize>(
        mut self,
        key: impl Into<String>,
        value: T,
    ) -> serde_json::Result<Self> {
        self.claims = self.claims.with_custom_claim(key, value)?;
        self
    }

    pub fn build(self) -> Claims {
        self.claims
    }
}

/// Claims extensions for domain-specific functionality
pub trait ClaimsExt {
    fn is_admin(&self) -> bool;
    fn is_tenant_admin(&self) -> bool;
    fn can_manage_users(&self) -> bool;
    fn can_access_tenant(&self, tenant_id: &str) -> bool;
}

impl ClaimsExt for Claims {
    fn is_admin(&self) -> bool {
        self.has_role("admin") || self.has_role("super_admin")
    }

    fn is_tenant_admin(&self) -> bool {
        self.has_role("tenant_admin") || self.is_admin()
    }

    fn can_manage_users(&self) -> bool {
        self.has_role("user_manager") || self.is_admin()
    }

    fn can_access_tenant(&self, tenant_id: &str) -> bool {
        self.tenant_id == tenant_id || self.is_admin()
    }
}
