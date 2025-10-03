//! Rate limiting middleware for API endpoints
//!
//! This module provides configurable rate limiting functionality
//! that can be applied to API endpoints to prevent abuse.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use axum::{
    extract::Request,
    middleware::Next,
    response::{Response, IntoResponse},
    http::StatusCode,
};

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in the window
    pub max_requests: u32,
    /// Time window duration in seconds
    pub window_seconds: u64,
    /// Whether to use IP-based limiting (default) or user-based
    pub by_ip: bool,
    /// Custom identifier extractor function
    pub identifier_extractor: Option<fn(&Request) -> String>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60, // 1 minute
            by_ip: true,
            identifier_extractor: None,
        }
    }
}

/// Rate limiting state for a single identifier
#[derive(Debug)]
struct RateLimitState {
    requests: Vec<Instant>,
    window_start: Instant,
}

impl RateLimitState {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            window_start: Instant::now(),
        }
    }

    fn is_allowed(&mut self, max_requests: u32, window_duration: Duration) -> bool {
        let now = Instant::now();

        // Check if we need to reset the window
        if now.duration_since(self.window_start) >= window_duration {
            self.requests.clear();
            self.window_start = now;
        }

        // Remove old requests outside the current window
        let cutoff = now - window_duration;
        self.requests.retain(|&time| time > cutoff);

        // Check if under limit
        if self.requests.len() < max_requests as usize {
            self.requests.push(now);
            true
        } else {
            false
        }
    }

    fn remaining_requests(&self, max_requests: u32, window_duration: Duration) -> u32 {
        let now = Instant::now();

        // Clean up old requests
        let cutoff = now - window_duration;
        let valid_requests = self.requests.iter().filter(|&&time| time > cutoff).count();

        max_requests.saturating_sub(valid_requests as u32)
    }

    fn reset_time(&self, window_duration: Duration) -> Option<Instant> {
        if self.requests.is_empty() {
            None
        } else {
            Some(self.window_start + window_duration)
        }
    }
}

/// In-memory rate limiter storage
#[derive(Debug)]
pub struct InMemoryRateLimiter {
    states: Mutex<HashMap<String, RateLimitState>>,
    config: RateLimitConfig,
}

impl InMemoryRateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
            config,
        }
    }

    /// Check if a request is allowed
    pub fn check_limit(&self, identifier: &str) -> RateLimitResult {
        let mut states = self.states.lock().unwrap();
        let state = states.entry(identifier.to_string()).or_insert_with(RateLimitState::new);

        let window_duration = Duration::from_secs(self.config.window_seconds);

        if state.is_allowed(self.config.max_requests, window_duration) {
            let remaining = state.remaining_requests(self.config.max_requests, window_duration);
            let reset_time = state.reset_time(window_duration);
            RateLimitResult::Allowed { remaining, reset_time }
        } else {
            let reset_time = state.reset_time(window_duration);
            RateLimitResult::Exceeded { reset_time }
        }
    }

    /// Get remaining requests for an identifier
    pub fn remaining_requests(&self, identifier: &str) -> u32 {
        let states = self.states.lock().unwrap();
        if let Some(state) = states.get(identifier) {
            let window_duration = Duration::from_secs(self.config.window_seconds);
            state.remaining_requests(self.config.max_requests, window_duration)
        } else {
            self.config.max_requests
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    /// Clear all rate limit states (useful for testing)
    pub fn clear(&self) {
        let mut states = self.states.lock().unwrap();
        states.clear();
    }
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    /// Request is allowed
    Allowed {
        /// Remaining requests in current window
        remaining: u32,
        /// When the rate limit window resets
        reset_time: Option<Instant>,
    },
    /// Rate limit exceeded
    Exceeded {
        /// When the rate limit window resets
        reset_time: Option<Instant>,
    },
}

/// Rate limiting middleware
pub struct RateLimitMiddleware {
    limiter: Arc<InMemoryRateLimiter>,
}

impl RateLimitMiddleware {
    /// Create new rate limiting middleware
    pub fn new(limiter: Arc<InMemoryRateLimiter>) -> Self {
        Self { limiter }
    }

    /// Extract identifier from request (IP address by default)
    fn extract_identifier(&self, req: &Request) -> String {
        if let Some(extractor) = self.limiter.config.identifier_extractor {
            extractor(req)
        } else if self.limiter.config.by_ip {
            // Extract IP address from request
            req.headers()
                .get("x-forwarded-for")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.split(',').next())
                .unwrap_or("127.0.0.1")
                .to_string()
        } else {
            // For user-based limiting, this would need to be implemented
            // based on authentication context
            "anonymous".to_string()
        }
    }

    /// Create the middleware function
    pub fn layer(limiter: Arc<InMemoryRateLimiter>) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
        move |req: Request, next: Next| {
            let limiter = limiter.clone();
            Box::pin(async move {
                // Extract identifier and check rate limit
                let identifier = req.headers()
                    .get("x-forwarded-for")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.split(',').next())
                    .unwrap_or("127.0.0.1")
                    .to_string();

                match limiter.check_limit(&identifier) {
                    RateLimitResult::Allowed { .. } => {
                        next.run(req).await
                    }
                    RateLimitResult::Exceeded { reset_time } => {
                        let mut response = (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();

                        // Add rate limit headers
                        if let Some(reset_time) = reset_time {
                            let reset_timestamp = reset_time.elapsed().as_secs() as i64;
                            let headers = response.headers_mut();
                            headers.insert("x-ratelimit-reset", reset_timestamp.to_string().parse().unwrap());
                            headers.insert("retry-after", reset_timestamp.to_string().parse().unwrap());
                        }

                        response
                    }
                }
            })
        }
    }

    /// Check rate limit for a request (utility method)
    pub fn check_request(&self, req: &Request) -> RateLimitResult {
        let identifier = self.extract_identifier(req);
        self.limiter.check_limit(&identifier)
    }

    /// Get remaining requests for a request (utility method)
    pub fn remaining_requests(&self, req: &Request) -> u32 {
        let identifier = self.extract_identifier(req);
        self.limiter.remaining_requests(&identifier)
    }
}


