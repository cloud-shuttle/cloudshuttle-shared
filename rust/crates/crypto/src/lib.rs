//! # CloudShuttle Cryptography
//!
//! Secure cryptographic utilities for CloudShuttle services.
//!
//! ## Features
//!
//! - Password hashing with Argon2
//! - AES encryption/decryption
//! - Secure random generation
//! - Key derivation
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_crypto::{hash_password, verify_password};
//!
//! // Hash a password
//! let hash = hash_password("my-password")?;
//!
//! // Verify a password
//! assert!(verify_password("my-password", &hash)?);
//! ```

pub mod hashing;
pub mod encryption;
pub mod random;

// Re-export main functions
pub use hashing::{hash_password, verify_password};
pub use encryption::{encrypt_data, decrypt_data};
pub use random::generate_secure_token;
