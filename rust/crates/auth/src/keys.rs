//! Key management for JWT tokens

use ring::signature::{Ed25519KeyPair, KeyPair as RingKeyPair};
use ring::rand::SystemRandom;
use std::fs;
use std::path::Path;
use crate::{AuthResult, AuthError};

/// Key pair for asymmetric signing
#[derive(Clone)]
pub struct SigningKeyPair {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
    algorithm: KeyAlgorithm,
}

/// Supported key algorithms
#[derive(Debug, Clone, Copy)]
pub enum KeyAlgorithm {
    Ed25519,
    RSA,
    ECDSA,
}

impl SigningKeyPair {
    /// Generate a new Ed25519 key pair
    pub fn generate_ed25519() -> AuthResult<Self> {
        let rng = SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|_| AuthError::InvalidKey("Failed to generate Ed25519 key".to_string()))?;

        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .map_err(|_| AuthError::InvalidKey("Failed to create Ed25519 key pair".to_string()))?;

        Ok(Self {
            private_key: pkcs8_bytes.as_ref().to_vec(),
            public_key: key_pair.public_key().as_ref().to_vec(),
            algorithm: KeyAlgorithm::Ed25519,
        })
    }

    /// Generate a new RSA key pair
    pub fn generate_rsa() -> AuthResult<Self> {
        // RSA key generation would require additional dependencies
        // For now, return an error
        Err(AuthError::UnsupportedAlgorithm("RSA key generation not implemented".to_string()))
    }

    /// Generate a new ECDSA key pair
    pub fn generate_ecdsa() -> AuthResult<Self> {
        // ECDSA key generation would require additional dependencies
        // For now, return an error
        Err(AuthError::UnsupportedAlgorithm("ECDSA key generation not implemented".to_string()))
    }

    /// Load key pair from PEM files
    pub fn from_pem(private_key_path: &Path, public_key_path: &Path) -> AuthResult<Self> {
        let private_key = fs::read(private_key_path)
            .map_err(|e| AuthError::InvalidKey(format!("Failed to read private key: {}", e)))?;

        let public_key = fs::read(public_key_path)
            .map_err(|e| AuthError::InvalidKey(format!("Failed to read public key: {}", e)))?;

        // Determine algorithm from key content
        let algorithm = if private_key.starts_with(b"-----BEGIN ED25519 PRIVATE KEY-----") {
            KeyAlgorithm::Ed25519
        } else if private_key.starts_with(b"-----BEGIN RSA PRIVATE KEY-----") {
            KeyAlgorithm::RSA
        } else if private_key.starts_with(b"-----BEGIN EC PRIVATE KEY-----") {
            KeyAlgorithm::ECDSA
        } else {
            return Err(AuthError::InvalidKey("Unknown key format".to_string()));
        };

        Ok(Self {
            private_key,
            public_key,
            algorithm,
        })
    }

    /// Get private key bytes
    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }

    /// Get public key bytes
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Get algorithm
    pub fn algorithm(&self) -> KeyAlgorithm {
        self.algorithm
    }

    /// Save keys to PEM files
    pub fn save_to_pem(&self, private_key_path: &Path, public_key_path: &Path) -> AuthResult<()> {
        fs::write(private_key_path, &self.private_key)
            .map_err(|e| AuthError::InvalidKey(format!("Failed to write private key: {}", e)))?;

        fs::write(public_key_path, &self.public_key)
            .map_err(|e| AuthError::InvalidKey(format!("Failed to write public key: {}", e)))?;

        Ok(())
    }
}

/// Key manager for handling key rotation and multiple keys
pub struct KeyManager {
    current_key: SigningKeyPair,
    previous_keys: Vec<SigningKeyPair>,
    key_rotation_enabled: bool,
    rotation_interval_days: u32,
}

impl KeyManager {
    /// Create a new key manager with a single key
    pub fn new(key: SigningKeyPair) -> Self {
        Self {
            current_key: key,
            previous_keys: Vec::new(),
            key_rotation_enabled: false,
            rotation_interval_days: 30,
        }
    }

    /// Enable automatic key rotation
    pub fn with_rotation(mut self, interval_days: u32) -> Self {
        self.key_rotation_enabled = true;
        self.rotation_interval_days = interval_days;
        self
    }

    /// Get the current signing key
    pub fn current_key(&self) -> &SigningKeyPair {
        &self.current_key
    }

    /// Get all valid keys (current + previous for validation)
    pub fn all_keys(&self) -> Vec<&SigningKeyPair> {
        let mut keys = vec![&self.current_key];
        keys.extend(self.previous_keys.iter());
        keys
    }

    /// Rotate to a new key
    pub fn rotate_key(&mut self, new_key: SigningKeyPair) -> AuthResult<()> {
        self.previous_keys.push(self.current_key.clone());
        self.current_key = new_key;

        // Keep only the last few previous keys to prevent unlimited growth
        if self.previous_keys.len() > 5 {
            self.previous_keys.remove(0);
        }

        Ok(())
    }

