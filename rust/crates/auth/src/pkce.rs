//! PKCE (Proof Key for Code Exchange) implementation for OAuth 2.1
//!
//! This module provides RFC 7636 compliant PKCE implementation,
//! enhancing OAuth security by preventing authorization code interception attacks.

use base64::{Engine as _, engine::general_purpose};
use ring::digest;
use serde::{Deserialize, Serialize};
use crate::types::{AuthResult, AuthError};

/// PKCE code challenge method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PkceMethod {
    /// S256 - SHA256 hash of the code verifier (recommended)
    #[serde(rename = "S256")]
    S256,

    /// Plain - Code verifier sent as-is (not recommended for production)
    #[serde(rename = "plain")]
    Plain,
}

impl PkceMethod {
    /// Get the method as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::S256 => "S256",
            Self::Plain => "plain",
        }
    }
}

/// PKCE code verifier and challenge pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkcePair {
    /// The code verifier (43-128 characters)
    pub code_verifier: String,

    /// The code challenge (derived from verifier)
    pub code_challenge: String,

    /// The challenge method used
    pub code_challenge_method: PkceMethod,
}

/// PKCE handler for generating and validating challenges
pub struct PkceHandler;

impl PkceHandler {
    /// Generate a new PKCE code verifier and challenge pair
    ///
    /// Uses S256 method by default for security
    pub fn generate() -> AuthResult<PkcePair> {
        let code_verifier = Self::generate_code_verifier()?;
        let code_challenge = Self::generate_code_challenge(&code_verifier, PkceMethod::S256)?;

        Ok(PkcePair {
            code_verifier,
            code_challenge,
            code_challenge_method: PkceMethod::S256,
        })
    }

    /// Generate a PKCE pair with a specific method
    pub fn generate_with_method(method: PkceMethod) -> AuthResult<PkcePair> {
        let code_verifier = Self::generate_code_verifier()?;
        let code_challenge = Self::generate_code_challenge(&code_verifier, method)?;

        Ok(PkcePair {
            code_verifier,
            code_challenge,
            code_challenge_method: method,
        })
    }

    /// Validate a code verifier against a code challenge
    pub fn validate_challenge(code_verifier: &str, code_challenge: &str, method: PkceMethod) -> AuthResult<bool> {
        // Validate code verifier format
        Self::validate_code_verifier(code_verifier)?;

        // Generate expected challenge and compare
        let expected_challenge = Self::generate_code_challenge(code_verifier, method)?;
        Ok(expected_challenge == code_challenge)
    }

