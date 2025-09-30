//! Token refresh functionality

use crate::{JwtService, Claims, AuthResult, AuthError, AuthTokens};
use std::sync::Arc;

/// Token refresh service
pub struct TokenRefresh {
    jwt_service: Arc<JwtService>,
    refresh_token_expiry: u64,
}

impl TokenRefresh {
    /// Create a new token refresh service
    pub fn new(jwt_service: Arc<JwtService>) -> Self {
        Self {
            jwt_service,
            refresh_token_expiry: 604800, // 7 days
        }
    }

    /// Configure refresh token expiry in seconds
    pub fn with_refresh_expiry(mut self, seconds: u64) -> Self {
        self.refresh_token_expiry = seconds;
        self
    }

    /// Refresh access and refresh tokens
    pub async fn refresh_tokens(&self, refresh_token: &str) -> AuthResult<AuthTokens> {
        // Validate the refresh token
        let claims = self.jwt_service.extract_claims_unchecked(refresh_token)?;

        // Verify it's a refresh token
        self.jwt_service.validate_token_type(&claims, "refresh")?;

        // Check if refresh token is expired
        if self.jwt_service.is_token_expired(&claims) {
            return Err(AuthError::RefreshTokenExpired);
        }

        // Generate new tokens
        let access_token = self.jwt_service.create_access_token(
            &claims.sub,
            &claims.tenant_id,
            claims.roles.clone(),
        )?;

        let new_refresh_token = self.jwt_service.create_refresh_token(
            &claims.sub,
            &claims.tenant_id,
        )?;

        Ok(AuthTokens::new(
            access_token,
            new_refresh_token,
            3600, // 1 hour for access token
            self.refresh_token_expiry,
        ))
    }

    /// Validate refresh token without creating new tokens
    pub async fn validate_refresh_token(&self, refresh_token: &str) -> AuthResult<Claims> {
        let claims = self.jwt_service.extract_claims_unchecked(refresh_token)?;
        self.jwt_service.validate_token_type(&claims, "refresh")?;

        if self.jwt_service.is_token_expired(&claims) {
            return Err(AuthError::RefreshTokenExpired);
        }

        Ok(claims)
    }

    /// Revoke a refresh token (would typically involve a database operation)
    pub async fn revoke_refresh_token(&self, _token_id: &str) -> AuthResult<()> {
        // In a real implementation, this would mark the token as revoked in a database
        // For now, just return success
        Ok(())
    }

    /// Check if a refresh token is revoked
    pub async fn is_refresh_token_revoked(&self, _token_id: &str) -> AuthResult<bool> {
        // In a real implementation, this would check a database
        Ok(false)
    }

    /// Get refresh token expiry time
    pub fn refresh_token_expiry(&self) -> u64 {
        self.refresh_token_expiry
    }
}

