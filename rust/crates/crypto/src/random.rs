//! Secure random generation utilities
//!
//! This module provides cryptographically secure random number
//! generation for tokens, nonces, and other security-sensitive values.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{rngs::OsRng, RngCore};
use std::result::Result as StdResult;

/// Error type for random generation operations
#[derive(Debug, thiserror::Error)]
pub enum RandomError {
    #[error("Random generation failed: {0}")]
    GenerationFailed(String),
}

/// Result type for random operations
pub type Result<T> = StdResult<T, RandomError>;

/// Generate a secure random token
///
/// This function generates a cryptographically secure random token
/// of the specified length in bytes, encoded as a URL-safe base64 string.
///
/// # Arguments
/// * `length_bytes` - Length of the random data in bytes (default: 32)
///
/// # Returns
/// A Result containing the base64-encoded random token or an error
///
/// # Example
/// ```rust
/// let token = generate_secure_token(32)?;
/// assert!(!token.is_empty());
/// ```
pub fn generate_secure_token(length_bytes: usize) -> Result<String> {
    if length_bytes == 0 {
        return Err(RandomError::GenerationFailed("Length must be greater than 0".to_string()));
    }

    // Generate random bytes
    let mut bytes = vec![0u8; length_bytes];
    OsRng.fill_bytes(&mut bytes);

    // Base64 encode for URL-safe token
    Ok(URL_SAFE_NO_PAD.encode(&bytes))
}

/// Generate a secure random token with default length (32 bytes)
///
/// This is a convenience function that generates a 32-byte (256-bit)
/// cryptographically secure random token.
///
/// # Returns
/// A Result containing the base64-encoded random token or an error
///
/// # Example
/// ```rust
/// let token = generate_secure_token_default()?;
/// assert!(!token.is_empty());
/// ```
pub fn generate_secure_token_default() -> Result<String> {
    generate_secure_token(32)
}

/// Generate random bytes
///
/// This function generates cryptographically secure random bytes
/// of the specified length.
///
/// # Arguments
/// * `length` - Number of random bytes to generate
///
/// # Returns
/// A Result containing the random bytes or an error
///
/// # Example
/// ```rust
/// let bytes = generate_random_bytes(16)?;
/// assert_eq!(bytes.len(), 16);
/// ```
pub fn generate_random_bytes(length: usize) -> Result<Vec<u8>> {
    if length == 0 {
        return Err(RandomError::GenerationFailed("Length must be greater than 0".to_string()));
    }

    let mut bytes = vec![0u8; length];
    OsRng.fill_bytes(&mut bytes);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_token() {
        let token = generate_secure_token(16).unwrap();
        assert!(!token.is_empty());

        // Should be URL-safe base64
        assert!(token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_generate_secure_token_default() {
        let token = generate_secure_token_default().unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_generate_different_tokens() {
        let token1 = generate_secure_token(16).unwrap();
        let token2 = generate_secure_token(16).unwrap();
        assert_ne!(token1, token2); // Should be different
    }

    #[test]
    fn test_generate_random_bytes() {
        let bytes = generate_random_bytes(16).unwrap();
        assert_eq!(bytes.len(), 16);

        // Check that not all bytes are the same (very unlikely for random data)
        let first_byte = bytes[0];
        assert!(bytes.iter().any(|&b| b != first_byte));
    }

    #[test]
    fn test_zero_length_token() {
        let result = generate_secure_token(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_length_bytes() {
        let result = generate_random_bytes(0);
        assert!(result.is_err());
    }
}
