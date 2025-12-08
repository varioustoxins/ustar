use flate2::read::GzDecoder;
use similar::{ChangeTag, TextDiff};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Read a snapshot file, automatically decompressing if it's gzipped
pub fn read_snapshot<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn std::error::Error>> {
    let path = path.as_ref();
    let gz_path = path.with_extension("snap.gz");

    // Try to read the gzipped version first
    if gz_path.exists() {
        let file = File::open(&gz_path)?;
        let mut decoder = GzDecoder::new(file);
        let mut content = String::new();
        decoder.read_to_string(&mut content)?;
        Ok(content)
    }
    // Fall back to uncompressed version
    else if path.exists() {
        std::fs::read_to_string(path).map_err(Into::into)
    } else {
        Err(format!(
            "Snapshot file not found: {} or {}",
            path.display(),
            gz_path.display()
        )
        .into())
    }
}

/// Result of a snapshot check - either Ok or a mismatch with details
#[derive(Debug)]
pub struct SnapshotMismatch {
    pub snapshot_name: String,
    pub diff_path: std::path::PathBuf,
    pub new_path: std::path::PathBuf,
}

impl std::fmt::Display for SnapshotMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Snapshot mismatch for '{}'\n  Diff: {}\n  New:  {}",
            self.snapshot_name,
            self.diff_path.display(),
            self.new_path.display()
        )
    }
}

/// Check a snapshot without panicking. Returns Ok(()) if the snapshot matches,
/// or Err(SnapshotMismatch) if there's a mismatch. Creates .snap.new and .snap.diff
/// files on mismatch for later review/acceptance.
///
/// Use this in loops where you want to collect all failures before panicking.
/// For single-shot tests, use `assert_snapshot_gz` instead.
///
/// Looks for snapshots in the calling package's `tests/snapshots/` directory.
pub fn check_snapshot_gz(snapshot_name: &str, value: &str) -> Result<(), SnapshotMismatch> {
    let snapshot_dir = get_snapshot_dir();
    let snapshot_path = snapshot_dir.join(format!("{}.snap", snapshot_name));

    // Check if we already have a snapshot (compressed or uncompressed)
    match read_snapshot(&snapshot_path) {
        Ok(expected) => {
            // We have an existing snapshot - compare it
            let expected_content = extract_snapshot_content(&expected);
            if expected_content.trim() == value.trim() {
                // Match! Compress if needed and return success
                if let Err(e) = compress_snapshot_if_needed(&snapshot_path) {
                    eprintln!(
                        "Warning: Failed to compress snapshot {}: {}",
                        snapshot_path.display(),
                        e
                    );
                }
                Ok(())
            } else {
                // Mismatch! Create .snap.new using insta format and diff files
                create_snapshot_mismatch_files(
                    snapshot_name,
                    value,
                    &snapshot_path,
                    expected_content,
                )
            }
        }
        Err(_) => {
            // No existing snapshot - create a new one using insta
            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_path(&snapshot_dir);
            settings.bind(|| {
                insta::assert_snapshot!(snapshot_name, value);
            });
            // Compress the newly created snapshot
            if let Err(e) = compress_snapshot_if_needed(&snapshot_path) {
                eprintln!(
                    "Warning: Failed to compress snapshot {}: {}",
                    snapshot_path.display(),
                    e
                );
            }
            Ok(())
        }
    }
}

/// Custom assertion that works with gzipped snapshots.
/// Works like `insta::assert_snapshot!` but reads from `.snap.gz` files.
/// Panics on mismatch - use `check_snapshot_gz` if you need to collect multiple failures.
///
/// The `snapshot_name` should be the full name as it appears in the snapshot file
/// (e.g., "sas_walker_tests__loop_walker_output" for file
/// "sas_walker_tests__loop_walker_output.snap.gz")
pub fn assert_snapshot_gz(snapshot_name: &str, value: &str) {
    if let Err(mismatch) = check_snapshot_gz(snapshot_name, value) {
        panic!(
            "Snapshot mismatch for '{}':\n\nDiff available at: {}\nNew snapshot at: {}\n\nRun ./scripts/insta-accept.sh to accept the new snapshot.\n",
            mismatch.snapshot_name,
            mismatch.diff_path.display(),
            mismatch.new_path.display(),
        );
    }
}

/// Extract the actual content from an insta snapshot (skip the metadata header)
fn extract_snapshot_content(snapshot: &str) -> &str {
    // Find the end of the metadata section (marked by "---" on its own line)
    if let Some(pos) = snapshot.find("\n---\n") {
        &snapshot[pos + 5..] // Skip past "\n---\n"
    } else {
        snapshot
    }
}

