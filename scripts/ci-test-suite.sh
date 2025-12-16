#!/bin/bash
# Run the CI Test Suite locally
# Replicates all Test Suite matrix combinations

set -e

# Default values
PACKAGE=""
FEATURES=""
OS="ubuntu-latest"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --package)
            PACKAGE="$2"
            shift 2
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        --os)
            OS="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --package PACKAGE     Package to test (ustar-parser|ustar-tools)"
            echo "  --features FEATURES   Feature set (default|no-default)"
            echo "  --os OS               OS simulation (ubuntu-latest|macos-latest)"
            echo "  --help, -h            Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 --package ustar-tools --features default"
            echo "  $0 --package ustar-parser --features no-default"
            echo "  $0 --package ustar-tools --features no-default --os macos-latest"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set defaults if not specified
PACKAGE=${PACKAGE:-ustar-parser}
FEATURES=${FEATURES:-default}

echo "Running CI Test Suite locally..."
echo "Simulating: Test Suite ($OS, stable, $PACKAGE, $FEATURES)"
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

# Determine cargo flags based on features
if [ "$FEATURES" = "no-default" ]; then
    FEATURE_FLAGS="--no-default-features"
    FEATURE_DESC="no-default features"
else
    FEATURE_FLAGS=""
    FEATURE_DESC="default features"
fi

# Step 1: Build (matches CI build step)
print_step "Building $PACKAGE with $FEATURE_DESC"
cargo build --verbose --all-targets -p "$PACKAGE" $FEATURE_FLAGS
print_success "Build completed"

echo ""

# Step 2: Run tests (matches CI test step)
print_step "Running tests for $PACKAGE with $FEATURE_DESC"
cargo test --verbose -p "$PACKAGE" $FEATURE_FLAGS
print_success "Tests passed"

echo ""

# Step 3: Test documentation (matches CI doc test step)
print_step "Testing documentation for $PACKAGE"
cargo test --doc -p "$PACKAGE" $FEATURE_FLAGS
print_success "Documentation tests passed"

echo ""

# Step 4: Test binaries work (only for ustar-tools)
if [ "$PACKAGE" = "ustar-tools" ]; then
    print_step "Testing that binaries work"
    echo "Testing ustar-dumper..."
    cargo run --bin ustar-dumper $FEATURE_FLAGS -- --help >/dev/null
    echo "Testing ustar-benchmark..."
    cargo run --bin ustar-benchmark $FEATURE_FLAGS -- --help >/dev/null
    echo "Testing ustar-parse-debugger..."
    cargo run --bin ustar-parse-debugger $FEATURE_FLAGS -- --help >/dev/null
    print_success "All binaries work correctly"
    echo ""
fi

print_success "All CI Test Suite checks passed! 🎉"
echo "This matches: Test Suite ($OS, stable, $PACKAGE, $FEATURES)"