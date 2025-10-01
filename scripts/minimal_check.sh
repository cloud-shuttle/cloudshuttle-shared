#!/bin/sh

# Minimal check script using sh instead of bash
echo "🔍 Minimal Environment Check"
echo "==========================="
echo "Current shell: $0"
echo "User: $(whoami)"
echo "Working directory: $(pwd)"
echo "Home: $HOME"
echo ""

# Check for basic commands
echo "Checking basic commands:"
for cmd in ls cat grep find; do
    if command -v $cmd >/dev/null 2>&1; then
        echo "✅ $cmd: $(which $cmd)"
    else
        echo "❌ $cmd: not found"
    fi
done

echo ""
echo "Checking for Rust/Cargo:"

# Check common Rust installation locations
rust_locations="/usr/local/cargo/bin /opt/homebrew/bin /Users/$USER/.cargo/bin"

for location in $rust_locations; do
    if [ -x "$location/cargo" ]; then
        echo "✅ Cargo found at: $location/cargo"
        export PATH="$location:$PATH"
        break
    fi
done

if command -v cargo >/dev/null 2>&1; then
    echo "✅ Cargo available: $(cargo --version 2>&1 || echo 'version check failed')"
else
    echo "❌ Cargo not found in common locations"
    echo "Please install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi

echo ""
echo "Environment check complete."
