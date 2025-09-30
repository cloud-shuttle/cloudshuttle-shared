#!/bin/bash

# CloudShuttle Shared Libraries - Setup Script
# This script sets up the development environment

set -e

echo "🚀 Setting up CloudShuttle Shared Libraries"
echo "==========================================="

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

# Setup Rust environment
setup_rust() {
    print_status "Setting up Rust environment..."

    if ! command_exists cargo; then
        print_status "Installing Rust..."

        # Install Rust using rustup
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

        # Source cargo environment
        source ~/.cargo/env

        print_success "Rust installed"
    else
        print_success "Rust already installed"
    fi

    # Install additional tools
    print_status "Installing Rust tools..."
    cargo install cargo-edit
    cargo install cargo-watch
    cargo install cargo-tarpaulin
    cargo install cargo-audit
    cargo install cargo-deny

    print_success "Rust tools installed"
}

# Setup Node.js environment
setup_nodejs() {
    print_status "Setting up Node.js environment..."

    if ! command_exists node || ! command_exists npm; then
        print_status "Installing Node.js..."

        # Check if on macOS
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # Install using Homebrew
            if command_exists brew; then
                brew install node
            else
                print_error "Homebrew not found. Please install Node.js manually."
                exit 1
            fi
        else
            # Install using NodeSource
            curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
            sudo apt-get install -y nodejs
        fi

        print_success "Node.js installed"
    else
        print_success "Node.js already installed"
    fi

    # Install global tools
    print_status "Installing global npm tools..."
    npm install -g typescript
    npm install -g eslint
    npm install -g prettier
    npm install -g lerna

    print_success "Global npm tools installed"
}

# Setup TypeScript dependencies
setup_typescript_deps() {
    print_status "Setting up TypeScript dependencies..."

    cd typescript

    # Install root dependencies
    npm ci

    # Install dependencies for each library
    LIBRARIES=(
        "components"
        "hooks"
        "types"
        "utils"
        "api"
        "stores"
    )

    for lib in "${LIBRARIES[@]}"; do
        if [ -d "$lib" ]; then
            print_status "Installing dependencies for $lib..."
            cd "$lib"
            npm ci
            cd ..
        fi
    done

    cd ..
    print_success "TypeScript dependencies installed"
}

# Setup pre-commit hooks
setup_git_hooks() {
    print_status "Setting up git hooks..."

    # Create pre-commit hook
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash

# CloudShuttle pre-commit hook

echo "Running pre-commit checks..."

# Run tests (quick check)
if command -v cargo &> /dev/null; then
    echo "Running Rust tests..."
    cargo test --quiet
fi

if command -v npm &> /dev/null && [ -d "typescript" ]; then
    cd typescript
    echo "Running TypeScript tests..."
    npm run test:quick 2>/dev/null || npm test -- --passWithNoTests --watchAll=false
    cd ..
fi

echo "Pre-commit checks completed"
EOF

    chmod +x .git/hooks/pre-commit
    print_success "Git hooks installed"
}

# Main setup process
main() {
    local setup_rust=true
    local setup_nodejs=true
    local setup_deps=true
    local setup_hooks=true

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --rust-only)
                setup_nodejs=false
                setup_deps=false
                shift
                ;;
            --typescript-only)
                setup_rust=false
                shift
                ;;
            --no-deps)
                setup_deps=false
                shift
                ;;
            --no-hooks)
                setup_hooks=false
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --rust-only        Setup only Rust environment"
                echo "  --typescript-only  Setup only TypeScript environment"
                echo "  --no-deps          Skip installing dependencies"
                echo "  --no-hooks         Skip setting up git hooks"
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

    print_status "Starting setup process..."

    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ] || [ ! -d "typescript" ]; then
        print_error "Please run this script from the repository root directory."
        exit 1
    fi

    # Setup Rust
    if [ "$setup_rust" = true ]; then
        setup_rust
    fi

    # Setup Node.js
    if [ "$setup_nodejs" = true ]; then
        setup_nodejs
    fi

    # Setup dependencies
    if [ "$setup_deps" = true ]; then
        setup_typescript_deps
    fi

    # Setup git hooks
    if [ "$setup_hooks" = true ]; then
        setup_git_hooks
    fi

    print_success "🎉 Setup completed successfully!"
    echo ""
    print_status "Next steps:"
    echo "  1. Run './scripts/test-all.sh' to verify everything works"
    echo "  2. Run './scripts/build-all.sh' to build all libraries"
    echo "  3. Start developing!"
    echo ""
    print_status "Useful commands:"
    echo "  ./scripts/test-all.sh          - Run all tests"
    echo "  ./scripts/build-all.sh         - Build all libraries"
    echo "  ./scripts/release.sh --help    - Release management"
}
