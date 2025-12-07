//! Re-export test utilities from the shared ustar-test-utils crate.
//!
//! This module is used via `mod snapshot_utils;` in other test files,
//! so these re-exports are used even though rust-analyzer may not see it.

#![allow(unused_imports)]

pub use ustar_test_utils::{assert_snapshot_gz, check_snapshot_gz, SnapshotMismatch};
