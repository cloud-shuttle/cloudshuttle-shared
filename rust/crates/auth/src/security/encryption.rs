//! Encryption utilities and secure token generation

use crate::types::{AuthResult, AuthError};
use ring::rand::SecureRandom;
use ring::rand::SystemRandom;
use base64::Engine;

/// Cryptographic utilities
pub struct CryptoUtils;

impl CryptoUtils {
    /// Hash password using Argon2
    pub fn hash_password(password: &str) -> AuthResult<String> {
        use argon2::{Argon2, PasswordHasher, PasswordVerifier};
        use argon2::password_hash::{rand_core::OsRng, PasswordHash, SaltString};

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::ExternalServiceError("Password hashing failed".to_string()))?;

        Ok(password_hash.to_string())
    }

    /// Verify password against hash
    pub fn verify_password(password: &str, hash: &str) -> AuthResult<bool> {
        use argon2::{Argon2, PasswordVerifier};
        use argon2::password_hash::PasswordHash;

        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    /// Generate secure random token
    pub fn generate_secure_token(length: usize) -> AuthResult<String> {
        let rng = SystemRandom::new();
        let mut bytes = vec![0u8; length];
        rng.fill(&mut bytes).unwrap();

        Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
    }

    /// Generate cryptographically secure random bytes
    pub fn generate_random_bytes(length: usize) -> AuthResult<Vec<u8>> {
        let rng = SystemRandom::new();
        let mut bytes = vec![0u8; length];
        rng.fill(&mut bytes).map_err(|_| AuthError::ServiceUnavailable)?;
        Ok(bytes)
    }

    /// Hash data using SHA-256
    pub fn sha256_hash(data: &[u8]) -> String {
        use ring::digest;
        let digest = digest::digest(&digest::SHA256, data);
        hex::encode(digest.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "MyTestPassword123!";
        let hash = CryptoUtils::hash_password(password).unwrap();

        // Hash should be different from password
        assert_ne!(hash, password);

        // Should be able to verify
        assert!(CryptoUtils::verify_password(password, &hash).unwrap());
        assert!(!CryptoUtils::verify_password("wrongpassword", &hash).unwrap());
    }

    #[test]
    fn test_secure_token_generation() {
        let token1 = CryptoUtils::generate_secure_token(32).unwrap();
        let token2 = CryptoUtils::generate_secure_token(32).unwrap();

        // Tokens should be different
        assert_ne!(token1, token2);

        // Should be URL-safe base64
        assert!(token1.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"Hello, World!";
        let hash1 = CryptoUtils::sha256_hash(data);
        let hash2 = CryptoUtils::sha256_hash(data);

        // Same input should produce same hash
        assert_eq!(hash1, hash2);

        // Hash should be 64 characters (32 bytes * 2 hex chars per byte)
        assert_eq!(hash1.len(), 64);

        // Should be valid hex
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
