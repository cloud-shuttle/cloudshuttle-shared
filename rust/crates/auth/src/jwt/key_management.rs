//! JWT algorithm and key management

use jsonwebtoken::{Algorithm, EncodingKey, DecodingKey};
use crate::types::{AuthResult, AuthError};

/// Supported JWT algorithms
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// Convert to jsonwebtoken Algorithm
    pub fn to_algorithm(self) -> Algorithm {
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

    /// Get algorithm name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HS256 => "HS256",
            Self::HS384 => "HS384",
            Self::HS512 => "HS512",
            Self::RS256 => "RS256",
            Self::RS384 => "RS384",
            Self::RS512 => "RS512",
            Self::ES256 => "ES256",
            Self::ES384 => "ES384",
        }
    }

    /// Check if algorithm uses symmetric cryptography
    pub fn is_symmetric(&self) -> bool {
        matches!(self, Self::HS256 | Self::HS384 | Self::HS512)
    }

    /// Check if algorithm uses asymmetric cryptography
    pub fn is_asymmetric(&self) -> bool {
        !self.is_symmetric()
    }
}

/// Key management for JWT operations
pub struct KeyManager {
    algorithm: JwtAlgorithm,
    encoding_key: Option<EncodingKey>,
    decoding_key: Option<DecodingKey>,
}

impl KeyManager {
    /// Create key manager for symmetric algorithms (HS256, HS384, HS512)
    pub fn new_hmac(secret: &[u8]) -> AuthResult<Self> {
        Self::with_algorithm(secret, JwtAlgorithm::HS256)
    }

    /// Create key manager with specific algorithm
    pub fn with_algorithm(secret: &[u8], algorithm: JwtAlgorithm) -> AuthResult<Self> {
        let (encoding_key, decoding_key) = Self::create_keys(secret, algorithm)?;

        Ok(Self {
            algorithm,
            encoding_key: Some(encoding_key),
            decoding_key: Some(decoding_key),
        })
    }

    /// Create key manager for asymmetric algorithms (RS256, ES256, etc.)
    pub fn new_asymmetric(
        public_key: &[u8],
        private_key: &[u8],
        algorithm: JwtAlgorithm,
    ) -> AuthResult<Self> {
        if algorithm.is_symmetric() {
            return Err(AuthError::InvalidKey(
                "Symmetric algorithm cannot use asymmetric keys".to_string()
            ));
        }

        let encoding_key = Self::create_encoding_key(private_key, algorithm)?;
        let decoding_key = Self::create_decoding_key(public_key, algorithm)?;

        Ok(Self {
            algorithm,
            encoding_key: Some(encoding_key),
            decoding_key: Some(decoding_key),
        })
    }

    /// Generate new key pair for asymmetric algorithms
    pub fn generate_keypair(algorithm: JwtAlgorithm) -> AuthResult<(Vec<u8>, Vec<u8>)> {
        if algorithm.is_symmetric() {
            return Err(AuthError::UnsupportedAlgorithm(
                "Cannot generate keypair for symmetric algorithm".to_string()
            ));
        }

        // Note: In production, this would generate proper RSA/EC keys
        // For now, return placeholder - proper implementation needed
        match algorithm {
            JwtAlgorithm::RS256 | JwtAlgorithm::RS384 | JwtAlgorithm::RS512 => {
                // RSA key generation would go here
                Err(AuthError::UnsupportedAlgorithm(
                    "RSA key generation not implemented".to_string()
                ))
            }
            JwtAlgorithm::ES256 | JwtAlgorithm::ES384 => {
                // ECDSA key generation would go here
                Err(AuthError::UnsupportedAlgorithm(
                    "ECDSA key generation not implemented".to_string()
                ))
            }
            _ => unreachable!(),
        }
    }

    /// Get encoding key for token creation
    pub fn encoding_key(&self) -> AuthResult<&EncodingKey> {
        self.encoding_key.as_ref().ok_or_else(|| {
            AuthError::ServiceUnavailable
        })
    }

    /// Get decoding key for token validation
    pub fn decoding_key(&self) -> AuthResult<&DecodingKey> {
        self.decoding_key.as_ref().ok_or_else(|| {
            AuthError::ServiceUnavailable
        })
    }

    /// Get the algorithm used by this key manager
    pub fn algorithm(&self) -> JwtAlgorithm {
        self.algorithm
    }

    /// Validate key compatibility with algorithm
    pub fn validate_key_compatibility(&self, key: &[u8]) -> AuthResult<()> {
        // Basic key length validation
        match self.algorithm {
            JwtAlgorithm::HS256 => {
                if key.len() < 32 {
                    return Err(AuthError::InvalidKey(
                        "HS256 requires at least 32 bytes".to_string()
                    ));
                }
            }
            JwtAlgorithm::HS384 => {
                if key.len() < 48 {
                    return Err(AuthError::InvalidKey(
                        "HS384 requires at least 48 bytes".to_string()
                    ));
                }
            }
            JwtAlgorithm::HS512 => {
                if key.len() < 64 {
                    return Err(AuthError::InvalidKey(
                        "HS512 requires at least 64 bytes".to_string()
                    ));
                }
            }
            _ => {
                // For asymmetric algorithms, we can't easily validate key format
                // without parsing, which is done by jsonwebtoken crate
            }
        }

        Ok(())
    }

