//! JWT token creation and validation

use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::{Claims, AuthResult, AuthError};

/// JWT service for token operations
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    issuer: String,
    audience: String,
    default_expiry: u64,
}

impl JwtService {
    /// Create a new JWT service with HMAC secret
    pub fn new(secret: &[u8]) -> AuthResult<Self> {
        Self::with_algorithm(secret, Algorithm::HS256)
    }

    /// Create a new JWT service with specific algorithm
    pub fn with_algorithm(secret: &[u8], algorithm: Algorithm) -> AuthResult<Self> {
        let encoding_key = match algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                EncodingKey::from_secret(secret)
            }
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                EncodingKey::from_rsa_pem(secret).map_err(|e| AuthError::InvalidKey(e.to_string()))?
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                EncodingKey::from_ec_pem(secret).map_err(|e| AuthError::InvalidKey(e.to_string()))?
            }
            _ => return Err(AuthError::UnsupportedAlgorithm(format!("{:?}", algorithm))),
        };

        let decoding_key = match algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                DecodingKey::from_secret(secret)
            }
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                DecodingKey::from_rsa_pem(secret).map_err(|e| AuthError::InvalidKey(e.to_string()))?
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                DecodingKey::from_ec_pem(secret).map_err(|e| AuthError::InvalidKey(e.to_string()))?
            }
            _ => return Err(AuthError::UnsupportedAlgorithm(format!("{:?}", algorithm))),
        };

        let mut validation = Validation::new(algorithm);
        validation.validate_exp = true;
        validation.validate_nbf = false; // Not using nbf claims
        validation.leeway = 30; // 30 seconds leeway for clock skew

        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
            issuer: "cloudshuttle".to_string(),
            audience: "cloudshuttle-api".to_string(),
            default_expiry: 3600, // 1 hour
        })
    }

    /// Configure issuer
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = issuer.into();
        self
    }

    /// Configure audience
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = audience.into();
        self
    }

    /// Configure default expiry time in seconds
    pub fn with_default_expiry(mut self, seconds: u64) -> Self {
        self.default_expiry = seconds;
        self
    }

    /// Create a JWT token from claims
    pub fn create_token(&self, claims: &Claims) -> AuthResult<String> {
        let header = Header::new(self.validation.algorithms[0]);

        encode(&header, claims, &self.encoding_key)
            .map_err(|e| AuthError::TokenCreation(e.to_string()))
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> AuthResult<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| AuthError::TokenValidation(e.to_string()))?;

        Ok(token_data.claims)
    }

    /// Create access token with default expiry
    pub fn create_access_token(&self, subject: &str, tenant_id: &str, roles: Vec<String>) -> AuthResult<String> {
        let mut claims = Claims::new(subject, tenant_id);
        claims.roles = roles;
        claims.exp = self.get_expiry_timestamp(self.default_expiry);
        claims.iat = self.get_current_timestamp();
        claims.iss = Some(self.issuer.clone());
        claims.aud = Some(self.audience.clone());

        self.create_token(&claims)
    }

    /// Create refresh token with longer expiry
    pub fn create_refresh_token(&self, subject: &str, tenant_id: &str) -> AuthResult<String> {
        let mut claims = Claims::new(subject, tenant_id);
        claims.exp = self.get_expiry_timestamp(self.default_expiry * 24 * 7); // 7 days
        claims.iat = self.get_current_timestamp();
        claims.iss = Some(self.issuer.clone());
        claims.aud = Some(self.audience.clone());
        claims.token_type = Some("refresh".to_string());

        self.create_token(&claims)
    }

    /// Extract claims without full validation (for refresh scenarios)
    pub fn extract_claims_unchecked(&self, token: &str) -> AuthResult<Claims> {
        let mut validation = self.validation.clone();
        validation.validate_exp = false; // Skip expiry validation

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AuthError::TokenValidation(e.to_string()))?;

        Ok(token_data.claims)
    }

    /// Check if token is expired
    pub fn is_token_expired(&self, claims: &Claims) -> bool {
        let now = self.get_current_timestamp();
        claims.exp < now
    }

    /// Get remaining seconds until expiry
    pub fn get_seconds_until_expiry(&self, claims: &Claims) -> i64 {
        let now = self.get_current_timestamp() as i64;
        claims.exp as i64 - now
    }

    /// Validate token type
    pub fn validate_token_type(&self, claims: &Claims, expected_type: &str) -> AuthResult<()> {
        if let Some(token_type) = &claims.token_type {
            if token_type != expected_type {
                return Err(AuthError::InvalidTokenType {
                    expected: expected_type.to_string(),
                    actual: token_type.clone(),
                });
            }
        }
        Ok(())
    }

    fn get_current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn get_expiry_timestamp(&self, seconds_from_now: u64) -> u64 {
        self.get_current_timestamp() + seconds_from_now
    }
}

/// Token validation options for different scenarios
#[derive(Debug, Clone)]
pub enum TokenValidation {
    Strict,
    RefreshToken,
    ApiKey,
}

impl TokenValidation {
    pub fn to_validation(&self, base_validation: &Validation) -> Validation {
        let mut validation = base_validation.clone();

        match self {
            Self::Strict => {
                // All validations enabled
            }
            Self::RefreshToken => {
                validation.validate_exp = false; // Allow expired tokens for refresh
            }
            Self::ApiKey => {
                validation.validate_exp = true;
                // Note: validate_iat not available in current jsonwebtoken version
                validation.validate_nbf = false;
            }
        }

        validation
    }
}

/// JWT algorithm variants
#[derive(Debug, Clone)]
pub enum JwtAlgorithm {
    HS256,
    HS384,
    HS512,
    RS256,
    RS384,
    RS512,
    ES256,
    ES384,
}

