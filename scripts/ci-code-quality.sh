#!/bin/bash
# Run the complete CI Code Quality checks locally
# Replicates: Code Quality job from .github/workflows/ci.yml

set -e

echo "Running CI Code Quality checks locally..."
echo "This matches the exact commands used in CI Code Quality job"
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Step 1: Check formatting (matches CI line 154)
print_step "Check formatting"
cargo fmt --all -- --check
print_success "Formatting check passed"

echo ""

# Step 2: Run Clippy for ustar-parser (matches CI line 157)
print_step "Run Clippy for ustar-parser"
cargo clippy --all-targets --all-features -p ustar-parser -- -D warnings
print_success "Clippy check passed for ustar-parser"

echo ""

# Step 3: Run Clippy for ustar-tools (matches CI line 160)
print_step "Run Clippy for ustar-tools"
cargo clippy --all-targets --all-features -p ustar-tools -- -D warnings
print_success "Clippy check passed for ustar-tools"

echo ""

# Step 4: Check documentation for ustar-parser (matches CI line 163)
print_step "Check documentation for ustar-parser"
cargo doc --no-deps --document-private-items -p ustar-parser >/dev/null 2>&1
print_success "Documentation check passed for ustar-parser"

echo ""

# Step 5: Check documentation for ustar-tools (matches CI line 166)
print_step "Check documentation for ustar-tools"
cargo doc --no-deps --document-private-items -p ustar-tools >/dev/null 2>&1
print_success "Documentation check passed for ustar-tools"

echo ""
print_success "All CI Code Quality checks passed! 🎉"
echo "This matches: CI Code Quality job"