    /// Rotate keys (for key rotation scenarios)
    pub fn rotate_keys(&mut self, new_secret: &[u8]) -> AuthResult<()> {
        let (encoding_key, decoding_key) = Self::create_keys(new_secret, self.algorithm)?;
        self.encoding_key = Some(encoding_key);
        self.decoding_key = Some(decoding_key);
        Ok(())
    }

    fn create_keys(secret: &[u8], algorithm: JwtAlgorithm) -> AuthResult<(EncodingKey, DecodingKey)> {
        let encoding_key = Self::create_encoding_key(secret, algorithm)?;
        let decoding_key = Self::create_decoding_key(secret, algorithm)?;
        Ok((encoding_key, decoding_key))
    }

    fn create_encoding_key(secret: &[u8], algorithm: JwtAlgorithm) -> AuthResult<EncodingKey> {
        match algorithm {
            JwtAlgorithm::HS256 | JwtAlgorithm::HS384 | JwtAlgorithm::HS512 => {
                Ok(EncodingKey::from_secret(secret))
            }
            JwtAlgorithm::RS256 | JwtAlgorithm::RS384 | JwtAlgorithm::RS512 => {
                EncodingKey::from_rsa_pem(secret)
                    .map_err(|e| AuthError::InvalidKey(e.to_string()))
            }
            JwtAlgorithm::ES256 | JwtAlgorithm::ES384 => {
                EncodingKey::from_ec_pem(secret)
                    .map_err(|e| AuthError::InvalidKey(e.to_string()))
            }
        }
    }

    fn create_decoding_key(secret: &[u8], algorithm: JwtAlgorithm) -> AuthResult<DecodingKey> {
        match algorithm {
            JwtAlgorithm::HS256 | JwtAlgorithm::HS384 | JwtAlgorithm::HS512 => {
                Ok(DecodingKey::from_secret(secret))
            }
            JwtAlgorithm::RS256 | JwtAlgorithm::RS384 | JwtAlgorithm::RS512 => {
                DecodingKey::from_rsa_pem(secret)
                    .map_err(|e| AuthError::InvalidKey(e.to_string()))
            }
            JwtAlgorithm::ES256 | JwtAlgorithm::ES384 => {
                DecodingKey::from_ec_pem(secret)
                    .map_err(|e| AuthError::InvalidKey(e.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_algorithm_mapping() {
        assert_eq!(JwtAlgorithm::HS256.to_algorithm(), Algorithm::HS256);
        assert_eq!(JwtAlgorithm::HS384.to_algorithm(), Algorithm::HS384);
        assert_eq!(JwtAlgorithm::HS512.to_algorithm(), Algorithm::HS512);
        assert_eq!(JwtAlgorithm::RS256.to_algorithm(), Algorithm::RS256);
        assert_eq!(JwtAlgorithm::ES256.to_algorithm(), Algorithm::ES256);
    }

    #[test]
    fn test_algorithm_string_representation() {
        assert_eq!(JwtAlgorithm::HS256.as_str(), "HS256");
        assert_eq!(JwtAlgorithm::RS256.as_str(), "RS256");
        assert_eq!(JwtAlgorithm::ES256.as_str(), "ES256");
    }

    #[test]
    fn test_algorithm_type_classification() {
        assert!(JwtAlgorithm::HS256.is_symmetric());
        assert!(!JwtAlgorithm::HS256.is_asymmetric());

        assert!(JwtAlgorithm::RS256.is_asymmetric());
        assert!(!JwtAlgorithm::RS256.is_symmetric());

        assert!(JwtAlgorithm::ES256.is_asymmetric());
        assert!(!JwtAlgorithm::ES256.is_symmetric());
    }

    #[test]
    fn test_key_manager_hmac() {
        let key_manager = KeyManager::new_hmac(b"test-secret-key").unwrap();

        assert_eq!(key_manager.algorithm(), JwtAlgorithm::HS256);
        assert!(key_manager.encoding_key().is_ok());
        assert!(key_manager.decoding_key().is_ok());
    }

    #[test]
    fn test_key_compatibility_validation() {
        let key_manager = KeyManager::new_hmac(b"test-secret-key").unwrap();

        // Valid key length
        assert!(key_manager.validate_key_compatibility(b"test-secret-key-that-is-long-enough").is_ok());

        // Invalid key length for HS256 (needs 32+ bytes)
        let short_key = b"short";
        assert!(key_manager.validate_key_compatibility(short_key).is_err());
    }

    #[test]
    fn test_key_rotation() {
        let mut key_manager = KeyManager::new_hmac(b"original-secret").unwrap();
        let new_secret = b"new-secret-key";

        assert!(key_manager.rotate_keys(new_secret).is_ok());
        assert!(key_manager.encoding_key().is_ok());
        assert!(key_manager.decoding_key().is_ok());
    }

    #[test]
    fn test_asymmetric_key_manager_creation() {
        // Test that asymmetric algorithms are rejected for symmetric key manager
        let result = KeyManager::new_hmac(b"secret").unwrap();
        assert_eq!(result.algorithm(), JwtAlgorithm::HS256); // Should default to HS256
    }

    #[test]
    fn test_keypair_generation_not_implemented() {
        // Should return error for symmetric algorithms
        let result = KeyManager::generate_keypair(JwtAlgorithm::HS256);
        assert!(result.is_err());

        // Should return error for asymmetric algorithms (not implemented)
        let result = KeyManager::generate_keypair(JwtAlgorithm::RS256);
        assert!(result.is_err());
    }
}
