//! Input sanitization implementations.
//!
//! This module contains various sanitizer implementations for cleaning
//! and securing user input including HTML, SQL, and filename sanitization.

use std::collections::HashSet;
use regex::Regex;
use crate::validation::types::{Sanitizer};

/// HTML sanitizer for removing dangerous tags and attributes
#[derive(Debug)]
pub struct HtmlSanitizer {
    allowed_tags: HashSet<String>,
}

impl HtmlSanitizer {
    pub fn new() -> Self {
        let mut allowed_tags = HashSet::new();
        allowed_tags.insert("b".to_string());
        allowed_tags.insert("i".to_string());
        allowed_tags.insert("em".to_string());
        allowed_tags.insert("strong".to_string());
        allowed_tags.insert("p".to_string());
        allowed_tags.insert("br".to_string());

        Self { allowed_tags }
    }

    pub fn with_allowed_tags(mut self, tags: Vec<String>) -> Self {
        self.allowed_tags = tags.into_iter().collect();
        self
    }
}

impl Sanitizer for HtmlSanitizer {
    fn sanitize(&self, value: &str) -> String {
        // Simple HTML sanitization - remove script tags and unwanted attributes
        let mut result = value.to_string();

        // Remove script tags completely
        let script_regex = Regex::new(r"<script[^>]*>.*?</script>").unwrap();
        result = script_regex.replace_all(&result, "").to_string();

        // Remove other dangerous tags
        let dangerous_tags = ["iframe", "object", "embed", "form", "input", "meta", "link"];
        for tag in &dangerous_tags {
            let tag_regex = Regex::new(&format!(r"<{}[^>]*>.*?</{}>", tag, tag)).unwrap();
            result = tag_regex.replace_all(&result, "").to_string();
        }

        // Remove event handlers
        let event_regex = Regex::new(r"\s+on\w+\s*=\s*[^>\s]*").unwrap();
        result = event_regex.replace_all(&result, "").to_string();

        result
    }

    fn clone_box(&self) -> Box<dyn Sanitizer> {
        Box::new(Self {
            allowed_tags: self.allowed_tags.clone(),
        })
    }
}

/// SQL sanitizer (basic)
#[derive(Debug)]
pub struct SqlSanitizer;

impl SqlSanitizer {
    pub fn new() -> Self {
        Self
    }
}

impl Sanitizer for SqlSanitizer {
    fn sanitize(&self, value: &str) -> String {
        // Basic SQL sanitization - escape single quotes
        value.replace("'", "''")
    }

    fn clone_box(&self) -> Box<dyn Sanitizer> {
        Box::new(Self)
    }
}

/// Filename sanitizer
#[derive(Debug)]
pub struct FilenameSanitizer;

impl FilenameSanitizer {
    pub fn new() -> Self {
        Self
    }
}

impl Sanitizer for FilenameSanitizer {
    fn sanitize(&self, value: &str) -> String {
        // Remove dangerous characters for filenames
        let dangerous_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

        let mut result = value.chars()
            .filter(|c| !dangerous_chars.contains(c))
            .collect::<String>();

        // Remove directory traversal sequences
        result = result.replace("..", "");

        result.trim().to_string()
    }

    fn clone_box(&self) -> Box<dyn Sanitizer> {
        Box::new(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_sanitizer() {
        let sanitizer = HtmlSanitizer::new();
        let malicious_input = "<script>alert('xss')</script><p>Hello</p>";
        let sanitized = sanitizer.sanitize(malicious_input);

        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("<p>Hello</p>"));
    }

    #[test]
    fn test_sql_sanitizer() {
        let sanitizer = SqlSanitizer::new();
        let malicious_input = "'; DROP TABLE users; --";
        let sanitized = sanitizer.sanitize(malicious_input);

        assert_eq!(sanitized, "''; DROP TABLE users; --");
    }

    #[test]
    fn test_filename_sanitizer() {
        let sanitizer = FilenameSanitizer::new();
        let malicious_input = "../../../etc/passwd";
        let sanitized = sanitizer.sanitize(malicious_input);

        assert!(!sanitized.contains(".."));
        assert!(!sanitized.contains("/"));
        assert!(!sanitized.contains("\\"));
    }
}