/// Refresh token store trait for persistent storage
#[async_trait::async_trait]
pub trait RefreshTokenStore {
    /// Store a refresh token
    async fn store_refresh_token(
        &self,
        token_id: &str,
        user_id: &str,
        tenant_id: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> AuthResult<()>;

    /// Validate and retrieve refresh token
    async fn validate_refresh_token(&self, token_id: &str) -> AuthResult<RefreshTokenData>;

    /// Revoke a refresh token
    async fn revoke_refresh_token(&self, token_id: &str) -> AuthResult<()>;

    /// Revoke all refresh tokens for a user
    async fn revoke_user_tokens(&self, user_id: &str) -> AuthResult<()>;

    /// Revoke all refresh tokens for a tenant
    async fn revoke_tenant_tokens(&self, tenant_id: &str) -> AuthResult<()>;

    /// Clean up expired tokens
    async fn cleanup_expired_tokens(&self) -> AuthResult<usize>;
}

/// Refresh token data
#[derive(Debug, Clone)]
pub struct RefreshTokenData {
    pub token_id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub is_revoked: bool,
}

impl RefreshTokenData {
    pub fn new(
        token_id: String,
        user_id: String,
        tenant_id: String,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            token_id,
            user_id,
            tenant_id,
            created_at: chrono::Utc::now(),
            expires_at,
            is_revoked: false,
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked
    }
}

/// In-memory refresh token store (for development/testing)
pub struct InMemoryRefreshTokenStore {
    tokens: std::sync::RwLock<std::collections::HashMap<String, RefreshTokenData>>,
}

impl InMemoryRefreshTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl RefreshTokenStore for InMemoryRefreshTokenStore {
    async fn store_refresh_token(
        &self,
        token_id: &str,
        user_id: &str,
        tenant_id: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> AuthResult<()> {
        let token_data = RefreshTokenData::new(
            token_id.to_string(),
            user_id.to_string(),
            tenant_id.to_string(),
            expires_at,
        );

        let mut tokens = self.tokens.write().unwrap();
        tokens.insert(token_id.to_string(), token_data);
        Ok(())
    }

    async fn validate_refresh_token(&self, token_id: &str) -> AuthResult<RefreshTokenData> {
        let tokens = self.tokens.read().unwrap();
        match tokens.get(token_id) {
            Some(token_data) if token_data.is_valid() => Ok(token_data.clone()),
            Some(_) => Err(AuthError::InvalidRefreshToken),
            None => Err(AuthError::TokenNotFound),
        }
    }

    async fn revoke_refresh_token(&self, token_id: &str) -> AuthResult<()> {
        let mut tokens = self.tokens.write().unwrap();
        if let Some(token_data) = tokens.get_mut(token_id) {
            token_data.is_revoked = true;
        }
        Ok(())
    }

    async fn revoke_user_tokens(&self, user_id: &str) -> AuthResult<()> {
        let mut tokens = self.tokens.write().unwrap();
        for token_data in tokens.values_mut() {
            if token_data.user_id == user_id {
                token_data.is_revoked = true;
            }
        }
        Ok(())
    }

    async fn revoke_tenant_tokens(&self, tenant_id: &str) -> AuthResult<()> {
        let mut tokens = self.tokens.write().unwrap();
        for token_data in tokens.values_mut() {
            if token_data.tenant_id == tenant_id {
                token_data.is_revoked = true;
            }
        }
        Ok(())
    }

    async fn cleanup_expired_tokens(&self) -> AuthResult<usize> {
        let mut tokens = self.tokens.write().unwrap();
        let initial_count = tokens.len();

        tokens.retain(|_, token_data| token_data.is_valid());

        let removed_count = initial_count - tokens.len();
        Ok(removed_count)
    }
}

/// Refresh token service with storage
pub struct RefreshTokenService<S: RefreshTokenStore> {
    jwt_service: Arc<JwtService>,
    token_store: S,
    refresh_token_expiry: u64,
}

impl<S: RefreshTokenStore> RefreshTokenService<S> {
    /// Create a new refresh token service with storage
    pub fn new(jwt_service: Arc<JwtService>, token_store: S) -> Self {
        Self {
            jwt_service,
            token_store,
            refresh_token_expiry: 604800, // 7 days
        }
    }

    /// Issue new access and refresh tokens
    pub async fn issue_tokens(
        &self,
        user_id: &str,
        tenant_id: &str,
        roles: Vec<String>,
    ) -> AuthResult<AuthTokens> {
        // Generate access token
        let access_token = self.jwt_service.create_access_token(
            user_id,
            tenant_id,
            roles,
        )?;

        // Generate refresh token
        let refresh_token = self.jwt_service.create_refresh_token(
            user_id,
            tenant_id,
        )?;

        // Store refresh token
        let claims = self.jwt_service.extract_claims_unchecked(&refresh_token)?;
        let expires_at = chrono::DateTime::from_timestamp(claims.exp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::days(7));

        if let Some(token_id) = &claims.jti {
            self.token_store.store_refresh_token(
                token_id,
                user_id,
                tenant_id,
                expires_at,
            ).await?;
        }

        Ok(AuthTokens::new(
            access_token,
            refresh_token,
            3600, // 1 hour
            self.refresh_token_expiry,
        ))
    }

    /// Refresh tokens using a valid refresh token
    pub async fn refresh_tokens(&self, refresh_token: &str) -> AuthResult<AuthTokens> {
        // Validate refresh token with JWT service
        let claims = self.jwt_service.extract_claims_unchecked(refresh_token)?;
        self.jwt_service.validate_token_type(&claims, "refresh")?;

        // Validate with token store
        if let Some(token_id) = &claims.jti {
            let token_data = self.token_store.validate_refresh_token(token_id).await?;
            if token_data.is_expired() {
                return Err(AuthError::RefreshTokenExpired);
            }
        }

        // Issue new tokens
        self.issue_tokens(&claims.sub, &claims.tenant_id, claims.roles).await
    }

    /// Revoke a refresh token
    pub async fn revoke_token(&self, token_id: &str) -> AuthResult<()> {
        self.token_store.revoke_refresh_token(token_id).await
    }

    /// Revoke all tokens for a user
    pub async fn revoke_user_tokens(&self, user_id: &str) -> AuthResult<()> {
        self.token_store.revoke_user_tokens(user_id).await
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired(&self) -> AuthResult<usize> {
        self.token_store.cleanup_expired_tokens().await
    }
}
