#!/bin/bash
# Run integration tests (called by CI and locally)

set -e

echo "=== Integration Tests ==="

# Set environment variables to match CI
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1

echo "Running integration tests with release optimizations..."
echo "This includes BMRB, COD, PDB, and other real-world file tests"

# Run comprehensive integration tests (--no-fail-fast to see all failures)
cargo test --release --no-fail-fast --test integration_tests -p ustar-parser

echo "✅ Integration tests completed"