# CI Scripts

Some of the scripts this directory contains [those whose names begin with ci] are part of the ci system and are used for both by GitHub Actions CI and local development. This ensures perfect parity between CI and local environments.

# Naming Conventions

* all continuous integration scripts start with ci
* script that start ci-local are only designed for running locally and are not part of the GitHub Actions CI  infrastructure 
* all the ci scripst can be run locally or can be run in useful gtoups using ci-local scripts

## Quick Start

```bash
# Fast local development check
./scripts/ci-local-test.sh

# Before pushing (avoid CI failures)  
./scripts/ci-local-check-before-push.sh

# Full CI simulation
./scripts/ci-local-run-all.sh full
```

## Individual Scripts

### Core CI Components (used by both local and CI)
- **`ci-grammar-generation.sh`** - Generate grammar files (build step)
- **`ci-code-quality.sh`** - Format, clippy, documentation checks
- **`ci-test-matrix.sh`** - Test specific package/feature combination
- **`ci-integration-tests.sh`** - Run real-world file integration tests

### Local Development Workflows
- **`ci-local-run-all.sh [quick|full]`** - Complete CI pipeline
- **`ci-local-test.sh`** - Quick development testing
- **`ci-local-check-before-push.sh`** - Pre-push validation
- **`ci-local-rust-is-stable.sh`** - Check if local Rust matches stable version

### Legacy (still available)
- **`ci-clippy.sh`** - Just clippy checks
- **`ci-test-suite.sh`** - Original test suite script

## Usage Examples

```bash
# Test specific combinations
./scripts/ci-test-matrix.sh ustar-parser default macos
./scripts/ci-test-matrix.sh ustar-tools no-default macos

# Run what CI runs
./scripts/ci-code-quality.sh              # Code Quality job
./scripts/ci-integration-tests.sh         # Integration Tests job
./scripts/ci-local-run-all.sh full        # All CI jobs

# Development workflows
./scripts/ci-local-test.sh                # Quick feedback
./scripts/ci-local-check-before-push.sh   # Avoid CI failures

# Check Rust version
./scripts/ci-local-rust-is-stable.sh      # Standalone version check
```

## CI Integration

GitHub Actions calls these same scripts, ensuring:
- ✅ Perfect local/CI parity
- ✅ Faster local debugging  
- ✅ Consistent test environments
- ✅ Single source of truth for CI logic

## Platform Notes

Scripts default to `macos` platform but accept platform parameter. CI uses `ubuntu-latest` and `macos-latest` in the matrix.