    /// Check if key rotation is needed
    pub fn should_rotate(&self) -> bool {
        // This would typically check against a timestamp stored with the key
        // For now, always return false
        false
    }

    /// Clean up old keys (remove keys older than retention period)
    pub fn cleanup_old_keys(&mut self) {
        // Keep only recent keys
        while self.previous_keys.len() > 3 {
            self.previous_keys.remove(0);
        }
    }

    /// Get key by ID or index
    pub fn get_key(&self, index: usize) -> Option<&SigningKeyPair> {
        if index == 0 {
            Some(&self.current_key)
        } else {
            self.previous_keys.get(index - 1)
        }
    }

    /// Validate a key exists
    pub fn has_key(&self, index: usize) -> bool {
        if index == 0 {
            true
        } else {
            index - 1 < self.previous_keys.len()
        }
    }
}

/// Key store for persistent key management
pub struct KeyStore {
    keys_dir: std::path::PathBuf,
    current_key_id: String,
}

impl KeyStore {
    /// Create a new key store
    pub fn new(keys_dir: std::path::PathBuf) -> Self {
        Self {
            keys_dir,
            current_key_id: "current".to_string(),
        }
    }

    /// Load the current key pair
    pub fn load_current_key(&self) -> AuthResult<SigningKeyPair> {
        let private_key_path = self.keys_dir.join(format!("{}_private.pem", self.current_key_id));
        let public_key_path = self.keys_dir.join(format!("{}_public.pem", self.current_key_id));

        if private_key_path.exists() && public_key_path.exists() {
            SigningKeyPair::from_pem(&private_key_path, &public_key_path)
        } else {
            // Generate new keys if they don't exist
            let key_pair = SigningKeyPair::generate_ed25519()?;
            key_pair.save_to_pem(&private_key_path, &public_key_path)?;
            Ok(key_pair)
        }
    }

    /// Save a key pair with a specific ID
    pub fn save_key(&self, key: &SigningKeyPair, key_id: &str) -> AuthResult<()> {
        let private_key_path = self.keys_dir.join(format!("{}_private.pem", key_id));
        let public_key_path = self.keys_dir.join(format!("{}_public.pem", key_id));

        key.save_to_pem(&private_key_path, &public_key_path)
    }

    /// List all available key IDs
    pub fn list_keys(&self) -> AuthResult<Vec<String>> {
        let mut keys = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.keys_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with("_private.pem") {
                            let key_id = file_name.trim_end_matches("_private.pem");
                            keys.push(key_id.to_string());
                        }
                    }
                }
            }
        }

        Ok(keys)
    }

    /// Delete a key pair
    pub fn delete_key(&self, key_id: &str) -> AuthResult<()> {
        let private_key_path = self.keys_dir.join(format!("{}_private.pem", key_id));
        let public_key_path = self.keys_dir.join(format!("{}_public.pem", key_id));

        if private_key_path.exists() {
            fs::remove_file(private_key_path)
                .map_err(|e| AuthError::InvalidKey(format!("Failed to delete private key: {}", e)))?;
        }

        if public_key_path.exists() {
            fs::remove_file(public_key_path)
                .map_err(|e| AuthError::InvalidKey(format!("Failed to delete public key: {}", e)))?;
        }

        Ok(())
    }

    /// Backup current keys
    pub fn backup_keys(&self, backup_dir: &Path) -> AuthResult<()> {
        fs::create_dir_all(backup_dir)
            .map_err(|e| AuthError::InvalidKey(format!("Failed to create backup dir: {}", e)))?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_subdir = backup_dir.join(format!("backup_{}", timestamp));

        fs::create_dir_all(&backup_subdir)
            .map_err(|e| AuthError::InvalidKey(format!("Failed to create backup subdir: {}", e)))?;

        for key_id in self.list_keys()? {
            let src_private = self.keys_dir.join(format!("{}_private.pem", key_id));
            let src_public = self.keys_dir.join(format!("{}_public.pem", key_id));
            let dst_private = backup_subdir.join(format!("{}_private.pem", key_id));
            let dst_public = backup_subdir.join(format!("{}_public.pem", key_id));

            if src_private.exists() {
                fs::copy(&src_private, &dst_private)
                    .map_err(|e| AuthError::InvalidKey(format!("Failed to backup private key: {}", e)))?;
            }

            if src_public.exists() {
                fs::copy(&src_public, &dst_public)
                    .map_err(|e| AuthError::InvalidKey(format!("Failed to backup public key: {}", e)))?;
            }
        }

        Ok(())
    }
}

/// Key rotation policy
#[derive(Debug, Clone)]
pub struct KeyRotationPolicy {
    pub enabled: bool,
    pub interval_days: u32,
    pub keep_previous_keys: usize,
    pub backup_before_rotation: bool,
}

impl Default for KeyRotationPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_days: 30,
            keep_previous_keys: 3,
            backup_before_rotation: true,
        }
    }
}
