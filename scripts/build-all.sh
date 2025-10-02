#!/bin/bash

# CloudShuttle Shared Libraries - Build All Script
# This script builds all Rust and TypeScript libraries

set -e

echo "🔨 Building CloudShuttle Shared Libraries"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status messages
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Build Rust libraries
build_rust() {
    print_status "Building Rust libraries..."

    if ! command_exists cargo; then
        print_error "Cargo is not installed. Please install Rust and Cargo."
        exit 1
    fi

    # Check if we're in the right directory
    if [ ! -f "rust/crates/error-handling/Cargo.toml" ]; then
        print_error "Please run this script from the repository root directory."
        exit 1
    fi

    # Build all Rust libraries
    RUST_LIBRARIES=(
        "error-handling"
        "database"
        "auth"
        "observability"
        "config"
        "api"
        "validation"
        "crypto"
    )

    for lib in "${RUST_LIBRARIES[@]}"; do
        print_status "Building rust/crates/${lib}..."
        cd "rust/crates/${lib}"

        # Check Cargo.toml exists
        if [ ! -f "Cargo.toml" ]; then
            print_error "Cargo.toml not found in rust/crates/${lib}"
            cd ../../..
            continue
        fi

        # Build with all features
        if cargo build --all-features; then
            print_success "Built rust/crates/${lib}"
        else
            print_error "Failed to build rust/crates/${lib}"
            cd ../../..
            exit 1
        fi

        # Run clippy if available
        if command_exists cargo-clippy; then
            if cargo clippy --all-targets --all-features -- -D warnings; then
                print_success "Clippy passed for rust/crates/${lib}"
            else
                print_warning "Clippy warnings in rust/crates/${lib}"
            fi
        fi

        cd ../../..
    done

    print_success "All Rust libraries built successfully"
}

# Build TypeScript libraries
build_typescript() {
    print_status "Building TypeScript libraries..."

    if ! command_exists node; then
        print_error "Node.js is not installed. Please install Node.js."
        exit 1
    fi

    if ! command_exists npm; then
        print_error "npm is not installed. Please install npm."
        exit 1
    fi

    cd typescript

    # Check if package.json exists
    if [ ! -f "package.json" ]; then
        print_error "package.json not found in typescript directory"
        cd ..
        return 1
    fi

    # Install dependencies if node_modules doesn't exist
    if [ ! -d "node_modules" ]; then
        print_status "Installing dependencies..."
        if ! npm ci; then
            print_error "Failed to install dependencies"
            cd ..
            exit 1
        fi
    fi

    # Build TypeScript libraries
    TYPESCRIPT_LIBRARIES=(
        "components"
        "hooks"
        "types"
        "utils"
        "api"
        "stores"
    )

    for lib in "${TYPESCRIPT_LIBRARIES[@]}"; do
        print_status "Building typescript/${lib}..."

        if [ ! -d "${lib}" ]; then
            print_warning "Directory typescript/${lib} not found, skipping"
            continue
        fi

        cd "${lib}"

        # Check package.json exists
        if [ ! -f "package.json" ]; then
            print_warning "package.json not found in typescript/${lib}, skipping"
            cd ..
            continue
        fi

        # Install dependencies
        if ! npm ci; then
            print_error "Failed to install dependencies for typescript/${lib}"
            cd ..
            continue
        fi

        # Run linting
        if npm run lint 2>/dev/null; then
            print_success "Linting passed for typescript/${lib}"
        else
            print_warning "Linting failed for typescript/${lib}"
        fi

        # Build
        if npm run build; then
            print_success "Built typescript/${lib}"
        else
            print_error "Failed to build typescript/${lib}"
            cd ..
            exit 1
        fi

        cd ..
    done

    cd ..
    print_success "All TypeScript libraries built successfully"
}

# Main build process
main() {
    local build_rust=true
    local build_typescript=true

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --rust-only)
                build_typescript=false
                shift
                ;;
            --typescript-only)
                build_rust=false
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --rust-only        Build only Rust libraries"
                echo "  --typescript-only  Build only TypeScript libraries"
                echo "  --help             Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    print_status "Starting build process..."

    # Build Rust libraries
    if [ "$build_rust" = true ]; then
        if build_rust; then
            print_success "Rust build completed"
        else
            print_error "Rust build failed"
            exit 1
        fi
    fi

    # Build TypeScript libraries
    if [ "$build_typescript" = true ]; then
        if build_typescript; then
            print_success "TypeScript build completed"
        else
            print_error "TypeScript build failed"
            exit 1
        fi
    fi

    print_success "🎉 All builds completed successfully!"
}

# Run main function
main "$@"
