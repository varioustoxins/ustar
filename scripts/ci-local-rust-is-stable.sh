#!/bin/bash
# Check if local Rust version matches stable
# Can be run standalone or integrated into other CI scripts

set -e

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

# Get local Rust version
LOCAL_VERSION=$(rustc --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')

# Get stable version from rustup (fallback to forge API if needed)
if command -v rustup &> /dev/null; then
    STABLE_VERSION=$(rustup check 2>/dev/null | grep -oE 'stable-[^ ]+ - [0-9]+\.[0-9]+\.[0-9]+' | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
else
    STABLE_VERSION="unknown"
fi

# If rustup didn't work, try forge API
if [ "$STABLE_VERSION" = "unknown" ]; then
    if command -v curl &> /dev/null; then
        STABLE_VERSION=$(curl -s https://forge.rust-lang.org/infra/channel-releases.html | grep -oE 'Rust [0-9]+\.[0-9]+\.[0-9]+' | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
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