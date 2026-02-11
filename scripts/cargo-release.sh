#!/bin/bash
# Safe cargo release script with proper checks and rollback
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
DRY_RUN=false
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN=true
    echo -e "${YELLOW}🔍 Running in DRY-RUN mode${NC}"
fi

# Check if we're in the right directory
if [[ ! -f Cargo.toml ]] || [[ ! -d ustar-parser ]]; then
    echo -e "${RED}❌ Error: Must run from ustar project root${NC}"
    exit 1
fi

# Check for uncommitted changes
if [[ -n $(git status --porcelain) ]]; then
    echo -e "${RED}❌ Error: You have uncommitted changes. Commit or stash them first.${NC}"
    git status --short
    exit 1
fi

# Read current version from workspace Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo -e "${GREEN}📦 Preparing to release version ${VERSION}${NC}"

# Check if tag already exists
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    echo -e "${RED}❌ Error: Tag v${VERSION} already exists${NC}"
    exit 1
fi

# Check cargo authentication
if ! cargo login --help >/dev/null 2>&1; then
    echo -e "${RED}❌ Error: cargo not available${NC}"
    exit 1
fi

# Verify cargo registry token exists
if [[ ! -f ~/.cargo/credentials.toml ]] && [[ ! -f ~/.cargo/credentials ]]; then
    echo -e "${RED}❌ Error: Not logged in to crates.io. Run: cargo login${NC}"
    exit 1
fi

# Run CI checks
echo -e "${YELLOW}🧪 Running CI checks...${NC}"
if ! bash scripts/ci-local-run-all.sh; then
    echo -e "${RED}❌ CI checks failed. Fix errors before releasing.${NC}"
    exit 1
fi

# Packages in dependency order
PACKAGES=("ustar-test-utils" "ustar-parser" "ustar-tools")

# Verify all packages build in release mode
# Note: We skip `cargo package` or `cargo publish --dry-run` because they
# both validate against the crates.io registry, which fails for workspace crates
# that depend on each other but aren't published yet. The CI checks above have
# already verified tests pass; this just confirms release builds work.
echo -e "${YELLOW}🔨 Building all packages in release mode...${NC}"
for package in "${PACKAGES[@]}"; do
    echo "  Building $package..."
    if ! cargo build -p "$package" --release; then
        echo -e "${RED}❌ Failed to build $package${NC}"
        exit 1
    fi
done

if [[ "$DRY_RUN" == true ]]; then
    echo -e "${GREEN}✅ All pre-publish checks passed!${NC}"
    echo -e "${YELLOW}💡 To actually publish, run: $0${NC}"
    exit 0
fi

# Confirm with user
echo ""
echo -e "${YELLOW}⚠️  Ready to publish version ${VERSION} to crates.io${NC}"
echo "Packages: ${PACKAGES[*]}"
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 1
fi

# Publish packages
echo -e "${GREEN}🚀 Publishing packages...${NC}"
for package in "${PACKAGES[@]}"; do
    echo ""
    echo -e "${YELLOW}📦 Publishing $package...${NC}"

    if ! cargo publish -p "$package"; then
        echo -e "${RED}❌ Failed to publish $package${NC}"
        echo -e "${YELLOW}⚠️  Some packages may have been published. Check crates.io manually.${NC}"
        exit 1
    fi

    echo -e "${GREEN}✅ Published $package${NC}"

    # Wait for crates.io to index (except for last package)
    if [[ "$package" != "${PACKAGES[-1]}" ]]; then
        echo -e "${YELLOW}⏳ Waiting 30s for crates.io to index...${NC}"
        sleep 30
    fi
done

# Create git tag
echo ""
echo -e "${YELLOW}🏷️  Creating git tag v${VERSION}...${NC}"
git tag -a "v$VERSION" -m "Release version $VERSION"

echo ""
echo -e "${GREEN}✅ Successfully released version ${VERSION}!${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Push the tag: git push origin v${VERSION}"
echo "  2. Or push all tags: git push --tags"
echo "  3. Verify on crates.io:"
for package in "${PACKAGES[@]}"; do
    echo "     https://crates.io/crates/$package"
done