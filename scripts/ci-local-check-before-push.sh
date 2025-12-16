#!/bin/bash
# Run before pushing to avoid CI failures
# This runs the most common CI failure points

set -e

echo "🔍 Running pre-push checks to avoid CI failures..."
echo ""

# 1. Code quality (most common CI failure)
echo "1️⃣ Code Quality Checks"
./scripts/ci-code-quality.sh

echo ""

# 2. Quick test run
echo "2️⃣ Quick Test Run"
./scripts/ci-test-matrix.sh ustar-tools default macos

echo ""
./scripts/ci-local-rust-is-stable.sh

echo ""
echo "✅ Pre-push checks completed!"
echo ""
echo "💡 These checks cover ~80% of potential CI failures."
echo "   For full coverage: ./scripts/ci-run-all.sh full"