/// Create a unified diff between expected and actual content using the `similar` crate
fn create_diff(expected: &str, actual: &str, snapshot_name: &str) -> String {
    let diff = TextDiff::from_lines(expected, actual);
    let mut output = String::new();

    output.push_str(&format!("--- {}.snap (expected)\n", snapshot_name));
    output.push_str(&format!("+++ {}.snap (actual)\n", snapshot_name));

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            output.push_str("...\n");
        }

        for op in group {
            for change in diff.iter_inline_changes(op) {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };

                output.push_str(sign);
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        output.push_str(&format!("«{}»", value));
                    } else {
                        output.push_str(&value);
                    }
                }
                if change.missing_newline() {
                    output.push_str("\n\\ No newline at end of file\n");
                }
            }
        }
    }

    if output.lines().count() <= 2 {
        output.push_str("No differences found (this shouldn't happen)\n");
    }

    output
}

/// Create snapshot mismatch files (.snap.new and .snap.diff) using insta's format
fn create_snapshot_mismatch_files(
    snapshot_name: &str,
    actual_value: &str,
    snapshot_path: &std::path::Path,
    expected_content: &str,
) -> Result<(), SnapshotMismatch> {
    let diff_path = snapshot_path.with_extension("snap.diff");
    let new_path = snapshot_path.with_extension("snap.new");

    // Create diff file
    let diff_content = create_diff(expected_content, actual_value, snapshot_name);
    if let Err(e) = std::fs::write(&diff_path, &diff_content) {
        eprintln!(
            "Warning: Could not write diff to {}: {}",
            diff_path.display(),
            e
        );
    } else {
        eprintln!("Diff written to: {}", diff_path.display());
    }

    // Create .snap.new file using insta's format by calling insta in a temporary location
    // then copying the result. This ensures we get the same format insta would create.
    let temp_dir = std::env::temp_dir().join("insta_temp_snapshots");
    std::fs::create_dir_all(&temp_dir).ok();

    let temp_snapshot_path = temp_dir.join(format!("{}.snap", snapshot_name));

    // Remove any existing temp file
    let _ = std::fs::remove_file(&temp_snapshot_path);

    // Use insta to create the snapshot in temp location, catch any panic
    let _result = std::panic::catch_unwind(|| {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(&temp_dir);
        settings.bind(|| {
            insta::assert_snapshot!(snapshot_name, actual_value);
        });
    });

    // Whether insta succeeded or failed, it should have created a .snap file
    if temp_snapshot_path.exists() {
        // Copy the insta-created snapshot to our .snap.new location
        if let Err(e) = std::fs::copy(&temp_snapshot_path, &new_path) {
            eprintln!(
                "Warning: Could not copy snapshot to {}: {}",
                new_path.display(),
                e
            );
        } else {
            eprintln!("New snapshot written to: {}", new_path.display());
        }
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_snapshot_path);
    } else {
        eprintln!("Warning: insta failed to create temporary snapshot");
    }

    Err(SnapshotMismatch {
        snapshot_name: snapshot_name.to_string(),
        diff_path,
        new_path,
    })
}

/// Compress a snapshot file to .gz format if it exists and isn't already compressed
fn compress_snapshot_if_needed(
    snapshot_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let gz_path = snapshot_path.with_extension("snap.gz");

    // If .snap exists but .snap.gz doesn't, compress it
    if snapshot_path.exists() && !gz_path.exists() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::fs::File;
        use std::io::Write;

        let input = std::fs::read_to_string(snapshot_path)?;
        let output = File::create(&gz_path)?;
        let mut encoder = GzEncoder::new(output, Compression::default());
        encoder.write_all(input.as_bytes())?;
        encoder.finish()?;

        // Remove the uncompressed version
        std::fs::remove_file(snapshot_path)?;
    }
    Ok(())
}

/// Get the snapshot directory for the current package.
/// Uses CARGO_MANIFEST_DIR which Cargo sets to the package being tested.
///
/// # Panics
/// Panics with a helpful message if CARGO_MANIFEST_DIR is not set, which
/// indicates the code is being run outside of Cargo (e.g., direct binary execution).
fn get_snapshot_dir() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        panic!(
            "CARGO_MANIFEST_DIR environment variable not found!\n\
            \n\
            This usually means you're running tests outside of Cargo.\n\
            \n\
            Solutions:\n\
            - Run tests with: cargo test\n\
            - If running from IDE, ensure it uses Cargo to run tests\n\
            - If running binary directly, set CARGO_MANIFEST_DIR manually\n\
            \n\
            CARGO_MANIFEST_DIR should point to the directory containing Cargo.toml"
        );
    });

    std::path::Path::new(&manifest_dir)
        .join("tests")
        .join("snapshots")
}
