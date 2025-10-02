//! JWT token management - orchestrates all JWT operations
//!
//! This module orchestrates multiple JWT domains through specialized sub-modules:
//! - `token_operations`: Core token creation, validation, and refresh operations
//! - `claims_management`: Claims structure validation and access control
//! - `key_management`: Algorithm selection and cryptographic key management

pub mod token_operations;
pub mod claims_management;
pub mod key_management;

// Re-export for backward compatibility and convenience
pub use token_operations::TokenService;
pub use claims_management::{ClaimsManager, TokenValidation};
pub use key_management::{KeyManager, JwtAlgorithm};

use crate::Claims;
use crate::types::{AuthResult, AuthError};

/// Legacy JWT service for backward compatibility
pub struct JwtService {
    token_service: TokenService,
}

impl JwtService {
    /// Create a new JWT service with HMAC secret (backward compatibility)
    pub fn new(secret: &[u8]) -> AuthResult<Self> {
        let token_service = TokenService::new(secret)?;
        Ok(Self { token_service })
    }

    /// Create a new JWT service with specific algorithm (backward compatibility)
    pub fn with_algorithm(secret: &[u8], algorithm: jsonwebtoken::Algorithm) -> AuthResult<Self> {
        let jwt_algorithm = match algorithm {
            jsonwebtoken::Algorithm::HS256 => JwtAlgorithm::HS256,
            jsonwebtoken::Algorithm::HS384 => JwtAlgorithm::HS384,
            jsonwebtoken::Algorithm::HS512 => JwtAlgorithm::HS512,
            jsonwebtoken::Algorithm::RS256 => JwtAlgorithm::RS256,
            jsonwebtoken::Algorithm::RS384 => JwtAlgorithm::RS384,
            jsonwebtoken::Algorithm::RS512 => JwtAlgorithm::RS512,
            jsonwebtoken::Algorithm::ES256 => JwtAlgorithm::ES256,
            jsonwebtoken::Algorithm::ES384 => JwtAlgorithm::ES384,
            _ => return Err(AuthError::UnsupportedAlgorithm(format!("{:?}", algorithm))),
        };

        let token_service = TokenService::with_algorithm(secret, jwt_algorithm.to_algorithm())?;
        Ok(Self { token_service })
    }

    /// Configure issuer (backward compatibility)
    pub fn with_issuer(self, issuer: impl Into<String>) -> Self {
        Self {
            token_service: self.token_service.with_issuer(issuer),
        }
    }

    /// Configure audience (backward compatibility)
    pub fn with_audience(self, audience: impl Into<String>) -> Self {
        Self {
            token_service: self.token_service.with_audience(audience),
        }
    }

    /// Configure default expiry time in seconds (backward compatibility)
    pub fn with_default_expiry(self, seconds: u64) -> Self {
        Self {
            token_service: self.token_service.with_default_expiry(seconds),
        }
    }

    /// Create a JWT token from claims (backward compatibility)
    pub fn create_token(&self, claims: &Claims) -> AuthResult<String> {
        self.token_service.create_token(claims)
    }

    /// Validate and decode a JWT token (backward compatibility)
    pub fn validate_token(&self, token: &str) -> AuthResult<Claims> {
        self.token_service.validate_token(token)
    }

    /// Create access token with default expiry (backward compatibility)
    pub fn create_access_token(&self, subject: &str, tenant_id: &str, roles: Vec<String>) -> AuthResult<String> {
        self.token_service.create_access_token(subject, tenant_id, roles)
    }

    /// Create refresh token with longer expiry (backward compatibility)
    pub fn create_refresh_token(&self, subject: &str, tenant_id: &str) -> AuthResult<String> {
        self.token_service.create_refresh_token(subject, tenant_id)
    }

    /// Extract claims without full validation (backward compatibility)
    pub fn extract_claims_unchecked(&self, token: &str) -> AuthResult<Claims> {
        self.token_service.extract_claims_unchecked(token)
    }

    /// Check if token is expired (backward compatibility)
    pub fn is_token_expired(&self, claims: &Claims) -> bool {
        self.token_service.is_token_expired(claims)
    }

    /// Get remaining seconds until expiry (backward compatibility)
    pub fn get_seconds_until_expiry(&self, claims: &Claims) -> i64 {
        self.token_service.get_seconds_until_expiry(claims)
    }

    /// Validate token type (backward compatibility)
    pub fn validate_token_type(&self, claims: &Claims, expected_type: &str) -> AuthResult<()> {
        self.token_service.validate_token_type(claims, expected_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_jwt_creation() {
        let service = JwtService::new(b"test-secret-key").unwrap();
        let claims = Claims::new("user-123", "tenant-456");

        let token = service.create_token(&claims).unwrap();
        assert!(!token.is_empty());
        assert!(token.starts_with("eyJ")); // JWT tokens start with "eyJ"
    }

    #[test]
    fn test_jwt_validation() {
        let service = JwtService::new(b"test-secret-key").unwrap();
        let claims = Claims::new("user-123", "tenant-456");

        let token = service.create_token(&claims).unwrap();
        let validated = service.validate_token(&token).unwrap();

        assert_eq!(validated.sub, "user-123");
        assert_eq!(validated.tenant_id, "tenant-456");
    }

    #[test]
    fn test_invalid_token() {
        let service = JwtService::new(b"test-secret-key").unwrap();
        let result = service.validate_token("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token() {
        let service = JwtService::new(b"test-secret-key").unwrap();
        let mut claims = Claims::new("user-123", "tenant-456");

        // Set expiration to past
        claims.exp = (chrono::Utc::now() - Duration::hours(1)).timestamp() as u64;

        let token = service.create_token(&claims).unwrap();
        let result = service.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_with_roles_and_permissions() {
        let service = JwtService::new(b"test-secret-key").unwrap();
        let mut claims = Claims::new("user-123", "tenant-456");
        claims.roles = vec!["admin".to_string(), "user".to_string()];
        claims.permissions = vec!["read".to_string(), "write".to_string()];

        let token = service.create_token(&claims).unwrap();
        let validated = service.validate_token(&token).unwrap();

        assert_eq!(validated.sub, "user-123");
        assert_eq!(validated.tenant_id, "tenant-456");
        assert_eq!(validated.roles, vec!["admin".to_string(), "user".to_string()]);
        assert_eq!(validated.permissions, vec!["read".to_string(), "write".to_string()]);
    }

    #[test]
    fn test_jwt_algorithm_mapping() {
        assert_eq!(JwtAlgorithm::HS256.to_algorithm(), jsonwebtoken::Algorithm::HS256);
        assert_eq!(JwtAlgorithm::HS384.to_algorithm(), jsonwebtoken::Algorithm::HS384);
        assert_eq!(JwtAlgorithm::HS512.to_algorithm(), jsonwebtoken::Algorithm::HS512);
        assert_eq!(JwtAlgorithm::RS256.to_algorithm(), jsonwebtoken::Algorithm::RS256);
        assert_eq!(JwtAlgorithm::ES256.to_algorithm(), jsonwebtoken::Algorithm::ES256);
    }
}
