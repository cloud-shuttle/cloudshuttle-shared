//! Performance benchmarks for authentication operations
//!
//! Run with: cargo bench --bench auth_benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cloudshuttle_auth::{JwtService, Claims, SecurityValidator};
use std::collections::HashMap;

fn bench_jwt_service_creation(c: &mut Criterion) {
    c.bench_function("jwt_service_creation", |b| {
        b.iter(|| {
            black_box(JwtService::new(b"benchmark-secret-key-that-is-long-enough-for-performance-testing").unwrap());
        });
    });
}

fn bench_claims_creation(c: &mut Criterion) {
    c.bench_function("claims_creation_simple", |b| {
        b.iter(|| {
            black_box(Claims::new("user-123", "tenant-456"));
        });
    });

    c.bench_function("claims_creation_complex", |b| {
        b.iter(|| {
            let mut claims = Claims::new("user-123", "tenant-456");
            claims.roles = vec!["admin".to_string(), "user".to_string()];
            claims.permissions = vec!["read".to_string(), "write".to_string()];
            claims.custom = HashMap::new();
            claims.custom.insert("department".to_string(), serde_json::json!("engineering"));
            claims.custom.insert("clearance".to_string(), serde_json::json!("high"));
            black_box(claims);
        });
    });
}

fn bench_security_validation(c: &mut Criterion) {
    let validator = SecurityValidator;

    c.bench_function("security_validate_email", |b| {
        b.iter(|| {
            black_box(validator.validate_email("user@example.com"));
        });
    });

    c.bench_function("security_validate_password", |b| {
        b.iter(|| {
            black_box(validator.validate_password_strength("MySecureP@ssw0rd123!"));
        });
    });

    c.bench_function("security_validate_username", |b| {
        b.iter(|| {
            black_box(validator.validate_username("testuser123"));
        });
    });

    let malicious_input = "<script>alert('xss')</script><p>Hello</p><iframe src='evil.com'></iframe>";
    c.bench_function("security_sanitize_html", |b| {
        b.iter(|| {
            black_box(validator.sanitize_html(malicious_input));
        });
    });

    let sql_input = "'; DROP TABLE users; --";
    c.bench_function("security_sanitize_sql", |b| {
        b.iter(|| {
            black_box(validator.sanitize_sql_input(sql_input, false).unwrap());
        });
    });
}

fn bench_enterprise_workloads(c: &mut Criterion) {
    let service = JwtService::new(b"benchmark-secret-key-that-is-long-enough-for-performance-testing").unwrap();

    c.bench_function("enterprise_user_authentication_flow", |b| {
        b.iter(|| {
            // Simulate a complete authentication flow
            let user_id = "enterprise-user-123";
            let tenant_id = "enterprise-tenant";

            // Create claims for new user
            let mut claims = Claims::new(user_id, tenant_id);
            claims.roles = vec!["user".to_string(), "editor".to_string()];
            claims.permissions = vec!["read".to_string(), "write".to_string(), "create".to_string()];

            // Create JWT token
            let token = service.create_token(&claims).unwrap();

            // Validate token (simulating middleware validation)
            let validated = service.validate_token(&token).unwrap();

            // Verify claims
            assert_eq!(validated.sub, user_id);
            assert_eq!(validated.tenant_id, tenant_id);
            assert!(validated.roles.contains(&"user".to_string()));

            black_box((token, validated));
        });
    });

    c.bench_function("enterprise_token_validation_bulk", |b| {
        b.iter(|| {
            // Pre-create tokens for bulk validation
            let tokens: Vec<String> = (0..50).map(|i| {
                let claims = Claims::new(&format!("user-{}", i), "enterprise-tenant");
                service.create_token(&claims).unwrap()
            }).collect();

            // Validate all tokens
            for token in &tokens {
                let _validated = service.validate_token(token).unwrap();
            }

            black_box(tokens);
        });
    });
}

fn bench_security_edge_cases(c: &mut Criterion) {
    let validator = SecurityValidator;

    c.bench_function("security_large_input_validation", |b| {
        b.iter(|| {
            let large_input = "A".repeat(10000);
            black_box(validator.validate_username(&large_input));
        });
    });

    c.bench_function("security_complex_password_validation", |b| {
        b.iter(|| {
            let complex_password = "MyV3ryC0mpl3xP@ssw0rd!WithNumb3rsAndSp3c!alChars#2024";
            black_box(validator.validate_password_strength(complex_password));
        });
    });

    c.bench_function("security_malicious_input_detection", |b| {
        b.iter(|| {
            let malicious_inputs = vec![
                "<script>evil()</script>",
                "UNION SELECT * FROM users",
                "../../../etc/passwd",
                "javascript:alert('xss')",
                "<iframe src='evil.com'>",
                "'; DROP TABLE users; --",
            ];

            for input in &malicious_inputs {
                black_box(validator.sanitize_html(input));
                black_box(validator.sanitize_sql_input(input, false));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_jwt_service_creation,
    bench_claims_creation,
    bench_security_validation,
    bench_enterprise_workloads,
    bench_security_edge_cases
);
criterion_main!(benches);
