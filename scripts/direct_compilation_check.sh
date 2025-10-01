#!/bin/bash

# Direct compilation check bypassing shell environment issues
# Execute with: /bin/bash scripts/direct_compilation_check.sh

# Clear any problematic environment variables
unset ZDOTDIR
unset ZSH
export SHELL=/bin/bash

# Set minimal PATH
export PATH="/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/local/cargo/bin:/opt/homebrew/bin:/Users/$USER/.cargo/bin"

echo "🔧 CloudShuttle Direct Compilation Check"
echo "========================================"
echo "Working directory: $(pwd)"
echo "Shell: $SHELL"
echo "PATH: $PATH"
echo ""

# Check for cargo
if command -v cargo >/dev/null 2>&1; then
    echo "✅ Cargo found at: $(which cargo)"
    cargo --version
else
    echo "❌ Cargo not found"
    echo "Searching for cargo..."

    find /usr -name cargo 2>/dev/null | head -5 || echo "No cargo found in /usr"
    find /opt -name cargo 2>/dev/null | head -5 || echo "No cargo found in /opt"
    find /Users -name cargo 2>/dev/null | head -5 || echo "No cargo found in /Users"

    exit 1
fi

echo ""
echo "🔨 Attempting compilation..."

# Try compilation
if cargo check --workspace --quiet 2>&1; then
    echo "✅ Compilation successful!"
else
    echo "❌ Compilation failed"
    cargo check --workspace 2>&1 | head -30
    exit 1
fi

echo ""
echo "🎉 CloudShuttle compilation verified!"
