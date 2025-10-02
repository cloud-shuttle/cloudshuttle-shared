//! Refresh token manager implementation.
//!
//! This module contains the core RefreshTokenManager struct and its
//! methods for managing refresh token lifecycle, rotation, and security.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::types::{AuthResult, AuthError};
#[cfg(feature = "observability")]
use cloudshuttle_observability::audit::{audit_auth, AuditResult};
use crate::{Claims, JwtService};

use super::types::*;

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
    use crate::JwtService;

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
}
