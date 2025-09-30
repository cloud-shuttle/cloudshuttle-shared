//! # CloudShuttle Configuration Management
//!
//! Centralized configuration loading and validation for CloudShuttle services.
//!
//! ## Features
//!
//! - Environment-based configuration
//! - File-based configuration
//! - Validation with detailed error messages
//! - Secret management
//! - Hot reloading support
//! - Type-safe configuration access
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_config::ConfigLoader;
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize, Validate)]
//! pub struct AppConfig {
//!     pub database_url: String,
//!     pub port: u16,
//!     #[validate(range(min = 1, max = 100))]
//!     pub max_connections: u32,
//! }
//!
//! let config: AppConfig = ConfigLoader::new("my-service")
//!     .with_env_prefix("MY_SERVICE")
//!     .load()?;
//! ```

pub mod loader;
pub mod validator;
pub mod secrets;
pub mod hot_reload;

// Re-export main types
pub use loader::ConfigLoader;
pub use secrets::Secret;
pub use validator::ValidationError;