impl JwtAlgorithm {
    pub fn to_algorithm(&self) -> Algorithm {
        match self {
            Self::HS256 => Algorithm::HS256,
            Self::HS384 => Algorithm::HS384,
            Self::HS512 => Algorithm::HS512,
            Self::RS256 => Algorithm::RS256,
            Self::RS384 => Algorithm::RS384,
            Self::RS512 => Algorithm::RS512,
            Self::ES256 => Algorithm::ES256,
            Self::ES384 => Algorithm::ES384,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_jwt_creation() {
        let service = JwtService::new(b"test-secret-key");
        let claims = Claims::new("user-123", "tenant-456");

        let token = service.create_token(&claims).unwrap();
        assert!(!token.is_empty());
        assert!(token.starts_with("eyJ")); // JWT tokens start with "eyJ"
    }

    #[test]
    fn test_jwt_validation() {
        let service = JwtService::new(b"test-secret-key");
        let claims = Claims::new("user-123", "tenant-456");

        let token = service.create_token(&claims).unwrap();
        let validated = service.validate_token(&token).unwrap();

        assert_eq!(validated.sub, "user-123");
        assert_eq!(validated.tenant_id, "tenant-456");
    }

    #[test]
    fn test_invalid_token() {
        let service = JwtService::new(b"test-secret-key");
        let result = service.validate_token("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token() {
        let service = JwtService::new(b"test-secret-key");
        let mut claims = Claims::new("user-123", "tenant-456");

        // Set expiration to past
        claims.exp = (chrono::Utc::now() - Duration::hours(1)).timestamp() as usize;

        let token = service.create_token(&claims).unwrap();
        let result = service.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_with_roles_and_permissions() {
        let service = JwtService::new(b"test-secret-key");
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
    fn test_token_with_custom_expiry() {
        let service = JwtService::new(b"test-secret-key");
        let mut claims = Claims::new("user-123", "tenant-456");

        // Set custom expiry (1 hour from now)
        let future_time = chrono::Utc::now() + Duration::hours(1);
        claims.exp = future_time.timestamp() as usize;

        let token = service.create_token(&claims).unwrap();
        let validated = service.validate_token(&token).unwrap();

        assert_eq!(validated.sub, "user-123");
        // Allow some tolerance for timestamp comparison
        assert!((validated.exp as i64 - future_time.timestamp()).abs() <= 1);
    }

    #[test]
    fn test_token_tampering_detection() {
        let service1 = JwtService::new(b"secret-key-1");
        let service2 = JwtService::new(b"secret-key-2");

        let claims = Claims::new("user-123", "tenant-456");
        let token = service1.create_token(&claims).unwrap();

        // Try to validate with different secret
        let result = service2.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_claims() {
        let service = JwtService::new(b"test-secret-key");
        let claims = Claims::new("", "");

        let token = service.create_token(&claims).unwrap();
        let validated = service.validate_token(&token).unwrap();

        assert_eq!(validated.sub, "");
        assert_eq!(validated.tenant_id, "");
    }

    #[test]
    fn test_very_long_claims() {
        let service = JwtService::new(b"test-secret-key");
        let long_string = "a".repeat(1000);
        let claims = Claims::new(&long_string, &long_string);

        let token = service.create_token(&claims).unwrap();
        let validated = service.validate_token(&token).unwrap();

        assert_eq!(validated.sub, long_string);
        assert_eq!(validated.tenant_id, long_string);
    }

    #[test]
    fn test_jwt_service_with_different_key_lengths() {
        // Test with different key lengths
        let keys = vec![
            b"short",
            b"medium-length-key-for-testing",
            b"very-long-secret-key-that-should-work-fine-with-jwt-encryption-and-decryption-algorithms",
        ];

        for key in keys {
            let service = JwtService::new(key);
            let claims = Claims::new("test-user", "test-tenant");

            let token = service.create_token(&claims).unwrap();
            let validated = service.validate_token(&token).unwrap();

            assert_eq!(validated.sub, "test-user");
            assert_eq!(validated.tenant_id, "test-tenant");
        }
    }

    #[test]
    fn test_jwt_expiry_precision() {
        let service = JwtService::new(b"test-secret-key");
        let claims = Claims::new("user-123", "tenant-456");

        let token = service.create_token(&claims).unwrap();
        let validated1 = service.validate_token(&token).unwrap();
        let validated2 = service.validate_token(&token).unwrap();

        // Expiry should be consistent across validations
        assert_eq!(validated1.exp, validated2.exp);
    }

    #[test]
    fn test_jwt_iat_claim() {
        let service = JwtService::new(b"test-secret-key");
        let claims = Claims::new("user-123", "tenant-456");

        let before_creation = chrono::Utc::now().timestamp() as usize;
        let token = service.create_token(&claims).unwrap();
        let after_creation = chrono::Utc::now().timestamp() as usize;

        let validated = service.validate_token(&token).unwrap();

        // iat should be set and reasonable
        assert!(validated.iat >= before_creation);
        assert!(validated.iat <= after_creation);
    }

    #[test]
    fn test_jwt_algorithm_mapping() {
        assert_eq!(JwtAlgorithm::HS256.to_algorithm(), Algorithm::HS256);
        assert_eq!(JwtAlgorithm::HS384.to_algorithm(), Algorithm::HS384);
        assert_eq!(JwtAlgorithm::HS512.to_algorithm(), Algorithm::HS512);
        assert_eq!(JwtAlgorithm::RS256.to_algorithm(), Algorithm::RS256);
        assert_eq!(JwtAlgorithm::ES256.to_algorithm(), Algorithm::ES256);
    }
}
