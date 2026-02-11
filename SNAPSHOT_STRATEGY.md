# Snapshot File Management Strategy

## Overview

The ustar project uses snapshot testing with `insta` for regression testing. Due to the large size of many snapshot files, we use compression to manage repository size and package distribution. This document describes the strategy and workflow for managing compressed and uncompressed snapshot files.

## Compression Formats

- **ustar-parser** tests: Use **zstd** compression (`.snap.zst` files)
- **ustar-tools** tests: Use **gzip** compression (`.snap.gz` files)

> **Note**: This inconsistency exists for historical reasons. Future work may standardize on zstd across all tests.

## File Distribution Strategy

### 1. crates.io Packages: NO snapshot files or test_data files

Packages published to crates.io should contain **no snapshot files** (compressed or uncompressed) or **large test data files**.

**Rationale**: Test data and snapshot files are only needed for development and testing. They significantly increase package size and download times for users who just want to use the library.

The snapshot files in both compressed and uncompressed forms are excluded from cargo builds by an excluded sections filters as are large test data files e.g extended funftional test of  nef, cif, str etc. However, some smaller test data files are present in the cargo package.

### 2. Git Repository / GitHub: test data file and ONLY compressed snapshot files 

The git repository should track **only compressed snapshot files** (`.snap.zst` and `.snap.gz`) and test data files.

Uncompressed `.snap` files are generated locally during testing and should not be committed.

Uncompressed snap files are excluded from github by filters in .gitignore.

### 3. Local Development: Both compressed and uncompressed files

During local development and testing:
- Compressed files (`.snap.zst`/`.snap.gz`) are committed to the repository
- Uncompressed files (`.snap`) are generated on-demand and used by `insta`
- Uncompressed files are ignored by git

### 4. CI/Fresh Checkouts: Automatic test data download from GitHub

When running tests in CI or on a fresh checkout where test data files might be missing:

**Implemented Workflow**:
1. Test suite calls `ensure_test_data_available()` from `ustar-test-utils`
2. Discovers test data directories by looking for `checksums.sha1` files
3. Checks which files are missing from each directory
4. Automatically downloads entire repository archive from GitHub main branch
5. Extracts only test data files from the archive
6. Verifies checksums of all downloaded files
7. Tests can proceed normally

**Key Features**:
- Downloads from `https://github.com/varioustoxins/ustar/archive/refs/heads/main.zip`
- Uses in-memory processing (no temporary files needed)
- Downloads only once per test run (cached with `OnceLock`)
- Can be disabled with `--features no-large-tests`
- Provides helpful manual download instructions if automatic download fails

## Snapshot File Workflow

### Testing with Snapshots

When `insta` runs tests:

1. **Decompression**: Compressed snapshots (`.snap.zst`/`.snap.gz`) are decompressed to `.snap` files
   the list of test data files are stored in `checksums.sha1` files in directories where test data is missing from 
   the cargo package

2. **Implemented Tooling**:

1. **Automatic Decompression** - `ustar-test-utils/snapshot_utils.rs`:
   - `ensure_snapshots_synchronized()` automatically decompresses `.snap.zst` files to `.snap`
   - Called before tests run to ensure `insta` has uncompressed files
   - Compares content to avoid unnecessary decompression
   - `read_snapshot()` transparently reads from compressed or uncompressed files

2. **Snapshot Acceptance Script** - `scripts/insta-zstd.sh`:
   - Full wrapper around `cargo insta` commands
   - **Before running insta**: Decompresses all `.snap.zst` files to `.snap` files
   - Tracks modification times of `.snap` files before and after insta runs
   - **After running insta**: Automatically compresses changed `.snap` files back to `.snap.zst`
   - Cleans up temporary files (`.snap.new`, `.snap.old`, `.snap.diff`)
   - Usage: `./scripts/insta-zstd.sh accept` or `./scripts/insta-zstd.sh review`

3. **Custom Snapshot Assertions** - `ustar-test-utils/snapshot_utils.rs`:
   - `assert_snapshot_gz()` - Works with compressed snapshots, panics on mismatch
   - `check_snapshot_gz()` - Non-panicking version for collecting multiple failures
   - Automatically creates `.snap.diff` and `.snap.old` files for review
   - Error messages include instructions to run `./scripts/insta-zstd.sh`

> **Important**: `insta` currently cannot work directly with compressed snapshots. It requires uncompressed `.snap` files.

### Keeping Files in Sync

**Tools should ensure**:
1. Every `.snap` file has a corresponding compressed version (`.snap.zst` or `.snap.gz`)
2. When snapshots are updated they are updated via via `insta-zstd.sh` which wraps `cargo insta` to ensure corect the compressed versions are automatically updated
3. Checksums or timestamps verify that compressed and uncompressed versions match

## File Naming Convention

Each snapshot test generates two files:
4. **Sync tooling** - Complete implementation in `ustar-test-utils` and `scripts/insta-zstd.sh`
5. **Test data download** - Automatic download from GitHub works perfectly
6. **Checksum verification** - All test data integrity is verified via SHA-1

- `test_name.snap` - Uncompressed working file (git-ignored, local only)
- `test_name.snap.zst` or `test_name.snap.gz` - Compressed version (git-tracked, pushed to GitHub)

## TODO

**Inconsistent compression** (LOW PRIORITY - cosmetic issue)
   - ustar-parser uses zstd (`.snap.zst`)
   - ustar-tools uses gzip (`.snap.gz`)
   - Both work fine; standardizing would be nice but not critical


## Related Files

- `.gitignore` - Snapshot ignore rules
- `ustar-parser/Cargo.toml` - Parser crate packaging config (has snapshot exclusions)
- `ustar-tools/Cargo.toml` - Tools crate packaging config (has snapshot exclusions)
- `scripts/insta-zstd.sh` - wraps and extends `cargo insta` to ensure correct compression 
- `ustar-test-utils/src/test_data_download_utils.rs` - utilities to download test data 
- `ustar-test-utils/src/snapshot_utils.rs` - utilities for dealing with compressed snapshots during tests
- `checksums.sha1` - placeholders an checksums for files that need to be downloaded


