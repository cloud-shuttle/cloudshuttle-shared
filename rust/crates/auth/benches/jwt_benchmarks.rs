//! Performance benchmarks for JWT operations
//!
//! Run with: cargo bench --bench jwt_benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cloudshuttle_auth::{JwtService, Claims};
use chrono::Duration;

fn bench_jwt_operations(c: &mut Criterion) {
    let service = JwtService::new(b"benchmark-secret-key-that-is-long-enough-for-performance-testing").unwrap();
    let claims = Claims::new("benchmark-user", "benchmark-tenant");

    c.bench_function("jwt_token_creation", |b| {
        b.iter(|| {
            black_box(service.create_token(&claims).unwrap());
        });
    });

    let token = service.create_token(&claims).unwrap();
    c.bench_function("jwt_token_validation", |b| {
        b.iter(|| {
            black_box(service.validate_token(&token).unwrap());
        });
    });
}

fn bench_jwt_with_complex_claims(c: &mut Criterion) {
    let service = JwtService::new(b"benchmark-secret-key-that-is-long-enough-for-performance-testing").unwrap();
    let mut claims = Claims::new("benchmark-user", "benchmark-tenant");
    claims.roles = vec![
        "admin".to_string(),
        "user".to_string(),
        "moderator".to_string(),
        "editor".to_string(),
        "viewer".to_string(),
    ];
    claims.permissions = vec![
        "read".to_string(),
        "write".to_string(),
        "delete".to_string(),
        "publish".to_string(),
        "moderate".to_string(),
        "admin".to_string(),
    ];

    c.bench_function("jwt_complex_claims_creation", |b| {
        b.iter(|| {
            black_box(service.create_token(&claims).unwrap());
        });
    });

    let token = service.create_token(&claims).unwrap();
    c.bench_function("jwt_complex_claims_validation", |b| {
        b.iter(|| {
            black_box(service.validate_token(&token).unwrap());
        });
    });
}

fn bench_jwt_different_key_sizes(c: &mut Criterion) {
    let keys = vec![
        b"short-key",
        b"medium-length-key-for-testing-purposes",
        b"very-long-secret-key-that-should-work-fine-with-jwt-encryption-and-decryption-algorithms-and-should-be-sufficiently-long-for-security-purposes",
    ];

    for (i, key) in keys.iter().enumerate() {
        let service = JwtService::new(key).unwrap();
        let claims = Claims::new("benchmark-user", "benchmark-tenant");

        c.bench_function(&format!("jwt_creation_key_size_{}", i), |b| {
            b.iter(|| {
                black_box(service.create_token(&claims).unwrap());
            });
        });

        let token = service.create_token(&claims).unwrap();
        c.bench_function(&format!("jwt_validation_key_size_{}", i), |b| {
            b.iter(|| {
                black_box(service.validate_token(&token).unwrap());
            });
        });
    }
}

fn bench_jwt_concurrent_operations(c: &mut Criterion) {
    let service = JwtService::new(b"benchmark-secret-key-that-is-long-enough-for-performance-testing").unwrap();
    let claims = Claims::new("benchmark-user", "benchmark-tenant");

    c.bench_function("jwt_concurrent_creation", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..10).map(|_| {
                std::thread::spawn(move || {
                    let local_service = JwtService::new(b"benchmark-secret-key-that-is-long-enough-for-performance-testing").unwrap();
                    let local_claims = Claims::new("benchmark-user", "benchmark-tenant");
                    black_box(local_service.create_token(&local_claims).unwrap());
                })
            }).collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    let token = service.create_token(&claims).unwrap();
    c.bench_function("jwt_concurrent_validation", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..10).map(|_| {
                let token = token.clone();
                std::thread::spawn(move || {
                    let local_service = JwtService::new(b"benchmark-secret-key-that-is-long-enough-for-performance-testing").unwrap();
                    black_box(local_service.validate_token(&token).unwrap());
                })
            }).collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
}

fn bench_jwt_expiry_scenarios(c: &mut Criterion) {
    let service = JwtService::new(b"benchmark-secret-key-that-is-long-enough-for-performance-testing").unwrap();

    c.bench_function("jwt_creation_with_custom_expiry", |b| {
        b.iter(|| {
            let mut claims = Claims::new("benchmark-user", "benchmark-tenant");
            let future_time = chrono::Utc::now() + Duration::hours(1);
            claims.exp = future_time.timestamp() as usize;

            black_box(service.create_token(&claims).unwrap());
        });
    });

    c.bench_function("jwt_validation_expired_token", |b| {
        b.iter(|| {
            let mut claims = Claims::new("benchmark-user", "benchmark-tenant");
            claims.exp = (chrono::Utc::now() - Duration::hours(1)).timestamp() as usize;

            let token = service.create_token(&claims).unwrap();
            let _ = black_box(service.validate_token(&token));
        });
    });
}

fn bench_jwt_bulk_operations(c: &mut Criterion) {
    let service = JwtService::new(b"benchmark-secret-key-that-is-long-enough-for-performance-testing").unwrap();

    c.bench_function("jwt_bulk_creation_100", |b| {
        b.iter(|| {
            for i in 0..100 {
                let claims = Claims::new(&format!("user-{}", i), "benchmark-tenant");
                black_box(service.create_token(&claims).unwrap());
            }
        });
    });

    let tokens: Vec<String> = (0..100)
        .map(|i| {
            let claims = Claims::new(&format!("user-{}", i), "benchmark-tenant");
            service.create_token(&claims).unwrap()
        })
        .collect();

    c.bench_function("jwt_bulk_validation_100", |b| {
        b.iter(|| {
            for token in &tokens {
                black_box(service.validate_token(token).unwrap());
            }
        });
    });
}

criterion_group!(
    benches,
    bench_jwt_operations,
    bench_jwt_with_complex_claims,
    bench_jwt_different_key_sizes,
    bench_jwt_concurrent_operations,
    bench_jwt_expiry_scenarios,
    bench_jwt_bulk_operations
);
criterion_main!(benches);
