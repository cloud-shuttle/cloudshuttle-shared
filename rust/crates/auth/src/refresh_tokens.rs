//! Advanced refresh token management with rotation and security features.
//!
//! This module provides enterprise-grade refresh token lifecycle management,
//! including automatic rotation, security validation, and configurable policies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::types::{AuthResult, AuthError};
#[cfg(feature = "observability")]
use cloudshuttle_observability::audit::{audit_auth, AuditResult};
use crate::{Claims, JwtService};

/// Refresh token configuration and policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenConfig {
    /// Maximum lifetime of refresh tokens (seconds)
    pub max_lifetime: u64,

    /// Whether to rotate refresh tokens on use
    pub rotation_enabled: bool,

    /// Maximum number of active refresh tokens per user
    pub max_tokens_per_user: usize,

    /// Whether to revoke all tokens when a security event occurs
    pub revoke_on_security_event: bool,

    /// Family ID for token grouping (for cascade revocation)
    pub family_id: Option<String>,
}

impl Default for RefreshTokenConfig {
    fn default() -> Self {
        Self {
            max_lifetime: 30 * 24 * 60 * 60, // 30 days
            rotation_enabled: true,
            max_tokens_per_user: 5,
            revoke_on_security_event: true,
            family_id: None,
        }
    }
}

/// Stored refresh token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRecord {
    /// Unique token identifier
    pub token_id: String,

    /// User ID associated with the token
    pub user_id: String,

    /// Family ID for cascade operations
    pub family_id: Option<String>,

    /// Token creation timestamp
    pub created_at: u64,

    /// Token expiration timestamp
    pub expires_at: u64,

    /// Device/client identifier
    pub device_id: Option<String>,

    /// IP address when token was issued
    pub ip_address: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Whether this token has been revoked
    pub revoked: bool,

    /// Reason for revocation (if applicable)
    pub revocation_reason: Option<String>,
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    /// The refresh token to use
    pub refresh_token: String,

    /// Optional device/client identifier for tracking
    pub device_id: Option<String>,

    /// Optional scope to request (subset of original)
    pub scope: Option<String>,
}

/// Refresh token response
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    /// New access token
    pub access_token: String,

    /// New refresh token (if rotation enabled)
    pub refresh_token: Option<String>,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// Access token expiration time
    pub expires_in: u64,

    /// Granted scope
    pub scope: Option<String>,
}

/// Refresh token manager with security features
pub struct RefreshTokenManager {
    jwt_service: JwtService,
    config: RefreshTokenConfig,
    token_store: Arc<Mutex<HashMap<String, RefreshTokenRecord>>>,
}

