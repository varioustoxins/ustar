# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

USTAR is a general STAR (Self-defining Text Archive and Retrieval) format parser written in Rust. STAR is a data format commonly used in scientific computing, particularly for crystallographic data (CIF files), NMR-STAR files (BMRB), and mmCIF (Protein Data Bank) files.

## Build and Development Commands

### Build
```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo build --all-targets      # Build all targets including tests and benchmarks
```

### Testing
```bash
cargo test --no-fail-fast      # Run all tests (always use --no-fail-fast)
cargo test --no-fail-fast parser_tests        # Run specific test module
cargo test --no-fail-fast --test parse_bmrb_stars    # Run integration test for BMRB files
cargo test --no-fail-fast --test parse_cod_cifs      # Run integration test for COD CIF files
cargo test --no-fail-fast --test parse_pdb_mmcifs    # Run integration test for PDB mmCIF files
```

### Binaries
The project includes several command-line tools:
```bash
cargo run --bin ustar-dumper           # Parse and dump STAR files with visualization
cargo run --bin ustar-benchmark        # Performance benchmarking
cargo run --bin ustar-parse-debugger   # Debug parser behavior
```

## Architecture and Key Components

### Multi-Encoding Parser System
The parser supports three encoding modes through dynamically generated grammars:
- **ASCII**: Standard ASCII character set
- **ExtendedAscii**: Extended ASCII including characters up to 0xFF
- **Unicode**: Full Unicode support with comprehensive whitespace handling

Grammar files are generated at build time by `build.rs` from a template (`src/star.pest_template`) using placeholder substitution.

### Core Components

**Parser Module (`src/parsers.rs`)**
- Three separate parser modules (ascii, extended, unicode) to avoid Rule enum conflicts
- Each uses Pest grammar files generated at build time
- All parsers share the same Rule enum structure

**Configuration System (`src/config.rs`)**
- `ParserConfig` type for runtime configuration
- Supports encoding mode selection, string decomposition options, and BOM detection
- Default configurations available via `default_config()`

**Mutable Parse Tree (`src/mutable_pair.rs`)**
- `MutablePair` provides a mutable alternative to Pest's immutable `Pair` type
- Enables post-parsing transformations like string decomposition
- Converts from Pest pairs via `MutablePair::from_pest_pair()`

**String Processing**
- `src/string_decomposer.rs`: Transforms string tokens into delimiter + content + delimiter
- Optional feature controlled by `DecomposedStrings` configuration

**Buffered Processing (`src/sas_buffered.rs`, `src/sas_buffered_walker.rs`)**
- Handler traits for for output to SAS [SAX like API]
- Walker pattern for traversing parse trees efficiently

### Test Data and Integration Tests
Extensive test suite includes:
- Unit tests in `tests/parser_tests.rs` and `tests/encoding_tests.rs`
- Integration tests with real-world data:
  - BMRB NMR-STAR files (`tests/parse_bmrb_stars.rs`)
  - Crystallography Open Database CIF files (`tests/parse_cod_cifs.rs`)  
  - Protein Data Bank mmCIF files (`tests/parse_pdb_mmcifs.rs`)
- Test data stored in `tests/test_data/` with samples from real databases
- When running tests this should be done in release mode, as it is _much_ faster
- When running cargo test, do NOT use `| tail` or other output truncation - show full output
- Always use `--no-fail-fast` with cargo test to see all failures, not just the first one

### Snapshot Testing
```bash
./scripts/insta-accept.sh --keep-diffs    # Accept snapshots, keep .diff files for review (DEFAULT)
./scripts/insta-accept.sh                  # Accept snapshots and remove .diff files
```
- When running insta-accept.sh, use `--keep-diffs` by default to preserve diff files for review

### Grammar Template System
The `build.rs` script generates three grammar variants from `src/star.pest_template`:
- Placeholder system allows encoding-specific character class definitions
- Unicode whitespace handling includes comprehensive character ranges
- Generated files: `star_ascii.pest`, `star_extended.pest`, `star_unicode.pest`

## Development Notes

- The parser handles STAR format variants including CIF, NMR-STAR, mmCIF, and NEF
- BOM detection is automatic when enabled in configuration
- String decomposition is optional and controlled via configuration
- All parsers share identical rule structures but differ in character class definitions
- The project includes extensive real-world test data for validation

## Version control
- _ALL_ commits to version control will be made by the user not claude...