//! Shared test utilities for ustar crates.
//!
//! This crate provides common testing functionality including
//! support for gzip-compressed snapshot files.

mod snapshot_utils;

pub use snapshot_utils::{assert_snapshot_gz, check_snapshot_gz, read_snapshot, SnapshotMismatch};
