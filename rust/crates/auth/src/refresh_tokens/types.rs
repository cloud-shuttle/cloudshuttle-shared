//! Refresh token data structures and types.
//!
//! This module contains all the fundamental types used by the refresh token system
//! including configuration, records, requests, and responses.

use serde::{Deserialize, Serialize};

/// Refresh token configuration and policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenConfig {
    /// Maximum lifetime of refresh tokens (seconds)
    pub max_lifetime: u64,

    /// Whether to rotate refresh tokens on use
    pub rotation_enabled: bool,

    /// Maximum number of active refresh tokens per user
    pub max_tokens_per_user: usize,

    /// Whether to revoke all tokens when a security event occurs
    pub revoke_on_security_event: bool,

    /// Family ID for token grouping (for cascade revocation)
    pub family_id: Option<String>,
}

impl Default for RefreshTokenConfig {
    fn default() -> Self {
        Self {
            max_lifetime: 30 * 24 * 60 * 60, // 30 days
            rotation_enabled: true,
            max_tokens_per_user: 5,
            revoke_on_security_event: true,
            family_id: None,
        }
    }
}

/// Stored refresh token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRecord {
    /// Unique token identifier
    pub token_id: String,

    /// User ID associated with the token
    pub user_id: String,

    /// Family ID for cascade operations
    pub family_id: Option<String>,

    /// Token creation timestamp
    pub created_at: u64,

    /// Token expiration timestamp
    pub expires_at: u64,

    /// Device/client identifier
    pub device_id: Option<String>,

    /// IP address when token was issued
    pub ip_address: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Whether this token has been revoked
    pub revoked: bool,

    /// Reason for revocation (if applicable)
    pub revocation_reason: Option<String>,
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    /// The refresh token to use
    pub refresh_token: String,

    /// Optional device/client identifier for tracking
    pub device_id: Option<String>,

    /// Optional scope to request (subset of original)
    pub scope: Option<String>,
}

/// Refresh token response
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    /// New access token
    pub access_token: String,

    /// New refresh token (if rotation enabled)
    pub refresh_token: Option<String>,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// Access token expiration time
    pub expires_in: u64,

    /// Granted scope
    pub scope: Option<String>,
}
