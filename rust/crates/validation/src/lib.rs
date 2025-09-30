//! # CloudShuttle Validation
//!
//! Input validation and sanitization utilities for CloudShuttle services.
//!
//! ## Features
//!
//! - Common validation rules
//! - Input sanitization
//! - Custom validators
//! - Security-focused validation
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_validation::{validate_email, sanitize_html};
//!
//! // Validate email
//! assert!(validate_email("user@example.com").is_ok());
//!
//! // Sanitize HTML input
//! let clean = sanitize_html("<script>alert('xss')</script>Hello");
//! assert_eq!(clean, "Hello");
//! ```

pub mod rules;
pub mod sanitization;

// Re-export main functions
pub use rules::{validate_email, validate_password_strength, validate_username};
pub use sanitization::{sanitize_html, sanitize_sql_input, sanitize_filename};
