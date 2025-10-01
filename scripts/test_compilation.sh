#!/bin/bash

# Quick compilation test for CloudShuttle
# Run this when terminal environment is fixed

echo "🧪 CloudShuttle Compilation Test"
echo "==============================="

# Try to find cargo
if command -v cargo >/dev/null 2>&1; then
    CARGO_PATH=$(which cargo)
    echo "✅ Cargo found: $CARGO_PATH"
    echo "   Version: $(cargo --version 2>/dev/null || echo 'version check failed')"

    echo ""
    echo "🔨 Testing compilation..."

    # Test compilation
    if cargo check --workspace --quiet 2>&1; then
        echo "✅ Workspace compilation: PASSED"

        # Test tests
        if cargo test --workspace --lib --quiet 2>&1; then
            echo "✅ Unit tests: PASSED"
            echo ""
            echo "🎉 CloudShuttle is ready for release!"
            echo "   Run: ./scripts/create_release.sh"
        else
            echo "❌ Unit tests: FAILED"
            exit 1
        fi
    else
        echo "❌ Workspace compilation: FAILED"
        echo ""
        echo "🔍 Compilation errors:"
        cargo check --workspace 2>&1 | head -20
        exit 1
    fi
else
    echo "❌ Cargo not found in PATH"
    echo "   Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
