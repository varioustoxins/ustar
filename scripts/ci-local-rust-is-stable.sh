#!/bin/bash
# Check if local Rust version matches stable
# Can be run standalone or integrated into other CI scripts

set -e

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

# Extract version number from a string like "rustc 1.92.0 (hash date)"
extract_version() {
    local version_string="$1"
    # Remove everything before the first space
    version_string="${version_string#* }"
    # Remove everything after the first space (hash and date)
    version_string="${version_string%% *}"
    echo "$version_string"
}

# Get local Rust version
LOCAL_VERSION=$(extract_version "$(rustc --version)")

# Get stable version from rustup
STABLE_VERSION="unknown"
if command -v rustup &> /dev/null; then
    # Parse rustup check output like "stable-x86_64-apple-darwin - Up to date : 1.92.0"
    rustup_output=$(rustup check 2>/dev/null | grep "stable-" | head -1)
    if [ -n "$rustup_output" ]; then
        # Extract everything after the colon and space
        after_colon="${rustup_output##*: }"
        # Remove any parentheses and what follows
        STABLE_VERSION="${after_colon%% (*}"
        # Remove any trailing whitespace
        STABLE_VERSION="${STABLE_VERSION%% }"
    fi
fi

# Fallback: try to get stable version from release info
if [ "$STABLE_VERSION" = "unknown" ] && command -v curl &> /dev/null; then
    # Get current stable from GitHub API (simpler than parsing HTML)
    api_response=$(curl -s "https://api.github.com/repos/rust-lang/rust/releases" 2>/dev/null || echo "")
    if [ -n "$api_response" ]; then
        # Look for first tag that looks like a version number (starts with number)
        for tag in $(echo "$api_response" | grep '"tag_name"' | head -5); do
            # Extract tag value between quotes
            tag="${tag#*\"}"
            tag="${tag%%\"*}"
            # If it starts with a digit, it's probably a version
            first_char="${tag:0:1}"
            if [[ "$first_char" == [0-9] ]]; then
                STABLE_VERSION="$tag"
                break
            fi
        done
    fi
fi

echo "Rust version check:"
echo "  Local:  $LOCAL_VERSION"
echo "  Stable: $STABLE_VERSION"

if [ "$STABLE_VERSION" = "unknown" ]; then
    echo -e "${YELLOW}⚠️  Could not determine stable Rust version${NC}"
    exit 0
fi

if [ "$LOCAL_VERSION" = "$STABLE_VERSION" ]; then
    echo -e "${GREEN}✅ Using stable Rust${NC}"
elif [[ "$LOCAL_VERSION" > "$STABLE_VERSION" ]]; then
    echo ""
    echo -e "${YELLOW}================================================${NC}"
    echo -e "${YELLOW}⚠️  WARNING: You're using newer Rust than stable!${NC}"
    echo -e "${YELLOW}   Local: $LOCAL_VERSION | Stable: $STABLE_VERSION${NC}"
    echo -e "${YELLOW}   CI uses stable - behavior may differ${NC}"
    echo -e "${YELLOW}================================================${NC}"
    echo ""
else
    echo ""
    echo -e "${RED}================================================${NC}"
    echo -e "${RED}❌ WARNING: You're using older Rust than stable!${NC}"
    echo -e "${RED}   Local: $LOCAL_VERSION | Stable: $STABLE_VERSION${NC}"
    echo -e "${RED}   Consider updating: rustup update${NC}"
    echo -e "${RED}================================================${NC}"
    echo ""
fi