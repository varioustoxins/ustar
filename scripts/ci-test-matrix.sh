#!/bin/bash
# Run test matrix for a specific package/features combination
# Supports both positional and named arguments

set -e

# Default values
PACKAGE=""
FEATURES=""
PLATFORM=""

# Parse command line arguments (named flags take precedence)
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
        --platform)
            PLATFORM="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS] or $0 <package> <features> [platform]"
            echo ""
            echo "Options:"
            echo "  --package PACKAGE     Package to test (ustar-parser|ustar-tools)"
            echo "  --features FEATURES   Feature set (default|no-default)"
            echo "  --platform PLATFORM  Platform simulation (macos|ubuntu)"
            echo "  --help, -h            Show this help message"
            echo ""
            echo "Positional arguments (legacy):"
            echo "  $0 ustar-tools default macos"
            echo "  $0 ustar-parser no-default ubuntu"
            echo ""
            echo "Named arguments (recommended):"
            echo "  $0 --package ustar-tools --features default --platform macos"
            echo "  $0 --package ustar-parser --features no-default --platform ubuntu"
            exit 0
            ;;
        -*)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
        *)
            # Positional arguments (legacy support)
            if [ -z "$PACKAGE" ]; then
                PACKAGE="$1"
            elif [ -z "$FEATURES" ]; then
                FEATURES="$1"
            elif [ -z "$PLATFORM" ]; then
                PLATFORM="$1"
            else
                echo "Too many positional arguments"
                echo "Use --help for usage information"
                exit 1
            fi
            shift
            ;;
    esac
done

# Set defaults if not specified
PACKAGE=${PACKAGE:-ustar-parser}
FEATURES=${FEATURES:-default}
PLATFORM=${PLATFORM:-macos}

echo "=== Test Matrix: $PACKAGE ($FEATURES features) on $PLATFORM ==="

if [ "$FEATURES" = "default" ]; then
    FEATURES_FLAG=""
    BUILD_DESC="with default features"
    TEST_DESC="with default features"
else
    FEATURES_FLAG="--no-default-features"
    BUILD_DESC="without default features"
    TEST_DESC="without default features"
fi

# Step 1: Build
echo "Building $PACKAGE $BUILD_DESC..."
cargo build --verbose --all-targets $FEATURES_FLAG -p $PACKAGE

# Step 2: Debug walker differences (only for ustar-parser with default features)
if [ "$FEATURES" = "default" ] && [ "$PACKAGE" = "ustar-parser" ]; then
    echo "Running debug walker test to show platform differences..."
    cargo test test_comprehensive_example_walker_output --test sas_walker_tests --verbose -p ustar-parser -- --nocapture || true
fi

# Step 3: Run tests
echo "Running tests for $PACKAGE $TEST_DESC..."
cargo test --verbose $FEATURES_FLAG -p $PACKAGE

# Step 4: Test documentation (only for default features with stable rust)
RUST_VERSION=${RUST_VERSION:-stable}
if [ "$FEATURES" = "default" ] && [ "$RUST_VERSION" = "stable" ]; then
    echo "Testing documentation for $PACKAGE..."
    cargo test --doc -p $PACKAGE
fi

# Step 5: Test binaries work (only for ustar-tools with default features and stable rust)
if [ "$FEATURES" = "default" ] && [ "$RUST_VERSION" = "stable" ] && [ "$PACKAGE" = "ustar-tools" ]; then
    echo "Testing that binaries work..."
    echo "  Testing ustar-dumper..."
    cargo run --bin ustar-dumper -- --help >/dev/null
    echo "  Testing ustar-benchmark..."
    cargo run --bin ustar-benchmark -- --help >/dev/null
    echo "  Testing ustar-parse-debugger..."
    cargo run --bin ustar-parse-debugger -- --help >/dev/null
fi

echo "✅ Test matrix completed: $PACKAGE ($FEATURES) on $PLATFORM"