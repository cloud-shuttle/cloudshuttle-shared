//! JWT token creation, validation, and refresh operations

use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::Claims;
use crate::types::{AuthResult, AuthError};
#[cfg(feature = "observability")]
use cloudshuttle_observability::audit::{audit_auth, AuditResult};

/// JWT token service
pub struct TokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    issuer: String,
    audience: String,
    default_expiry: u64,
}

impl TokenService {
    /// Create new token service with HMAC secret
    pub fn new(secret: &[u8]) -> AuthResult<Self> {
        Self::with_algorithm(secret, Algorithm::HS256)
    }

    /// Create a new token service with specific algorithm
    pub fn with_algorithm(secret: &[u8], algorithm: Algorithm) -> AuthResult<Self> {
        let (encoding_key, decoding_key) = Self::create_keys(secret, algorithm)?;

        let mut validation = Validation::new(algorithm);
        validation.validate_exp = true;
        validation.validate_nbf = false; // Not using nbf claims
        validation.leeway = 30; // 30 seconds leeway for clock skew
        validation.set_audience(&["cloudshuttle-api"]);

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
            .map_err(|e| {
                #[cfg(feature = "observability")]
                audit_auth("token_validation_failed", None, AuditResult::Failure);
                AuthError::TokenValidation(e.to_string())
            })?;

        #[cfg(feature = "observability")]
        audit_auth("token_validated", Some(&token_data.claims.sub), AuditResult::Success);

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

        let token = self.create_token(&claims)?;

        #[cfg(feature = "observability")]
        audit_auth("access_token_created", Some(subject), AuditResult::Success);

        Ok(token)
    }

    /// Create refresh token with longer expiry
    pub fn create_refresh_token(&self, subject: &str, tenant_id: &str) -> AuthResult<String> {
        let mut claims = Claims::new(subject, tenant_id);
        claims.exp = self.get_expiry_timestamp(self.default_expiry * 24 * 7); // 7 days
        claims.iat = self.get_current_timestamp();
        claims.iss = Some(self.issuer.clone());
        claims.aud = Some(self.audience.clone());
        claims.token_type = Some("refresh".to_string());

        let token = self.create_token(&claims)?;

        #[cfg(feature = "observability")]
        audit_auth("refresh_token_created", Some(subject), AuditResult::Success);

        Ok(token)
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

    fn create_keys(secret: &[u8], algorithm: Algorithm) -> AuthResult<(EncodingKey, DecodingKey)> {
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

        Ok((encoding_key, decoding_key))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_service_creation() {
        let service = TokenService::new(b"test-secret-key").unwrap();
        assert_eq!(service.issuer, "cloudshuttle");
        assert_eq!(service.audience, "cloudshuttle-api");
        assert_eq!(service.default_expiry, 3600);
    }

    #[test]
    fn test_token_creation_and_validation() {
        let service = TokenService::new(b"test-secret-key").unwrap();
        let claims = Claims::new("user-123", "tenant-456");

        let token = service.create_token(&claims).unwrap();
        assert!(!token.is_empty());
        assert!(token.starts_with("eyJ")); // JWT tokens start with "eyJ"

        let validated = service.validate_token(&token).unwrap();
        assert_eq!(validated.sub, "user-123");
        assert_eq!(validated.tenant_id, "tenant-456");
    }

    #[test]
    fn test_invalid_token() {
        let service = TokenService::new(b"test-secret-key").unwrap();
        let result = service.validate_token("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_access_token_creation() {
        let service = TokenService::new(b"test-secret-key").unwrap();
        let roles = vec!["admin".to_string(), "user".to_string()];

        let token = service.create_access_token("user-123", "tenant-456", roles.clone()).unwrap();
        let validated = service.validate_token(&token).unwrap();

        assert_eq!(validated.sub, "user-123");
        assert_eq!(validated.tenant_id, "tenant-456");
        assert_eq!(validated.roles, roles);
        assert!(validated.exp > validated.iat);
    }

    #[test]
    fn test_refresh_token_creation() {
        let service = TokenService::new(b"test-secret-key").unwrap();

        let token = service.create_refresh_token("user-123", "tenant-456").unwrap();
        let validated = service.validate_token(&token).unwrap();

        assert_eq!(validated.sub, "user-123");
        assert_eq!(validated.tenant_id, "tenant-456");
        assert_eq!(validated.token_type, Some("refresh".to_string()));
        assert!(validated.exp > validated.iat);
    }

    #[test]
    fn test_token_type_validation() {
        let service = TokenService::new(b"test-secret-key").unwrap();
        let mut claims = Claims::new("user-123", "tenant-456");
        claims.token_type = Some("refresh".to_string());

        assert!(service.validate_token_type(&claims, "refresh").is_ok());
        assert!(service.validate_token_type(&claims, "access").is_err());
    }
}
