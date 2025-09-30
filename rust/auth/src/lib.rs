//! # CloudShuttle Authentication
//!
//! Common authentication utilities and JWT handling for CloudShuttle services.
//!
//! This crate provides:
//! - JWT token creation and validation
//! - Authentication middleware for web frameworks
//! - Token claims structures
//! - Key management utilities
//! - Password hashing and verification
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_auth::{JwtService, Claims};
//!
//! // Create JWT service
//! let jwt_service = JwtService::new("your-secret-key")?;
//!
//! // Create token
//! let claims = Claims {
//!     sub: "user123".to_string(),
//!     exp: 1234567890,
//!     iat: 1234567800,
//!     tenant_id: Uuid::new_v4(),
//!     roles: vec!["admin".to_string()],
//! };
//!
//! let token = jwt_service.create_token(&claims)?;
//!
//! // Validate token
//! let decoded_claims = jwt_service.validate_token(&token)?;
//! ```

pub mod jwt;
pub mod claims;
pub mod middleware;
pub mod keys;
pub mod password;

// Re-export main types for convenience
pub use jwt::JwtService;
pub use claims::Claims;
pub use password::{PasswordHasher, PasswordHashError};