    /// Validate code verifier format (RFC 7636 requirements)
    pub fn validate_code_verifier(code_verifier: &str) -> AuthResult<()> {
        // Must be 43-128 characters
        if code_verifier.len() < 43 || code_verifier.len() > 128 {
            return Err(AuthError::InvalidRequest(
                "Code verifier must be between 43 and 128 characters".to_string()
            ));
        }

        // Must contain only unreserved characters (RFC 3986)
        // A-Z, a-z, 0-9, "-", ".", "_", "~"
        for ch in code_verifier.chars() {
            if !matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '.' | '_' | '~') {
                return Err(AuthError::InvalidRequest(
                    "Code verifier contains invalid characters".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Generate a cryptographically secure code verifier
    fn generate_code_verifier() -> AuthResult<String> {
        use ring::rand::{SecureRandom, SystemRandom};

        let rng = SystemRandom::new();
        let mut bytes = [0u8; 32]; // 32 bytes = 256 bits of entropy

        rng.fill(&mut bytes).map_err(|_| {
            AuthError::InternalError("Failed to generate secure random bytes".to_string())
        })?;

        // Base64url encode (RFC 7636 requires base64url)
        let encoded = general_purpose::URL_SAFE_NO_PAD.encode(&bytes);

        // Ensure it's exactly 43-128 characters (should be 43 for 32 bytes)
        Ok(encoded.chars().take(128).collect())
    }

    /// Generate code challenge from verifier using specified method
    fn generate_code_challenge(code_verifier: &str, method: PkceMethod) -> AuthResult<String> {
        match method {
            PkceMethod::S256 => {
                // SHA256 hash of the code verifier
                let digest = digest::digest(&digest::SHA256, code_verifier.as_bytes());
                let hash_bytes = digest.as_ref();

                // Base64url encode the hash
                let challenge = general_purpose::URL_SAFE_NO_PAD.encode(hash_bytes);
                Ok(challenge)
            }
            PkceMethod::Plain => {
                // Plain method just returns the verifier as-is
                // Note: This is NOT recommended for production use
                Ok(code_verifier.to_string())
            }
        }
    }
}

impl PkcePair {
    /// Get the code challenge for use in authorization requests
    pub fn challenge(&self) -> &str {
        &self.code_challenge
    }

    /// Get the code challenge method
    pub fn method(&self) -> PkceMethod {
        self.code_challenge_method
    }

    /// Validate this pair against a provided challenge
    pub fn validate_against(&self, provided_challenge: &str) -> AuthResult<bool> {
        PkceHandler::validate_challenge(
            &self.code_verifier,
            provided_challenge,
            self.code_challenge_method,
        )
    }

    /// Get the code verifier for use in token requests
    pub fn verifier(&self) -> &str {
        &self.code_verifier
    }
}

/// PKCE-protected OAuth authorization request
#[derive(Debug, Serialize, Deserialize)]
pub struct PkceAuthorizationRequest {
    /// OAuth client ID
    pub client_id: String,

    /// Authorization code redirect URI
    pub redirect_uri: String,

    /// Requested scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// State parameter for CSRF protection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// PKCE code challenge
    pub code_challenge: String,

    /// PKCE code challenge method
    pub code_challenge_method: PkceMethod,
}

/// PKCE token request with code verifier
#[derive(Debug, Deserialize)]
pub struct PkceTokenRequest {
    /// Authorization grant type (must be "authorization_code")
    pub grant_type: String,

    /// Authorization code received from authorization server
    pub code: String,

    /// Redirect URI used in authorization request
    pub redirect_uri: String,

    /// Client ID
    pub client_id: String,

    /// PKCE code verifier
    pub code_verifier: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pkce_pair() {
        let pair = PkceHandler::generate().unwrap();

        // Verify code verifier format
        assert!(pair.code_verifier.len() >= 43);
        assert!(pair.code_verifier.len() <= 128);
        PkceHandler::validate_code_verifier(&pair.code_verifier).unwrap();

        // Verify challenge is generated
        assert!(!pair.code_challenge.is_empty());
        assert_eq!(pair.code_challenge_method, PkceMethod::S256);
    }

    #[test]
    fn test_generate_with_method() {
        let pair = PkceHandler::generate_with_method(PkceMethod::S256).unwrap();
        assert_eq!(pair.code_challenge_method, PkceMethod::S256);

        // Note: Plain method not tested due to security concerns
    }

    #[test]
    fn test_validate_code_verifier_valid() {
        let valid_verifier = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
        assert!(PkceHandler::validate_code_verifier(valid_verifier).is_ok());
    }

    #[test]
    fn test_validate_code_verifier_too_short() {
        let short_verifier = "too_short";
        assert!(PkceHandler::validate_code_verifier(short_verifier).is_err());
    }

    #[test]
    fn test_validate_code_verifier_invalid_chars() {
        let invalid_verifier = "invalid@verifier!";
        assert!(PkceHandler::validate_code_verifier(invalid_verifier).is_err());
    }

    #[test]
    fn test_validate_challenge_s256() {
        let pair = PkceHandler::generate().unwrap();
        let is_valid = PkceHandler::validate_challenge(
            &pair.code_verifier,
            &pair.code_challenge,
            PkceMethod::S256,
        ).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_validate_challenge_invalid() {
        let pair = PkceHandler::generate().unwrap();
        let is_valid = PkceHandler::validate_challenge(
            &pair.code_verifier,
            "invalid_challenge",
            PkceMethod::S256,
        ).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_pkce_pair_validation() {
        let pair = PkceHandler::generate().unwrap();
        let is_valid = pair.validate_against(&pair.code_challenge).unwrap();
        assert!(is_valid);

        let is_invalid = pair.validate_against("wrong_challenge").unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_pkce_methods() {
        assert_eq!(PkceMethod::S256.as_str(), "S256");
        assert_eq!(PkceMethod::Plain.as_str(), "plain");
    }

    #[test]
    fn test_s256_challenge_generation() {
        let verifier = "test_verifier_123";
        let challenge = PkceHandler::generate_code_challenge(verifier, PkceMethod::S256).unwrap();

        // S256 challenge should be different from plain verifier
        assert_ne!(challenge, verifier);

        // Should be valid base64url
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
        assert!(!challenge.contains('='));
    }
}
