//! Shared test utilities for ustar crates.
//!
//! This crate provides common testing functionality including
//! support for gzip-compressed snapshot files and test data management.

mod snapshot_utils;
mod test_data_download_utils;

pub use snapshot_utils::{assert_snapshot_gz, check_snapshot_gz, read_snapshot, SnapshotMismatch};
pub use test_data_download_utils::{ensure_test_data_available, verify_test_data_checksums};