/// Pre-configured rate limiters for common use cases
pub mod presets {
    use super::*;

    /// Create a rate limiter for general API endpoints (100 requests per minute)
    pub fn api_limiter() -> Arc<InMemoryRateLimiter> {
        Arc::new(InMemoryRateLimiter::new(RateLimitConfig {
            max_requests: 100,
            window_seconds: 60,
            ..Default::default()
        }))
    }

    /// Create a rate limiter for authentication endpoints (10 requests per minute)
    pub fn auth_limiter() -> Arc<InMemoryRateLimiter> {
        Arc::new(InMemoryRateLimiter::new(RateLimitConfig {
            max_requests: 10,
            window_seconds: 60,
            ..Default::default()
        }))
    }

    /// Create a rate limiter for search endpoints (50 requests per minute)
    pub fn search_limiter() -> Arc<InMemoryRateLimiter> {
        Arc::new(InMemoryRateLimiter::new(RateLimitConfig {
            max_requests: 50,
            window_seconds: 60,
            ..Default::default()
        }))
    }

    /// Create a rate limiter for file upload endpoints (5 requests per minute)
    pub fn upload_limiter() -> Arc<InMemoryRateLimiter> {
        Arc::new(InMemoryRateLimiter::new(RateLimitConfig {
            max_requests: 5,
            window_seconds: 60,
            ..Default::default()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_state() {
        let mut state = RateLimitState::new();
        let window = Duration::from_secs(60);

        // Should allow initial requests
        assert!(state.is_allowed(3, window));
        assert!(state.is_allowed(3, window));
        assert!(state.is_allowed(3, window));

        // Should deny fourth request
        assert!(!state.is_allowed(3, window));
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = InMemoryRateLimiter::new(RateLimitConfig {
            max_requests: 2,
            window_seconds: 60,
            ..Default::default()
        });

        // First request should be allowed
        match limiter.check_limit("test") {
            RateLimitResult::Allowed { remaining, .. } => assert_eq!(remaining, 1),
            _ => panic!("Expected allowed"),
        }

        // Second request should be allowed
        match limiter.check_limit("test") {
            RateLimitResult::Allowed { remaining, .. } => assert_eq!(remaining, 0),
            _ => panic!("Expected allowed"),
        }

        // Third request should be denied
        match limiter.check_limit("test") {
            RateLimitResult::Exceeded { .. } => {},
            _ => panic!("Expected exceeded"),
        }
    }

    #[test]
    fn test_presets() {
        let api_limiter = presets::api_limiter();
        assert_eq!(api_limiter.config.max_requests, 100);

        let auth_limiter = presets::auth_limiter();
        assert_eq!(auth_limiter.config.max_requests, 10);
    }
}
