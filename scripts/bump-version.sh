#!/usr/bin/env bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}ℹ${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Function to show usage
usage() {
    cat << EOF
Usage: $0 [VERSION]

Bump version in Cargo.toml and create a release commit.

Arguments:
  VERSION       New version number (e.g., 0.3.0)

Examples:
  $0 0.3.0      # Bump to version 0.3.0
  $0 1.0.0      # Bump to version 1.0.0

The script will:
1. Update version in Cargo.toml
2. Run tests to ensure everything works
3. Create a commit with the version bump
4. Create and push a git tag
5. The release workflow will automatically:
   - Build binaries for all platforms
   - Publish to crates.io
   - Update Homebrew formula
   - Create GitHub release

EOF
    exit 1
}

# Check if version is provided
if [ $# -ne 1 ]; then
    print_error "Version number required"
    usage
fi

NEW_VERSION="$1"

# Validate version format (semantic versioning)
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_error "Invalid version format: $NEW_VERSION"
    echo "Version must be in format: MAJOR.MINOR.PATCH (e.g., 0.3.0)"
    exit 1
fi

# Check if we're in the project root
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Run this script from the project root."
    exit 1
fi

# Get current version
CURRENT_VERSION=$(grep -m 1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
print_info "Current version: $CURRENT_VERSION"
print_info "New version: $NEW_VERSION"

# Confirm with user
read -p "Continue with version bump? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_warn "Version bump cancelled"
    exit 0
fi

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    print_warn "You have uncommitted changes:"
    git status --short
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warn "Version bump cancelled"
        exit 0
    fi
fi

# Update version in Cargo.toml
print_info "Updating Cargo.toml..."
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
print_info "Updating Cargo.lock..."
cargo update --workspace

# Update local Homebrew formula if it exists
if [ -f "Formula/sobriquet.rb" ]; then
    print_info "Updating Formula/sobriquet.rb (version placeholder)..."
    # We'll update the URL and version, but leave SHA256 as a placeholder
    # The release workflow will calculate the actual SHA256 and update homebrew-tap
    TAG="v$NEW_VERSION"
    sed -i.bak "s|archive/refs/tags/v[0-9]*\.[0-9]*\.[0-9]*|archive/refs/tags/$TAG|g" Formula/sobriquet.rb
    rm Formula/sobriquet.rb.bak
    
    # Add note that SHA256 will be updated by CI
    print_info "Note: SHA256 will be calculated and updated by CI in homebrew-tap"
fi

# Run tests
print_info "Running tests..."
if ! cargo test --quiet; then
    print_error "Tests failed. Reverting changes..."
    git checkout Cargo.toml Cargo.lock Formula/sobriquet.rb 2>/dev/null
    exit 1
fi

# Run clippy
print_info "Running clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings 2>/dev/null; then
    print_warn "Clippy warnings found, but continuing..."
fi

# Create commit
print_info "Creating commit..."
FILES_TO_ADD="Cargo.toml Cargo.lock"
if [ -f "Formula/sobriquet.rb" ]; then
    FILES_TO_ADD="$FILES_TO_ADD Formula/sobriquet.rb"
fi
git add $FILES_TO_ADD
git commit -m "chore: release version $NEW_VERSION"

# Create tag
TAG="v$NEW_VERSION"
print_info "Creating tag: $TAG"
git tag -a "$TAG" -m "Release $NEW_VERSION"

# Show what will happen
cat << EOF

${GREEN}✓${NC} Version bumped successfully!

Next steps:
1. Review the commit and tag:
   git show HEAD
   git show $TAG

2. Push to trigger release automation:
   git push origin main --follow-tags

This will automatically:
  • Build binaries for Linux (x86_64, aarch64, armv7) and macOS (x86_64, aarch64)
  • Create GitHub release with binaries
  • Publish to crates.io
  • Update Homebrew formula in homebrew-tap

EOF

# Ask if user wants to push now
read -p "Push to origin now? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_info "Pushing to origin..."
    git push origin main --follow-tags
    print_info "Release workflow triggered! Check: https://github.com/benjaminch/sobriquet/actions"
else
    print_warn "Remember to push with: git push origin main --follow-tags"
fi
