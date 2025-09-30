//! # CloudShuttle Authentication Utilities
//!
//! Common authentication and JWT handling utilities used across CloudShuttle services.
//!
//! ## Features
//!
//! - JWT token creation and validation
//! - Claims structure and management
//! - Authentication middleware
//! - Token refresh utilities
//! - Secure key management
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_auth::{JwtService, Claims};
//!
//! // Create JWT service
//! let jwt_service = JwtService::new("your-secret-key".as_bytes())?;
//!
//! // Create token
//! let claims = Claims::new("user-123", "tenant-456");
//! let token = jwt_service.create_token(&claims)?;
//!
//! // Validate token
//! let validated_claims = jwt_service.validate_token(&token)?;
//! ```

pub mod jwt;
pub mod claims;
#[cfg(feature = "middleware")]
pub mod middleware;
pub mod keys;
pub mod types;
pub mod refresh;

// Re-export main types
pub use jwt::JwtService;
pub use claims::Claims;
#[cfg(feature = "middleware")]
pub use middleware::AuthMiddleware;
pub use keys::{KeyManager, SigningKeyPair as KeyPair};
pub use types::*;
pub use refresh::TokenRefresh;
