//! JWT token handling utilities

use crate::claims::{Claims, RefreshClaims, EmailVerificationClaims, PasswordResetClaims};
use cloudshuttle_error_handling::CloudShuttleError;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// JWT service for token creation and validation
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    access_token_duration: Duration,
    refresh_token_duration: Duration,
}

impl JwtService {
    /// Create a new JWT service with HS256 algorithm
    pub fn new(secret: &str) -> Result<Self, CloudShuttleError> {
        Self::with_algorithm(secret, Algorithm::HS256)
    }

    /// Create a new JWT service with custom algorithm
    pub fn with_algorithm(secret: &str, algorithm: Algorithm) -> Result<Self, CloudShuttleError> {
        let encoding_key = match algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                EncodingKey::from_secret(secret.as_bytes())
            }
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                EncodingKey::from_rsa_pem(secret.as_bytes())
                    .map_err(|e| CloudShuttleError::crypto(format!("Invalid RSA key: {}", e)))?
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                EncodingKey::from_ec_pem(secret.as_bytes())
                    .map_err(|e| CloudShuttleError::crypto(format!("Invalid EC key: {}", e)))?
            }
            _ => return Err(CloudShuttleError::crypto("Unsupported algorithm")),
        };

        let decoding_key = match algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                DecodingKey::from_secret(secret.as_bytes())
            }
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                DecodingKey::from_rsa_pem(secret.as_bytes())
                    .map_err(|e| CloudShuttleError::crypto(format!("Invalid RSA key: {}", e)))?
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                DecodingKey::from_ec_pem(secret.as_bytes())
                    .map_err(|e| CloudShuttleError::crypto(format!("Invalid EC key: {}", e)))?
            }
            _ => return Err(CloudShuttleError::crypto("Unsupported algorithm")),
        };

        let mut validation = Validation::new(algorithm);
        validation.validate_exp = true;
        validation.validate_nbf = false;
        validation.validate_aud = false;

        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
            access_token_duration: Duration::from_secs(3600), // 1 hour
            refresh_token_duration: Duration::from_secs(604800), // 7 days
        })
    }

    /// Set access token duration
    pub fn with_access_token_duration(mut self, duration: Duration) -> Self {
        self.access_token_duration = duration;
        self
    }

    /// Set refresh token duration
    pub fn with_refresh_token_duration(mut self, duration: Duration) -> Self {
        self.refresh_token_duration = duration;
        self
    }

    /// Create an access token
    pub fn create_access_token(&self, user_id: &str, tenant_id: uuid::Uuid, roles: Vec<String>) -> Result<String, CloudShuttleError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CloudShuttleError::internal(format!("Time error: {}", e)))?
            .as_secs() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + self.access_token_duration.as_secs() as usize,
            iat: now,
            tenant_id,
            roles,
            custom: std::collections::HashMap::new(),
        };

        self.create_token(&claims)
    }

    /// Create a refresh token
    pub fn create_refresh_token(&self, user_id: &str, tenant_id: uuid::Uuid, version: i32) -> Result<String, CloudShuttleError> {
        let claims = RefreshClaims::new(user_id.to_string(), tenant_id, version);
        self.create_token(&claims)
    }

    /// Create an email verification token
    pub fn create_email_verification_token(&self, user_id: &str, email: &str, tenant_id: uuid::Uuid, token_hash: &str) -> Result<String, CloudShuttleError> {
        let claims = EmailVerificationClaims::new(user_id.to_string(), email.to_string(), tenant_id, token_hash.to_string());
        self.create_token(&claims)
    }

    /// Create a password reset token
    pub fn create_password_reset_token(&self, user_id: &str, email: &str, tenant_id: uuid::Uuid, token_hash: &str) -> Result<String, CloudShuttleError> {
        let claims = PasswordResetClaims::new(user_id.to_string(), email.to_string(), tenant_id, token_hash.to_string());
        self.create_token(&claims)
    }

    /// Create a custom token
    pub fn create_token<T: Serialize>(&self, claims: &T) -> Result<String, CloudShuttleError> {
        let header = Header::new(self.validation.algorithms[0]);
        encode(&header, claims, &self.encoding_key)
            .map_err(|e| CloudShuttleError::auth(format!("Token creation failed: {}", e)))
    }

    /// Validate an access token
    pub fn validate_access_token(&self, token: &str) -> Result<Claims, CloudShuttleError> {
        self.validate_token(token)
    }

    /// Validate a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<RefreshClaims, CloudShuttleError> {
        self.validate_token(token)
    }

    /// Validate an email verification token
    pub fn validate_email_verification_token(&self, token: &str) -> Result<EmailVerificationClaims, CloudShuttleError> {
        self.validate_token(token)
    }

    /// Validate a password reset token
    pub fn validate_password_reset_token(&self, token: &str) -> Result<PasswordResetClaims, CloudShuttleError> {
        self.validate_token(token)
    }

    /// Validate a custom token
    pub fn validate_token<T: DeserializeOwned>(&self, token: &str) -> Result<T, CloudShuttleError> {
        let token_data = decode::<T>(token, &self.decoding_key, &self.validation)
            .map_err(|e| CloudShuttleError::auth(format!("Token validation failed: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Extract claims without validation (for debugging)
    pub fn decode_token<T: DeserializeOwned>(&self, token: &str) -> Result<T, CloudShuttleError> {
        let mut validation = self.validation.clone();
        validation.validate_exp = false;
        validation.validate_nbf = false;

        let token_data = decode::<T>(token, &self.decoding_key, &validation)
            .map_err(|e| CloudShuttleError::auth(format!("Token decode failed: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Refresh an access token using a refresh token
    pub fn refresh_access_token(&self, refresh_token: &str) -> Result<String, CloudShuttleError> {
        let refresh_claims: RefreshClaims = self.validate_refresh_token(refresh_token)?;
        self.create_access_token(&refresh_claims.sub, refresh_claims.tenant_id, Vec::new())
    }

    /// Check if a token is expired without full validation
    pub fn is_token_expired(&self, token: &str) -> Result<bool, CloudShuttleError> {
        // Try to decode without exp validation
        let mut validation = self.validation.clone();
        validation.validate_exp = false;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| CloudShuttleError::auth(format!("Token decode failed: {}", e)))?;

        Ok(token_data.claims.is_expired())
    }
}

/// Token type enumeration
#[derive(Debug, Clone)]
pub enum TokenType {
    Access,
    Refresh,
    EmailVerification,
    PasswordReset,
}

impl TokenType {
    /// Get the default duration for this token type
    pub fn default_duration(&self) -> Duration {
        match self {
            Self::Access => Duration::from_secs(3600), // 1 hour
            Self::Refresh => Duration::from_secs(604800), // 7 days
            Self::EmailVerification => Duration::from_secs(86400), // 24 hours
            Self::PasswordReset => Duration::from_secs(3600), // 1 hour
        }
    }
}
