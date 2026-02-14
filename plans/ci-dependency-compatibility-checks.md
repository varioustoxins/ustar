# Plan: CI Dependency Compatibility Checks

## Problem

The workspace commits `Cargo.lock` (required because ustar-tools has binaries),
which means CI always tests with pinned dependency versions. This has two gaps:

1. **Newer deps**: If a dependency releases a semver-compatible update that
   breaks something (as pest 2.8.6 did with Debug format changes), we don't
   find out until someone reports it or we manually run `cargo update`.

2. **Older deps**: If we accidentally use an API only available in a newer
   version of a dependency but our `Cargo.toml` claims compatibility with
   older versions, downstream users with older lockfiles get compile errors.

## Current CI Structure

The existing `ci.yml` pipeline runs:
- `grammar-generation` - generates pest grammars from templates
- `test` - 8-job matrix (2 OS x 2 feature sets x 2 packages) using `Cargo.lock`
- `code-quality` - clippy + rustfmt
- `integration-tests` - real-world data parsing tests
- `release` - crates.io publishing

All of these use the committed `Cargo.lock` for reproducibility.

## Proposed Changes

Add a **separate workflow** (`dep-compat.yml`) that runs on a weekly schedule
plus manual trigger. This keeps the main CI fast on every push while catching
dependency drift.

### Job 1: Unlocked Dependencies (latest compatible)

**Purpose**: Catch issues with newer semver-compatible dependency versions.

```yaml
unlocked-deps:
  name: Unlocked Dependencies (latest)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo update
    - run: cargo check --workspace --all-targets
    - run: cargo test --lib --no-fail-fast -p ustar-parser -p ustar-tools
```

**Key decisions**:
- **Ubuntu only** - if it compiles on one OS, it compiles on all
- **`cargo check` first** - fast compile check across all targets
- **`--lib` tests only** - no integration tests, no test data downloads
- **No feature matrix** - default features only
- **~2-3 minutes** estimated runtime

### Job 2: Minimal Versions (oldest compatible)

**Purpose**: Verify our `Cargo.toml` version specs are accurate - that the
minimum versions we claim to support actually compile.

```yaml
minimal-versions:
  name: Minimal Versions (oldest)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@nightly
    - run: cargo +nightly -Z minimal-versions check --workspace
```

**Key decisions**:
- **Check only, no tests** - we just need to verify it compiles
- **Nightly required** - `-Z minimal-versions` is an unstable cargo flag
- **~1-2 minutes** estimated runtime
- Note: if minimal-versions check fails it may indicate our `Cargo.toml`
  lower bounds need bumping, not that the code is wrong

### Job 3: Locked Dependencies (baseline, optional)

**Purpose**: Identical to the main CI test job but explicitly documented as
the "reproducible build" baseline.

This already exists in the main `ci.yml` workflow and does NOT need
duplicating. It is listed here for completeness to show the three-tier
strategy:

| Tier | Versions | Runs on | Purpose |
|------|----------|---------|---------|
| Locked | Committed `Cargo.lock` | Every push | Reproducible, catches regressions |
| Unlocked | Latest compatible | Weekly + manual | Catches new dep issues early |
| Minimal | Oldest compatible | Weekly + manual | Validates `Cargo.toml` bounds |

## Workflow File

New file: `.github/workflows/dep-compat.yml`

```yaml
name: Dependency Compatibility

on:
  schedule:
    - cron: '0 6 * * 1'  # Every Monday at 06:00 UTC
  workflow_dispatch:       # Manual trigger

env:
  CARGO_TERM_COLOR: always

jobs:
  grammar-generation:
    name: Generate Grammars
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: ./scripts/ci-grammar-generation.sh
      - uses: actions/upload-artifact@v4
        with:
          name: generated-grammars
          path: ustar-parser/src/star_*.pest

  unlocked-deps:
    name: Latest Dependencies
    runs-on: ubuntu-latest
    needs: grammar-generation
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/download-artifact@v4
        with:
          name: generated-grammars
          path: ustar-parser/src/
      - name: Update all dependencies to latest compatible
        run: cargo update
      - name: Check compilation
        run: cargo check --workspace --all-targets
      - name: Run lib tests
        run: cargo test --lib --no-fail-fast -p ustar-parser -p ustar-tools

  minimal-versions:
    name: Minimum Dependency Versions
    runs-on: ubuntu-latest
    needs: grammar-generation
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: actions/download-artifact@v4
        with:
          name: generated-grammars
          path: ustar-parser/src/
      - name: Check with minimal dependency versions
        run: cargo +nightly -Z minimal-versions check --workspace
```

## What Happens When a Check Fails

- **Unlocked fails**: A newer dependency version broke something.
  - **Action**: Either pin the dep to a specific version range in
    `Cargo.toml`, or update our code to work with the new version.
  - **Urgency**: Medium - existing users on older versions are fine.

- **Minimal fails**: Our `Cargo.toml` lower bounds are too low.
  - **Action**: Bump the minimum version in `Cargo.toml` to match
    what we actually need.
  - **Urgency**: Low - most users run recent versions anyway.

- **Neither should block main CI** - these are informational/advisory.
  GitHub will send notifications on failure.

## Implementation Steps

1. Create `.github/workflows/dep-compat.yml` with the workflow above
2. Test by triggering manually via `workflow_dispatch`
3. If minimal-versions fails immediately (common for transitive deps),
   consider starting with unlocked-deps only and adding minimal-versions
   later once `Cargo.toml` bounds are tightened
4. Add a badge to `README.md` (optional)

## Estimated Total CI Cost

- Weekly scheduled run: ~5 minutes total (both jobs in parallel)
- No impact on per-push CI speed
- Free tier GitHub Actions: ~2000 minutes/month, this uses ~20 minutes/month
