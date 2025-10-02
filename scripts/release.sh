#!/bin/bash

# CloudShuttle Shared Libraries - Release Script
# This script handles the release process for all libraries

set -e

echo "🚀 CloudShuttle Shared Libraries Release"
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

# Validate version format
validate_version() {
    local version=$1
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
        print_error "Invalid version format: $version"
        print_error "Expected format: x.y.z or x.y.z-prerelease"
        exit 1
    fi
}

# Check if working directory is clean
check_clean_workspace() {
    if [ -n "$(git status --porcelain)" ]; then
        print_error "Working directory is not clean. Please commit or stash changes."
        git status
        exit 1
    fi
}

# Check if on main branch
check_main_branch() {
    local current_branch=$(git branch --show-current)
    if [ "$current_branch" != "main" ]; then
        print_warning "Not on main branch (current: $current_branch)"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# Update version in Cargo.toml files
update_rust_versions() {
    local version=$1
    print_status "Updating Rust library versions to $version..."

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
        local cargo_file="rust/crates/${lib}/Cargo.toml"
        if [ -f "$cargo_file" ]; then
            # Update version
            sed -i.bak "s/^version = \".*\"/version = \"$version\"/" "$cargo_file"
            rm "${cargo_file}.bak"
            print_success "Updated $cargo_file"
        else
            print_warning "Cargo.toml not found: $cargo_file"
        fi
    done
}

# Update version in package.json files
update_typescript_versions() {
    local version=$1
    print_status "Updating TypeScript library versions to $version..."

    if ! command_exists npm; then
        print_error "npm is required for TypeScript version updates"
        return 1
    fi

    TYPESCRIPT_LIBRARIES=(
        "components"
        "hooks"
        "types"
        "utils"
        "api"
        "stores"
    )

    for lib in "${TYPESCRIPT_LIBRARIES[@]}"; do
        local package_file="typescript/${lib}/package.json"
        if [ -f "$package_file" ]; then
            # Update version using npm
            cd "typescript/${lib}"
            npm version "$version" --no-git-tag-version
            cd ../..
            print_success "Updated $package_file"
        else
            print_warning "package.json not found: $package_file"
        fi
    done
}

# Update root package.json if it exists
update_root_package_json() {
    local version=$1
    if [ -f "typescript/package.json" ]; then
        cd typescript
        npm version "$version" --no-git-tag-version
        cd ..
        print_success "Updated root typescript/package.json"
    fi
}

# Run tests before release
run_tests() {
    print_status "Running tests before release..."

    if ! ./scripts/test-all.sh; then
        print_error "Tests failed. Aborting release."
        exit 1
    fi

    print_success "All tests passed"
}

# Build all libraries
build_libraries() {
    print_status "Building all libraries..."

    if ! ./scripts/build-all.sh --rust-only; then
        print_error "Build failed. Aborting release."
        exit 1
    fi

    print_success "All libraries built successfully"
}

# Create git commit and tag
create_git_release() {
    local version=$1
    local tag="v$version"

    print_status "Creating git release..."

    # Commit version changes
    git add .
    git commit -m "Release $version"

    # Create annotated tag
    git tag -a "$tag" -m "Release $version"

    print_success "Created git tag: $tag"
}

# Publish Rust crates
publish_rust_crates() {
    print_status "Publishing Rust crates..."

    if [ -z "$CRATES_IO_TOKEN" ]; then
        print_error "CRATES_IO_TOKEN environment variable not set"
        print_error "Please set your crates.io API token"
        exit 1
    fi

    # Login to crates.io
    cargo login "$CRATES_IO_TOKEN"

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
        print_status "Publishing rust/crates/${lib}..."

        cd "rust/crates/${lib}"

        # Wait a bit between publishes to avoid rate limiting
        sleep 30

        if cargo publish; then
            print_success "Published rust/${lib}"
        else
            print_error "Failed to publish rust/${lib}"
            cd ../..
            exit 1
        fi

        cd ../..
    done

    print_success "All Rust crates published"
}

# Publish TypeScript packages
publish_typescript_packages() {
    print_status "Publishing TypeScript packages..."

    if [ -z "$NPM_TOKEN" ]; then
        print_error "NPM_TOKEN environment variable not set"
        print_error "Please set your npm API token"
        exit 1
    fi

    # Set npm token
    echo "//registry.npmjs.org/:_authToken=$NPM_TOKEN" > ~/.npmrc

    cd typescript

    TYPESCRIPT_LIBRARIES=(
        "components"
        "hooks"
        "types"
        "utils"
        "api"
        "stores"
    )

    for lib in "${TYPESCRIPT_LIBRARIES[@]}"; do
        print_status "Publishing typescript/${lib}..."

        cd "${lib}"

        # Wait a bit between publishes
        sleep 10

        if npm publish; then
            print_success "Published typescript/${lib}"
        else
            print_error "Failed to publish typescript/${lib}"
            cd ../..
            exit 1
        fi

        cd ..
    done

    cd ..
    print_success "All TypeScript packages published"
}

# Push to remote repository
push_to_remote() {
    local version=$1
    local tag="v$version"

    print_status "Pushing to remote repository..."

    git push origin main
    git push origin "$tag"

    print_success "Pushed to remote repository"
}

# Main release process
main() {
    local version=""
    local skip_tests=false
    local skip_build=false
    local dry_run=false

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version=*)
                version="${1#*=}"
                shift
                ;;
            --skip-tests)
                skip_tests=true
                shift
                ;;
            --skip-build)
                skip_build=true
                shift
                ;;
            --dry-run)
                dry_run=true
                shift
                ;;
            --help)
                echo "Usage: $0 --version=x.y.z [OPTIONS]"
                echo ""
                echo "Required:"
                echo "  --version=x.y.z    Release version (e.g., 1.2.3)"
                echo ""
                echo "Options:"
                echo "  --skip-tests       Skip running tests"
                echo "  --skip-build       Skip building libraries"
                echo "  --dry-run          Show what would be done without doing it"
                echo "  --help             Show this help message"
                echo ""
                echo "Environment Variables:"
                echo "  CRATES_IO_TOKEN    Required for Rust crate publishing"
                echo "  NPM_TOKEN          Required for npm package publishing"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    # Validate version
    if [ -z "$version" ]; then
        print_error "Version is required. Use --version=x.y.z"
        exit 1
    fi

    validate_version "$version"

    if [ "$dry_run" = true ]; then
        print_status "DRY RUN MODE - No changes will be made"
    fi

    print_status "Starting release process for version $version..."

    # Pre-flight checks
    check_clean_workspace
    check_main_branch

    # Run tests
    if [ "$skip_tests" = false ]; then
        run_tests
    else
        print_warning "Skipping tests"
    fi

    # Build libraries
    if [ "$skip_build" = false ]; then
        build_libraries
    else
        print_warning "Skipping build"
    fi

    # Skip TypeScript builds for now (focus on Rust release)
    print_warning "Skipping TypeScript builds for this release"

    # Update versions (Rust only for this release)
    if [ "$dry_run" = false ]; then
        update_rust_versions "$version"
        # update_typescript_versions "$version"
        # update_root_package_json "$version"
        print_warning "Skipping TypeScript version updates"
    else
        print_status "Would update Rust versions to $version"
        print_status "Would skip TypeScript version updates"
    fi

    # Create git release
    if [ "$dry_run" = false ]; then
        create_git_release "$version"
    else
        print_status "Would create git tag v$version"
    fi

    # Publish Rust crates
    if [ "$dry_run" = false ]; then
        publish_rust_crates
    else
        print_status "Would publish Rust crates to crates.io"
    fi

    # Skip TypeScript publishing for now (focus on Rust release)
    if [ "$dry_run" = false ]; then
        print_warning "Skipping TypeScript package publishing"
    else
        print_status "Would skip TypeScript package publishing"
    fi

    # Push to remote
    if [ "$dry_run" = false ]; then
        push_to_remote "$version"
    else
        print_status "Would push to remote repository"
    fi

    print_success "🎉 Release $version completed successfully!"

    if [ "$dry_run" = false ]; then
        print_status "Next steps:"
        echo "  - Monitor CI/CD pipelines"
        echo "  - Check package registries for publication"
        echo "  - Update downstream services to use new versions"
        echo "  - Create release notes on GitHub"
    fi
}

# Run main function
main "$@"
