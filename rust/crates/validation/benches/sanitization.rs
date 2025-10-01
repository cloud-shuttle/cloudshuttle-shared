//! Benchmarks for sanitization functions
//!
//! This module provides comprehensive performance benchmarks for all sanitization functions.
//! To run these benchmarks, use: `cargo bench --bench sanitization`
//!
//! The benchmarks cover:
//! - HTML sanitization (basic and large content)
//! - Filename sanitization (basic and complex)
//! - SQL input sanitization
//! - URL sanitization
//! - Unicode normalization and trimming
//!
//! Note: These benchmarks use the Criterion.rs framework for accurate performance measurement.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cloudshuttle_validation::sanitization::*;

/// Benchmark HTML sanitization with various inputs
fn bench_sanitize_html(c: &mut Criterion) {
    let inputs = vec![
        "<script>alert('xss')</script>Hello World",
        "<b>Bold</b> and <i>italic</i> text",
        "Normal text without HTML",
        "<div><p>Nested <span>tags</span></p></div>",
        "<!-- comment --><img src='test.jpg' onerror='alert(1)'>",
    ];

    c.bench_function("sanitize_html", |b| {
        b.iter(|| {
            for input in &inputs {
                black_box(sanitize_html(input));
            }
        })
    });
}

/// Benchmark filename sanitization
fn bench_sanitize_filename(c: &mut Criterion) {
    let inputs = vec![
        "../../../etc/passwd",
        "file<name>.txt",
        "file with spaces and (special) chars!.txt",
        "normal_filename.txt",
        "file-with-dashes_and_underscores123.txt",
    ];

    c.bench_function("sanitize_filename", |b| {
        b.iter(|| {
            for input in &inputs {
                black_box(sanitize_filename(input));
            }
        })
    });
}

/// Benchmark SQL input sanitization with quote escaping
fn bench_sanitize_sql_input(c: &mut Criterion) {
    let inputs = vec![
        "normal input",
        "input with ' quotes",
        "input with \" double quotes",
        "input with 'single' and \"double\" quotes",
        "normal;text",
    ];

    c.bench_function("sanitize_sql_input", |b| {
        b.iter(|| {
            for input in &inputs {
                let _ = black_box(sanitize_sql_input(input, true));
            }
        })
    });
}

/// Benchmark whitespace trimming and normalization
fn bench_trim_and_normalize(c: &mut Criterion) {
    let inputs = vec![
        "  hello   world  ",
        "\t\thello\t\tworld\t\t",
        "\n\nhello\n\nworld\n\n",
        "   mixed   \t   whitespace   \n   ",
        "no_whitespace_needed",
    ];

    c.bench_function("trim_and_normalize", |b| {
        b.iter(|| {
            for input in &inputs {
                black_box(trim_and_normalize(input));
            }
        })
    });
}

/// Benchmark URL sanitization and normalization
fn bench_sanitize_url(c: &mut Criterion) {
    let inputs = vec![
        "example.com",
        "http://example.com",
        "https://example.com/path",
        "  spaced.example.com  ",
        "ftp://example.com",
    ];

    c.bench_function("sanitize_url", |b| {
        b.iter(|| {
            for input in &inputs {
                black_box(sanitize_url(input, "https"));
            }
        })
    });
}

/// Benchmark large HTML content sanitization
fn bench_large_html_sanitization(c: &mut Criterion) {
    // Create a large HTML string for benchmarking
    let large_html = format!(
        "<div>{}</div>",
        "<p>Lorem ipsum dolor sit amet</p>".repeat(1000)
    );

    c.bench_function("large_html_sanitization", |b| {
        b.iter(|| {
            black_box(sanitize_html(&large_html));
        })
    });
}

/// Benchmark complex filename sanitization with dangerous characters
fn bench_complex_filename_sanitization(c: &mut Criterion) {
    let complex_filenames = vec![
        "../../../etc/passwd/../../../root/.ssh/id_rsa",
        "<script>alert('xss')</script>.exe",
        "file:with:colons:everywhere.txt",
        "file|with|pipes.txt",
        "file*with*asterisks.txt",
    ];

    c.bench_function("complex_filename_sanitization", |b| {
        b.iter(|| {
            for filename in &complex_filenames {
                black_box(sanitize_filename(filename));
            }
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_sanitize_html,
             bench_sanitize_filename,
             bench_sanitize_sql_input,
             bench_trim_and_normalize,
             bench_sanitize_url,
             bench_large_html_sanitization,
             bench_complex_filename_sanitization
}
criterion_main!(benches);

