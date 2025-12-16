#!/bin/bash
# Run complete CI pipeline locally
# Usage: ./scripts/ci-run-all.sh [quick|full]

set -e

MODE=${1:-quick}

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_header() {
    echo -e "\n${BLUE}================================================${NC}"
    echo -e "${BLUE} $1${NC}"
    echo -e "${BLUE}================================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

if [ "$MODE" = "quick" ]; then
    print_header "Running Quick CI Pipeline (macOS, ustar-tools, default)"
    
    # Step 1: Grammar generation
    ./scripts/ci-grammar-generation.sh
    
    # Step 2: Code quality
    ./scripts/ci-code-quality.sh
    
    # Step 3: Test matrix (just ustar-tools with default features)
    ./scripts/ci-test-matrix.sh ustar-tools default macos
    
    print_success "Quick CI pipeline completed!"
    print_info "For full CI coverage, run: ./scripts/ci-run-all.sh full"

elif [ "$MODE" = "full" ]; then
    print_header "Running Full CI Pipeline"
    
    # Step 1: Grammar generation
    ./scripts/ci-grammar-generation.sh
    
    # Step 2: Code quality
    ./scripts/ci-code-quality.sh
    
    # Step 3: Test matrix - all combinations
    print_header "Test Matrix: All Package/Feature Combinations"
    
    echo "Running ustar-parser (default features)..."
    ./scripts/ci-test-matrix.sh ustar-parser default macos
    
    echo "Running ustar-parser (no-default features)..."
    ./scripts/ci-test-matrix.sh ustar-parser no-default macos
    
    echo "Running ustar-tools (default features)..."
    ./scripts/ci-test-matrix.sh ustar-tools default macos
    
    echo "Running ustar-tools (no-default features)..."
    ./scripts/ci-test-matrix.sh ustar-tools no-default macos
    
    # Step 4: Integration tests
    ./scripts/ci-integration-tests.sh
    
    print_success "Full CI pipeline completed!"
    print_info "This covers the same scope as CI on GitHub Actions"
    
    echo ""
    ./scripts/ci-local-rust-is-stable.sh

else
    echo "Usage: $0 [quick|full]"
    echo "  quick - Run basic CI checks (default)"
    echo "  full  - Run complete CI pipeline"
    exit 1
fi

echo ""
print_success "All CI checks passed! 🎉"