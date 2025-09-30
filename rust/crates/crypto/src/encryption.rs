//! AES encryption utilities
//!
//! This module provides secure AES-256-GCM encryption and decryption
//! for sensitive data with automatic nonce generation.

use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{rngs::OsRng, RngCore};
use std::result::Result as StdResult;

/// Error type for encryption operations
#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Invalid data format: {0}")]
    InvalidDataFormat(String),
}

/// Result type for encryption operations
pub type Result<T> = StdResult<T, EncryptionError>;

/// Encrypt data using AES-256-GCM
///
/// This function encrypts the provided data using AES-256-GCM with
/// a randomly generated nonce. The result is base64-encoded for
/// easy storage and transmission.
///
/// # Arguments
/// * `key` - 32-byte AES-256 key
/// * `data` - Data to encrypt
///
/// # Returns
/// A Result containing the base64-encoded encrypted data or an error
///
/// # Example
/// ```rust
/// let key = [0u8; 32]; // 32-byte key
/// let encrypted = encrypt_data(&key, b"secret data")?;
/// assert!(!encrypted.is_empty());
/// ```
pub fn encrypt_data(key: &[u8; 32], data: &[u8]) -> Result<String> {
    if data.is_empty() {
        return Err(EncryptionError::InvalidDataFormat("Data cannot be empty".to_string()));
    }

    // Create cipher from key
    let cipher = Aes256Gcm::new(key.into());

    // Generate random nonce (96 bits for GCM)
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);

    // Encrypt the data
    let ciphertext = cipher
        .encrypt(&nonce.into(), data)
        .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

    // Prepend nonce to ciphertext for storage
    let mut encrypted = Vec::with_capacity(nonce.len() + ciphertext.len());
    encrypted.extend_from_slice(&nonce);
    encrypted.extend_from_slice(&ciphertext);

    // Base64 encode for easy storage
    Ok(URL_SAFE_NO_PAD.encode(&encrypted))
}

/// Decrypt data using AES-256-GCM
///
/// This function decrypts base64-encoded data that was encrypted
/// with encrypt_data using the same key.
///
/// # Arguments
/// * `key` - 32-byte AES-256 key (same as used for encryption)
/// * `encrypted_data` - Base64-encoded encrypted data
///
/// # Returns
/// A Result containing the decrypted data or an error
///
/// # Example
/// ```rust
/// let key = [0u8; 32]; // 32-byte key
/// let encrypted = encrypt_data(&key, b"secret data")?;
/// let decrypted = decrypt_data(&key, &encrypted)?;
/// assert_eq!(decrypted, b"secret data");
/// ```
pub fn decrypt_data(key: &[u8; 32], encrypted_data: &str) -> Result<Vec<u8>> {
    // Base64 decode the data
    let data = URL_SAFE_NO_PAD
        .decode(encrypted_data)
        .map_err(|e| EncryptionError::InvalidDataFormat(e.to_string()))?;

    // Data must contain at least the nonce (12 bytes)
    if data.len() < 12 {
        return Err(EncryptionError::InvalidDataFormat("Data too short".to_string()));
    }

    // Split nonce and ciphertext
    let (nonce, ciphertext) = data.split_at(12);

    // Create cipher from key
    let cipher = Aes256Gcm::new(key.into());

    // Decrypt the data
    let plaintext = cipher
        .decrypt(nonce.into(), ciphertext)
        .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32]; // Test key
        let data = b"Hello, World! This is a test message.";

        // Encrypt
        let encrypted = encrypt_data(&key, data).unwrap();
        assert!(!encrypted.is_empty());

        // Decrypt
        let decrypted = decrypt_data(&key, &encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_encrypt_empty_data() {
        let key = [42u8; 32];
        let result = encrypt_data(&key, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let key = [42u8; 32];
        let result = decrypt_data(&key, "invalid-base64!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key1 = [42u8; 32];
        let key2 = [43u8; 32]; // Different key
        let data = b"secret message";

        let encrypted = encrypt_data(&key1, data).unwrap();
        let result = decrypt_data(&key2, &encrypted);
        assert!(result.is_err()); // Should fail with wrong key
    }

    #[test]
    fn test_decrypt_tampered_data() {
        let key = [42u8; 32];
        let data = b"secret message";

        let mut encrypted = encrypt_data(&key, data).unwrap();
        // Tamper with the encrypted data
        if let Some(last_char) = encrypted.chars().last() {
            let replacement = if last_char == 'A' { 'B' } else { 'A' };
            encrypted = encrypted[..encrypted.len() - 1].to_string() + &replacement.to_string();
        }

        let result = decrypt_data(&key, &encrypted);
        assert!(result.is_err()); // Should fail with tampered data
    }
}
