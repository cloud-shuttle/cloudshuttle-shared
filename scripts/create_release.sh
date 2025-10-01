#!/bin/bash

# CloudShuttle Release Creation Script
# Creates a new GitHub release using gh CLI

set -e

echo "🚀 CloudShuttle Release Creation Script"
echo "======================================"
echo ""

# Check if gh CLI is available
if ! command -v gh >/dev/null 2>&1; then
    echo "❌ GitHub CLI (gh) not found"
    echo ""
    echo "Install GitHub CLI:"
    echo "  macOS: brew install gh"
    echo "  Ubuntu: curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg && echo \"deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main\" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null && sudo apt update && sudo apt install gh"
    echo "  Other: https://cli.github.com/"
    echo ""
    echo "Then authenticate: gh auth login"
    exit 1
fi

echo "✅ GitHub CLI found: $(gh --version | head -1)"
echo ""

# Check if we're in a git repository
if ! git rev-parse --git-dir >/dev/null 2>&1; then
    echo "❌ Not in a git repository"
    exit 1
fi

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "⚠️  Warning: You have uncommitted changes"
    echo "   Commit or stash changes before creating a release"
    echo ""
    git status --short
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [ -z "$CURRENT_VERSION" ]; then
    echo "❌ Could not determine current version from Cargo.toml"
    exit 1
fi

echo "📦 Current version: $CURRENT_VERSION"
echo ""

# Get latest tag
LATEST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "none")
if [ "$LATEST_TAG" = "none" ]; then
    echo "📝 No previous tags found - this will be the first release"
else
    echo "🏷️  Latest tag: $LATEST_TAG"
fi

echo ""

# Generate changelog since last release
if [ "$LATEST_TAG" != "none" ]; then
    echo "📋 Generating changelog since $LATEST_TAG..."
    echo ""

    # Generate changelog
    git log --pretty=format:"* %s (%h)" --no-merges "$LATEST_TAG"..HEAD | cat

    echo ""
    echo "---"
    echo ""
else
    echo "📋 This is the first release - full changelog:"
    echo ""
    git log --pretty=format:"* %s (%h)" --no-merges | head -20
    echo "... (showing first 20 commits)"
    echo ""
fi

# Ask for release type
echo "🎯 What type of release?"
echo "  1) Patch (bug fixes) - $CURRENT_VERSION"
echo "  2) Minor (new features) - increment minor version"
echo "  3) Major (breaking changes) - increment major version"
echo "  4) Pre-release (alpha/beta/rc)"
echo "  5) Custom version"
echo ""

read -p "Choose release type (1-5): " -r
echo ""

case $REPLY in
    1)
        RELEASE_VERSION="$CURRENT_VERSION"
        RELEASE_TYPE="patch"
        ;;
    2)
        # Increment minor version
        IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
        ((VERSION_PARTS[1]++))
        RELEASE_VERSION="${VERSION_PARTS[0]}.${VERSION_PARTS[1]}.0"
        RELEASE_TYPE="minor"
        ;;
    3)
        # Increment major version
        IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
        ((VERSION_PARTS[0]++))
        RELEASE_VERSION="${VERSION_PARTS[0]}.0.0"
        RELEASE_TYPE="major"
        ;;
    4)
        read -p "Enter pre-release version (e.g., $CURRENT_VERSION-alpha.1): " RELEASE_VERSION
        RELEASE_TYPE="pre-release"
        ;;
    5)
        read -p "Enter custom version: " RELEASE_VERSION
        RELEASE_TYPE="custom"
        ;;
    *)
        echo "❌ Invalid choice"
        exit 1
        ;;
esac

echo "📦 Release version: $RELEASE_VERSION"
echo "🏷️  Release type: $RELEASE_TYPE"
echo ""

# Ask for release notes
echo "📝 Enter release notes (press Enter twice when done):"
echo ""

RELEASE_NOTES=""
while IFS= read -r line; do
    [ -z "$line" ] && break
    RELEASE_NOTES="${RELEASE_NOTES}${line}\n"
done

if [ -z "$RELEASE_NOTES" ]; then
    RELEASE_NOTES="Release $RELEASE_VERSION

## Changes
$(git log --pretty=format:"* %s" --no-merges -10)"
fi

echo ""
echo "📋 Release Summary:"
echo "=================="
echo "Version: $RELEASE_VERSION"
echo "Type: $RELEASE_TYPE"
echo ""
echo "Release Notes:"
echo "$RELEASE_NOTES"
echo ""

read -p "Create this release? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Release cancelled"
    exit 0
fi

# Create and push tag
echo "🏷️  Creating tag v$RELEASE_VERSION..."
git tag -a "v$RELEASE_VERSION" -m "Release $RELEASE_VERSION"
git push origin "v$RELEASE_VERSION"

echo ""
echo "🚀 Creating GitHub release..."

# Create GitHub release
gh release create "v$RELEASE_VERSION" \
    --title "Release $RELEASE_VERSION" \
    --notes "$RELEASE_NOTES" \
    --latest="$([ "$RELEASE_TYPE" = "major" ] && echo "true" || echo "false")"

echo ""
echo "✅ Release $RELEASE_VERSION created successfully!"
echo ""
echo "🎉 Next steps:"
echo "  • Monitor CI/CD pipelines"
echo "  • Update downstream dependencies if needed"
echo "  • Announce release to team"
echo ""
echo "📦 Release URL: $(gh release view "v$RELEASE_VERSION" --json url -q .url 2>/dev/null || echo 'Check GitHub repository')" 
