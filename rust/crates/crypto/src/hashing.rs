//! Password hashing utilities using Argon2
//!
//! This module provides secure password hashing and verification
//! using the Argon2 algorithm with appropriate security parameters.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as ArgonPasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::result::Result as StdResult;

/// Error type for password operations
#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Password hashing failed: {0}")]
    HashingFailed(String),

    #[error("Password verification failed: {0}")]
    VerificationFailed(String),

    #[error("Invalid password hash format: {0}")]
    InvalidHashFormat(String),
}

/// Result type for password operations
pub type Result<T> = StdResult<T, PasswordError>;

/// Hash a password using Argon2
///
/// This function generates a secure hash of the provided password
/// using Argon2 with a randomly generated salt.
///
/// # Arguments
/// * `password` - The password to hash
///
/// # Returns
/// A Result containing the password hash string or an error
///
/// # Example
/// ```rust
/// let hash = hash_password("my-password")?;
/// assert!(!hash.is_empty());
/// ```
pub fn hash_password(password: &str) -> Result<String> {
    // Validate password is not empty
    if password.is_empty() {
        return Err(PasswordError::HashingFailed("Password cannot be empty".to_string()));
    }

    // Generate a random salt
    let salt = SaltString::generate(&mut OsRng);

    // Create Argon2 hasher with default parameters
    let argon2 = Argon2::default();

    // Hash the password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordError::HashingFailed(e.to_string()))?;

    Ok(password_hash.to_string())
}

/// Verify a password against its hash
///
/// This function verifies that a provided password matches
/// the given hash using Argon2.
///
/// # Arguments
/// * `password` - The password to verify
/// * `hash` - The password hash to verify against
///
/// # Returns
/// A Result containing true if the password matches, false otherwise
///
/// # Example
/// ```rust
/// let hash = hash_password("my-password")?;
/// assert!(verify_password("my-password", &hash)?);
/// assert!(!verify_password("wrong-password", &hash)?);
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    // Parse the hash
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| PasswordError::InvalidHashFormat(e.to_string()))?;

    // Create Argon2 verifier with default parameters
    let argon2 = Argon2::default();

    // Verify the password
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(PasswordError::VerificationFailed(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test-password-123";
        let hash = hash_password(password).unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "test-password-123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "test-password-123";
        let hash = hash_password(password).unwrap();
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_empty_password() {
        let result = hash_password("");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_invalid_hash() {
        let result = verify_password("password", "invalid-hash");
        assert!(result.is_err());
    }
}
