//! JWT token creation and validation

use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
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
            _ => return Err(AuthError::UnsupportedAlgorithm(algorithm.to_string())),
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
            _ => return Err(AuthError::UnsupportedAlgorithm(algorithm.to_string())),
        };

        let mut validation = Validation::new(algorithm);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.validate_iat = true;
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
        claims.expiry = self.get_expiry_timestamp(self.default_expiry);
        claims.issued_at = self.get_current_timestamp();
        claims.issuer = Some(self.issuer.clone());
        claims.audience = Some(self.audience.clone());

        self.create_token(&claims)
    }

    /// Create refresh token with longer expiry
    pub fn create_refresh_token(&self, subject: &str, tenant_id: &str) -> AuthResult<String> {
        let mut claims = Claims::new(subject, tenant_id);
        claims.expiry = self.get_expiry_timestamp(self.default_expiry * 24 * 7); // 7 days
        claims.issued_at = self.get_current_timestamp();
        claims.issuer = Some(self.issuer.clone());
        claims.audience = Some(self.audience.clone());
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
        claims.expiry < now
    }

    /// Get remaining seconds until expiry
    pub fn get_seconds_until_expiry(&self, claims: &Claims) -> i64 {
        let now = self.get_current_timestamp() as i64;
        claims.expiry as i64 - now
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
                validation.validate_iat = false;
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
