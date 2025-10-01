//! Authentication types and structures
//!
//! This module contains all authentication-related types organized by functionality.

pub mod errors;
pub mod credentials;
pub mod session;
pub mod policy;

// Re-export commonly used types for convenience
pub use errors::{AuthResult, AuthError};
pub use credentials::{UserCredentials, AuthTokens, RefreshTokenRequest, LoginRequest, PasswordChangeRequest};
pub use session::{UserSession, AuthContext, SessionStore, InMemorySessionStore};
pub use policy::{PasswordPolicy, LoginAttempt, LockoutPolicy, MfaConfig, MfaMethod, SecurityEvent, SecurityEventType, SecuritySeverity};

// Re-export all types for backward compatibility
