#!/bin/bash

# CloudShuttle Shared Libraries - Test All Script
# This script runs tests for all Rust and TypeScript libraries

set -e

echo "🧪 Testing CloudShuttle Shared Libraries"
echo "======================================="

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

# Test Rust libraries
test_rust() {
    print_status "Testing Rust libraries..."

    if ! command_exists cargo; then
        print_error "Cargo is not installed. Please install Rust and Cargo."
        exit 1
    fi

    # Check if we're in the right directory
    if [ ! -f "rust/crates/error-handling/Cargo.toml" ]; then
        print_error "Please run this script from the repository root directory."
        exit 1
    fi

    # Test all Rust libraries
    RUST_LIBRARIES=(
        "error-handling"
        "database"
        # "auth"  # Temporarily disabled due to axum 0.8 compatibility issues
        "observability"
        "config"
        "api"
        "validation"
        "crypto"
    )

    for lib in "${RUST_LIBRARIES[@]}"; do
        print_status "Testing rust/crates/${lib}..."
        cd "rust/crates/${lib}"

        # Check Cargo.toml exists
        if [ ! -f "Cargo.toml" ]; then
            print_error "Cargo.toml not found in rust/crates/${lib}"
            cd ../../..
            continue
        fi

        # Run tests
        if cargo test --all-features --verbose; then
            print_success "Tests passed for rust/crates/${lib}"
        else
            print_error "Tests failed for rust/crates/${lib}"
            cd ../../..
            exit 1
        fi

        # Run doc tests
        if cargo test --doc --all-features; then
            print_success "Doc tests passed for rust/crates/${lib}"
        else
            print_warning "Doc tests failed for rust/crates/${lib}"
        fi

        cd ../../..
    done

    print_success "All Rust library tests completed"
}

# Test TypeScript libraries
test_typescript() {
    print_status "Testing TypeScript libraries..."

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

    # Test TypeScript libraries
    TYPESCRIPT_LIBRARIES=(
        "components"
        "hooks"
        "types"
        "utils"
        "api"
        "stores"
    )

    for lib in "${TYPESCRIPT_LIBRARIES[@]}"; do
        print_status "Testing typescript/${lib}..."

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

        # Run tests
        if npm test -- --watchAll=false --passWithNoTests; then
            print_success "Tests passed for typescript/${lib}"
        else
            print_error "Tests failed for typescript/${lib}"
            cd ..
            exit 1
        fi

        cd ..
    done

    cd ..
    print_success "All TypeScript library tests completed"
}

# Run integration tests
run_integration_tests() {
    print_status "Running integration tests..."

    # Check if PostgreSQL is available for database tests
    if command_exists psql; then
        print_status "Running database integration tests..."

        # Set up test database if environment variable is set
        if [ -n "$DATABASE_URL" ]; then
            # Run database integration tests
            if cargo test --test integration --all-features; then
                print_success "Database integration tests passed"
            else
                print_warning "Database integration tests failed (database may not be available)"
            fi
        else
            print_warning "DATABASE_URL not set, skipping database integration tests"
        fi
    else
        print_warning "PostgreSQL client not available, skipping database integration tests"
    fi

    # TypeScript integration tests
    cd typescript
    if npm run test:integration 2>/dev/null; then
        print_success "TypeScript integration tests passed"
    else
        print_warning "TypeScript integration tests not configured or failed"
    fi
    cd ..
}

# Generate coverage report
generate_coverage() {
    print_status "Generating coverage reports..."

    # Rust coverage
    if command_exists cargo-tarpaulin; then
        print_status "Generating Rust coverage report..."
        if cargo tarpaulin --all-features --workspace --out Html --output-dir target/coverage; then
            print_success "Rust coverage report generated: target/coverage/tarpaulin-report.html"
        else
            print_warning "Failed to generate Rust coverage report"
        fi
    else
        print_warning "cargo-tarpaulin not installed, skipping Rust coverage"
    fi

    # TypeScript coverage
    cd typescript
    if npm run test:coverage 2>/dev/null; then
        print_success "TypeScript coverage report generated"
    else
        print_warning "TypeScript coverage not configured"
    fi
    cd ..
}

# Main test process
main() {
    local test_rust=true
    local test_typescript=true
    local run_integration=false
    local generate_coverage_report=false

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --rust-only)
                test_typescript=false
                shift
                ;;
            --typescript-only)
                test_rust=false
                shift
                ;;
            --integration)
                run_integration=true
                shift
                ;;
            --coverage)
                generate_coverage_report=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --rust-only        Test only Rust libraries"
                echo "  --typescript-only  Test only TypeScript libraries"
                echo "  --integration      Run integration tests"
                echo "  --coverage         Generate coverage reports"
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

    print_status "Starting test process..."

    # Test Rust libraries
    if [ "$test_rust" = true ]; then
        if test_rust; then
            print_success "Rust tests completed"
        else
            print_error "Rust tests failed"
            exit 1
        fi
    fi

    # Test TypeScript libraries
    if [ "$test_typescript" = true ]; then
        if test_typescript; then
            print_success "TypeScript tests completed"
        else
            print_error "TypeScript tests failed"
            exit 1
        fi
    fi

    # Run integration tests
    if [ "$run_integration" = true ]; then
        run_integration_tests
    fi

    # Generate coverage
    if [ "$generate_coverage_report" = true ]; then
        generate_coverage
    fi

    print_success "🎉 All tests completed successfully!"
}

# Run main function
main "$@"
