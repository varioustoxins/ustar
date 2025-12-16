#!/bin/bash
# Quick local testing - what you'd run before committing
# Equivalent to: Test Suite (macos-latest, stable, ustar-tools, default)

set -e

echo "🚀 Running local development checks..."
echo "This simulates the CI environment you're most likely to match"
echo ""

# Run the specific CI combination that matches local development
./scripts/ci-test-matrix.sh ustar-tools default macos

echo ""
./scripts/ci-local-rust-is-stable.sh

echo ""
echo "💡 To run more comprehensive checks:"
echo "   Code Quality:    ./scripts/ci-code-quality.sh"
echo "   Full CI:         ./scripts/ci-run-all.sh full"
echo "   Integration:     ./scripts/ci-integration-tests.sh"