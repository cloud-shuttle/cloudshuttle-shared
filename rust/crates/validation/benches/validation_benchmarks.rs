//! Performance benchmarks for validation operations
//!
//! Run with: cargo bench --bench validation_benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cloudshuttle_validation::{validate_email, validate_password_strength, validate_username, sanitize_html, sanitize_sql_input, sanitize_filename, AdvancedValidator, ValidationConfig, ValidationContext, ValidationSeverity, ValidationRule};
use std::collections::HashMap;

fn bench_basic_validation(c: &mut Criterion) {
    c.bench_function("validate_email", |b| {
        b.iter(|| {
            black_box(validate_email("user@example.com"));
        });
    });

    c.bench_function("validate_email_invalid", |b| {
        b.iter(|| {
            black_box(validate_email("invalid-email"));
        });
    });

    c.bench_function("validate_password_strength", |b| {
        b.iter(|| {
            black_box(validate_password_strength("MySecureP@ssw0rd123!"));
        });
    });

    c.bench_function("validate_username", |b| {
        b.iter(|| {
            black_box(validate_username("testuser123"));
        });
    });
}

fn bench_sanitization(c: &mut Criterion) {
    let malicious_html = "<script>alert('xss')</script><p>Hello World</p><iframe src='evil.com'></iframe>";
    c.bench_function("sanitize_html_malicious", |b| {
        b.iter(|| {
            black_box(sanitize_html(malicious_html));
        });
    });

    let safe_html = "<p>Hello <strong>World</strong></p>";
    c.bench_function("sanitize_html_safe", |b| {
        b.iter(|| {
            black_box(sanitize_html(safe_html));
        });
    });

    let malicious_sql = "'; DROP TABLE users; --";
    c.bench_function("sanitize_sql_input", |b| {
        b.iter(|| {
            black_box(sanitize_sql_input(malicious_sql, false).unwrap());
        });
    });

    let dangerous_filename = "../../../etc/passwd";
    c.bench_function("sanitize_filename", |b| {
        b.iter(|| {
            black_box(sanitize_filename(dangerous_filename));
        });
    });
}

fn bench_advanced_validation(c: &mut Criterion) {
    let config = ValidationConfig::default();
    let mut validator = AdvancedValidator::new(config);

    // Add business rules
    validator.add_business_rule("email", ValidationRule {
        name: "email_format".to_string(),
        description: "Validate email format".to_string(),
        severity: ValidationSeverity::Error,
        enabled: true,
        config: HashMap::new(),
    });

    validator.add_business_rule("password", ValidationRule {
        name: "strong_password".to_string(),
        description: "Validate password strength".to_string(),
        severity: ValidationSeverity::Error,
        enabled: true,
        config: HashMap::new(),
    });

    c.bench_function("advanced_validation_simple", |b| {
        b.iter(|| {
            let context = ValidationContext {
                field_name: "email".to_string(),
                field_value: serde_json::json!("user@example.com"),
                context_data: HashMap::new(),
                user_id: None,
                request_id: None,
            };
            black_box(validator.validate(context));
        });
    });

    c.bench_function("advanced_validation_complex", |b| {
        b.iter(|| {
            let context = ValidationContext {
                field_name: "password".to_string(),
                field_value: serde_json::json!("MySecureP@ssw0rd123!"),
                context_data: HashMap::new(),
                user_id: Some("user-123".to_string()),
                request_id: Some("req-456".to_string()),
            };
            black_box(validator.validate(context));
        });
    });
}

fn bench_enterprise_validation_scenarios(c: &mut Criterion) {
    let config = ValidationConfig {
        max_length: 10000,
        enable_security_scan: true,
        enable_business_rules: true,
        enable_sanitization: true,
        ..Default::default()
    };
    let mut validator = AdvancedValidator::new(config);

    // Add comprehensive business rules
    validator.add_business_rule("email", ValidationRule {
        name: "email_format".to_string(),
        description: "Validate email format".to_string(),
        severity: ValidationSeverity::Error,
        enabled: true,
        config: HashMap::new(),
    });

    validator.add_business_rule("username", ValidationRule {
        name: "username_format".to_string(),
        description: "Validate username format".to_string(),
        severity: ValidationSeverity::Error,
        enabled: true,
        config: HashMap::new(),
    });

    validator.add_business_rule("comment", ValidationRule {
        name: "no_consecutive_spaces".to_string(),
        description: "Check for consecutive spaces".to_string(),
        severity: ValidationSeverity::Warning,
        enabled: true,
        config: HashMap::new(),
    });

    c.bench_function("enterprise_user_registration_validation", |b| {
        b.iter(|| {
            let test_cases = vec![
                ("email", serde_json::json!("newuser@company.com")),
                ("username", serde_json::json!("newuser123")),
                ("password", serde_json::json!("SecureP@ss123!")),
                ("first_name", serde_json::json!("John")),
                ("last_name", serde_json::json!("Doe")),
                ("comment", serde_json::json!("This is a user comment")),
            ];

            for (field_name, field_value) in &test_cases {
                let context = ValidationContext {
                    field_name: field_name.to_string(),
                    field_value: field_value.clone(),
                    context_data: HashMap::new(),
                    user_id: Some("admin-user".to_string()),
                    request_id: Some("reg-123".to_string()),
                };
                black_box(validator.validate(context));
            }
        });
    });

    c.bench_function("enterprise_security_scan", |b| {
        b.iter(|| {
            let malicious_inputs = vec![
                ("sql_injection", serde_json::json!("'; DROP TABLE users; --")),
                ("xss_attack", serde_json::json!("<script>alert('xss')</script>")),
                ("path_traversal", serde_json::json!("../../../etc/passwd")),
                ("command_injection", serde_json::json!("; rm -rf /")),
            ];

            for (field_name, field_value) in &malicious_inputs {
                let context = ValidationContext {
                    field_name: field_name.to_string(),
                    field_value: field_value.clone(),
                    context_data: HashMap::new(),
                    user_id: Some("test-user".to_string()),
                    request_id: Some("sec-test-123".to_string()),
                };
                black_box(validator.validate(context));
            }
        });
    });
}

fn bench_validation_edge_cases(c: &mut Criterion) {
    let config = ValidationConfig::default();
    let validator = AdvancedValidator::new(config);

    c.bench_function("validation_large_input", |b| {
        b.iter(|| {
            let large_input = "A".repeat(5000);
            let context = ValidationContext {
                field_name: "large_field".to_string(),
                field_value: serde_json::json!(large_input),
                context_data: HashMap::new(),
                user_id: None,
                request_id: None,
            };
            black_box(validator.validate(context));
        });
    });

    c.bench_function("validation_many_rules", |b| {
        b.iter(|| {
            let mut local_validator = AdvancedValidator::new(ValidationConfig::default());

            // Add many rules
            for i in 0..20 {
                local_validator.add_business_rule(
                    &format!("field_{}", i),
                    ValidationRule {
                        name: format!("rule_{}", i),
                        description: format!("Rule {}", i),
                        severity: ValidationSeverity::Warning,
                        enabled: true,
                        config: HashMap::new(),
                    }
                );
            }

            let context = ValidationContext {
                field_name: "field_0".to_string(),
                field_value: serde_json::json!("test_value"),
                context_data: HashMap::new(),
                user_id: None,
                request_id: None,
            };
            black_box(local_validator.validate(context));
        });
    });
}

criterion_group!(
    benches,
    bench_basic_validation,
    bench_sanitization,
    bench_advanced_validation,
    bench_enterprise_validation_scenarios,
    bench_validation_edge_cases
);
criterion_main!(benches);
