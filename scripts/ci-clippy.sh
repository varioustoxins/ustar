#!/bin/bash
# Run clippy with the same configuration as CI
# This helps catch issues locally before pushing

set -e

echo "Running Clippy with CI configuration..."
echo "This matches the exact commands used in .github/workflows/ci.yml"
echo ""

echo "=== Checking ustar-parser ==="
cargo clippy --all-targets --all-features -p ustar-parser -- -D warnings

echo ""
echo "=== Checking ustar-tools ==="
cargo clippy --all-targets --all-features -p ustar-tools -- -D warnings

echo ""
echo "=== All clippy checks passed! ==="