impl RefreshTokenManager {
    /// Create a new refresh token manager
    pub fn new(jwt_service: JwtService, config: RefreshTokenConfig) -> Self {
        Self {
            jwt_service,
            config,
            token_store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new refresh token for a user
    pub fn create_refresh_token(
        &self,
        user_id: &str,
        device_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AuthResult<String> {
        // Check token limit per user
        self.enforce_token_limit(user_id)?;

        // Generate unique token ID
        let token_id = format!("rt_{}_{}", user_id, uuid::Uuid::new_v4());

        // Create claims for refresh token
        let mut claims = Claims::new(user_id.to_string(), "refresh".to_string());
        claims.custom.insert("token_id".to_string(), serde_json::Value::String(token_id.clone()));
        claims.custom.insert("type".to_string(), serde_json::Value::String("refresh".to_string()));

        // Set expiration
        let now = Self::current_timestamp();
        claims.exp = now + self.config.max_lifetime;

        // Create the JWT
        let refresh_token = self.jwt_service.create_token(&claims)?;

        // Store token metadata
        let record = RefreshTokenRecord {
            token_id: token_id.clone(),
            user_id: user_id.to_string(),
            family_id: self.config.family_id.clone(),
            created_at: now,
            expires_at: claims.exp,
            device_id,
            ip_address,
            user_agent,
            revoked: false,
            revocation_reason: None,
        };

        let mut store = self.token_store.lock().unwrap();
        store.insert(token_id, record);

        Ok(refresh_token)
    }

    /// Use a refresh token to generate new tokens
    pub fn refresh_tokens(&self, request: RefreshTokenRequest) -> AuthResult<RefreshTokenResponse> {
        // Validate the refresh token
        let claims = self.jwt_service.validate_token(&request.refresh_token)?;

        // Extract token metadata
        let token_id = claims.custom.get("token_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::InvalidToken("Missing token ID".to_string()))?;

        let token_type = claims.custom.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::InvalidToken("Invalid token type".to_string()))?;

        if token_type != "refresh" {
            return Err(AuthError::InvalidToken("Not a refresh token".to_string()));
        }

        // Check if token is stored and valid
        let mut store = self.token_store.lock().unwrap();
        let record = store.get(token_id)
            .ok_or_else(|| AuthError::TokenNotFound)?;

        if record.revoked {
            #[cfg(feature = "observability")]
            audit_auth("refresh_token_revoked", Some(&claims.sub), AuditResult::Failure);
            return Err(AuthError::TokenRevoked);
        }

        if Self::current_timestamp() > record.expires_at {
            return Err(AuthError::TokenExpired);
        }

        // Create new access token
        let mut access_claims = Claims::new(claims.sub.clone(), "access".to_string());
        access_claims.exp = Self::current_timestamp() + 3600; // 1 hour
        let access_token = self.jwt_service.create_token(&access_claims)?;

        let mut refresh_token = None;

        // Handle token rotation
        if self.config.rotation_enabled {
            // Revoke the old token
            if let Some(record) = store.get_mut(token_id) {
                record.revoked = true;
                record.revocation_reason = Some("Rotated".to_string());
            }

            // Create new refresh token
            refresh_token = Some(self.create_refresh_token(
                &claims.sub,
                request.device_id,
                None, // IP address not provided in refresh request
                None, // User agent not provided in refresh request
            )?);
        }

        #[cfg(feature = "observability")]
        audit_auth("refresh_token_used", Some(&claims.sub), AuditResult::Success);

        Ok(RefreshTokenResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: request.scope,
        })
    }

    /// Revoke a specific refresh token
    pub fn revoke_token(&self, refresh_token: &str) -> AuthResult<()> {
        let claims = self.jwt_service.extract_claims_unchecked(refresh_token)?;

        let token_id = claims.custom.get("token_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::InvalidToken("Missing token ID".to_string()))?;

        let mut store = self.token_store.lock().unwrap();
        if let Some(record) = store.get_mut(token_id) {
            record.revoked = true;
            record.revocation_reason = Some("Manually revoked".to_string());

            #[cfg(feature = "observability")]
            audit_auth("refresh_token_revoked", Some(&claims.sub), AuditResult::Success);
        }

        Ok(())
    }

    /// Revoke all refresh tokens for a user
    pub fn revoke_all_user_tokens(&self, user_id: &str) -> AuthResult<usize> {
        let mut store = self.token_store.lock().unwrap();
        let mut revoked_count = 0;

        for record in store.values_mut() {
            if record.user_id == user_id {
                record.revoked = true;
                record.revocation_reason = Some("User revocation".to_string());
                revoked_count += 1;
            }
        }

        Ok(revoked_count)
    }

    /// Revoke all tokens in a family
    pub fn revoke_token_family(&self, family_id: &str) -> AuthResult<usize> {
        let mut store = self.token_store.lock().unwrap();
        let mut revoked_count = 0;

        for record in store.values_mut() {
            if record.family_id.as_ref() == Some(&family_id.to_string()) {
                record.revoked = true;
                record.revocation_reason = Some("Family revocation".to_string());
                revoked_count += 1;
            }
        }

        Ok(revoked_count)
    }

    /// Get active tokens for a user
    pub fn get_user_active_tokens(&self, user_id: &str) -> Vec<RefreshTokenRecord> {
        let store = self.token_store.lock().unwrap();
        let now = Self::current_timestamp();

        store.values()
            .filter(|record|
                record.user_id == user_id &&
                !record.revoked &&
                record.expires_at > now
            )
            .cloned()
            .collect()
    }

    /// Clean up expired tokens
    pub fn cleanup_expired_tokens(&self) -> usize {
        let mut store = self.token_store.lock().unwrap();
        let now = Self::current_timestamp();
        let initial_count = store.len();

        store.retain(|_, record| record.expires_at > now);

        initial_count - store.len()
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Enforce token limit per user
    fn enforce_token_limit(&self, user_id: &str) -> AuthResult<()> {
        let active_tokens = self.get_user_active_tokens(user_id);

        if active_tokens.len() >= self.config.max_tokens_per_user {
            // Revoke the oldest token to make room
            let oldest_token_id = active_tokens
                .iter()
                .min_by_key(|record| record.created_at)
                .map(|record| record.token_id.clone());

            if let Some(token_id) = oldest_token_id {
                let mut store = self.token_store.lock().unwrap();
                if let Some(record) = store.get_mut(&token_id) {
                    record.revoked = true;
                    record.revocation_reason = Some("Limit exceeded".to_string());
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_refresh_token() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let config = RefreshTokenConfig::default();
        let manager = RefreshTokenManager::new(jwt_service, config);

        let token = manager.create_refresh_token(
            "user123",
            Some("device456".to_string()),
            Some("127.0.0.1".to_string()),
            Some("Test Browser".to_string()),
        ).unwrap();

        assert!(!token.is_empty());

        // Verify token was stored
        let active_tokens = manager.get_user_active_tokens("user123");
        assert_eq!(active_tokens.len(), 1);
        assert_eq!(active_tokens[0].user_id, "user123");
        assert_eq!(active_tokens[0].device_id, Some("device456".to_string()));
    }

    #[test]
    fn test_refresh_tokens_with_rotation() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let config = RefreshTokenConfig {
            rotation_enabled: true,
            ..Default::default()
        };
        let manager = RefreshTokenManager::new(jwt_service.clone(), config);

        // Create initial refresh token
        let refresh_token = manager.create_refresh_token("user123", None, None, None).unwrap();

        // Use it to refresh
        let request = RefreshTokenRequest {
            refresh_token: refresh_token.clone(),
            device_id: None,
            scope: None,
        };

        let response = manager.refresh_tokens(request).unwrap();

        assert!(!response.access_token.is_empty());
        assert!(response.refresh_token.is_some()); // New token due to rotation
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 3600);

        // Original token should be revoked
        let claims = jwt_service.extract_claims_unchecked(&refresh_token).unwrap();
        let token_id = claims.custom["token_id"].as_str().unwrap();

        let store = manager.token_store.lock().unwrap();
        let record = store.get(token_id).unwrap();
        assert!(record.revoked);
        assert_eq!(record.revocation_reason, Some("Rotated".to_string()));
    }

    #[test]
    fn test_refresh_tokens_without_rotation() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let config = RefreshTokenConfig {
            rotation_enabled: false,
            ..Default::default()
        };
        let manager = RefreshTokenManager::new(jwt_service, config);

        // Create initial refresh token
        let refresh_token = manager.create_refresh_token("user123", None, None, None).unwrap();

        // Use it to refresh
        let request = RefreshTokenRequest {
            refresh_token: refresh_token.clone(),
            device_id: None,
            scope: None,
        };

        let response = manager.refresh_tokens(request).unwrap();

        assert!(!response.access_token.is_empty());
        assert!(response.refresh_token.is_none()); // No rotation
        assert_eq!(response.token_type, "Bearer");
    }

    #[test]
    fn test_revoke_token() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let config = RefreshTokenConfig::default();
        let manager = RefreshTokenManager::new(jwt_service.clone(), config);

        let refresh_token = manager.create_refresh_token("user123", None, None, None).unwrap();

        // Verify token is active
        let active_tokens = manager.get_user_active_tokens("user123");
        assert_eq!(active_tokens.len(), 1);

        // Revoke token
        manager.revoke_token(&refresh_token).unwrap();

        // Verify token is revoked
        let active_tokens = manager.get_user_active_tokens("user123");
        assert_eq!(active_tokens.len(), 0);
    }

    #[test]
    fn test_revoke_all_user_tokens() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let config = RefreshTokenConfig {
            max_tokens_per_user: 10, // Allow multiple tokens
            ..Default::default()
        };
        let manager = RefreshTokenManager::new(jwt_service, config);

        // Create multiple tokens
        for i in 0..3 {
            manager.create_refresh_token(&format!("user123_{}", i), None, None, None).unwrap();
        }

        let revoked_count = manager.revoke_all_user_tokens("user123_0").unwrap();
        assert_eq!(revoked_count, 1);

        let active_tokens = manager.get_user_active_tokens("user123_0");
        assert_eq!(active_tokens.len(), 0);
    }

    #[test]
    fn test_token_limit_enforcement() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let config = RefreshTokenConfig {
            max_tokens_per_user: 2,
            ..Default::default()
        };
        let manager = RefreshTokenManager::new(jwt_service, config);

        // Create tokens up to limit
        manager.create_refresh_token("user123", None, None, None).unwrap();
        manager.create_refresh_token("user123", None, None, None).unwrap();

        let active_tokens = manager.get_user_active_tokens("user123");
        assert_eq!(active_tokens.len(), 2);

        // Create one more - should revoke oldest
        manager.create_refresh_token("user123", None, None, None).unwrap();

        let active_tokens = manager.get_user_active_tokens("user123");
        assert_eq!(active_tokens.len(), 2); // Still 2, oldest was revoked
    }

    #[test]
    fn test_cleanup_expired_tokens() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let config = RefreshTokenConfig {
            max_lifetime: 1, // Very short lifetime
            ..Default::default()
        };
        let manager = RefreshTokenManager::new(jwt_service, config);

        manager.create_refresh_token("user123", None, None, None).unwrap();

        // Wait for token to expire
        std::thread::sleep(std::time::Duration::from_secs(2));

        let cleaned_count = manager.cleanup_expired_tokens();
        assert_eq!(cleaned_count, 1);

        let active_tokens = manager.get_user_active_tokens("user123");
        assert_eq!(active_tokens.len(), 0);
    }

    #[test]
    fn test_invalid_refresh_token() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let config = RefreshTokenConfig::default();
        let manager = RefreshTokenManager::new(jwt_service, config);

        let request = RefreshTokenRequest {
            refresh_token: "invalid-token".to_string(),
            device_id: None,
            scope: None,
        };

        let result = manager.refresh_tokens(request);
        assert!(result.is_err());
    }
}
