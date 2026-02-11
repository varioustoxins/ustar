#!/bin/bash
# Simple cargo release script
set -e

# Read current version from workspace Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

echo "Releasing version $VERSION..."

# Publish in order
cargo publish -p ustar-test-utils
cargo publish -p ustar-parser  
cargo publish -p ustar-tools

# Tag release
git add .
git commit -m "Release $VERSION"
git tag "v$VERSION"

echo "Released $VERSION successfully!"