//! Authentication middleware for Axum.
//!
//! This module orchestrates multiple middleware components through specialized sub-modules:
//! - `layers`: Core authentication middleware layers
//! - `extractors`: Request extractors for authenticated users
//! - `guards`: Authorization guards for roles and permissions

pub mod layers;
pub mod extractors;
pub mod guards;

// Re-export for backward compatibility and convenience
pub use layers::{AuthMiddleware, CorsAuthLayer};
pub use extractors::{AuthenticatedUser, OptionalUser, extract_token_from_header};
pub use guards::{RoleGuard, PermissionGuard, TenantGuard};

// Re-export types needed by middleware
pub use crate::{Claims, AuthError, MiddlewareFn};
