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
/// Looks for snapshots in `tests/snapshots/` relative to the current directory.
pub fn check_snapshot_gz(snapshot_name: &str, value: &str) -> Result<(), SnapshotMismatch> {
    let filename = format!("{}.snap", snapshot_name);
    let snapshot_path = std::path::Path::new("tests/snapshots").join(&filename);

    match read_snapshot(&snapshot_path) {
        Ok(expected) => {
            // Compare the content - extract just the content part after "---"
            let expected_content = extract_snapshot_content(&expected);
            let actual_content = value;

            if expected_content.trim() != actual_content.trim() {
                // Create an uncompressed version of the snapshot for easier debugging
                let uncompressed_path = snapshot_path.with_extension("snap");
                if let Err(e) = std::fs::write(&uncompressed_path, &expected) {
                    eprintln!(
                        "Warning: Could not write uncompressed snapshot to {}: {}",
                        uncompressed_path.display(),
                        e
                    );
                }

                // Create a diff file to show the differences
                let diff_path = snapshot_path.with_extension("snap.diff");
                let diff_content = create_diff(expected_content, actual_content, snapshot_name);
                if let Err(e) = std::fs::write(&diff_path, &diff_content) {
                    eprintln!(
                        "Warning: Could not write diff to {}: {}",
                        diff_path.display(),
                        e
                    );
                } else {
                    eprintln!("Diff written to: {}", diff_path.display());
                }

                // Create the new snapshot content for review
                let source_file = format!(
                    "{}/tests/{}.rs",
                    std::env::var("CARGO_PKG_NAME").unwrap_or_default(),
                    snapshot_name.split("__").next().unwrap_or("unknown")
                );
                let new_snapshot_content = format!(
                    "---\nsource: {}\nexpression: output\n---\n{}",
                    source_file, actual_content
                );
                let new_path = snapshot_path.with_extension("snap.new");
                if let Err(e) = std::fs::write(&new_path, &new_snapshot_content) {
                    eprintln!(
                        "Warning: Could not write new snapshot to {}: {}",
                        new_path.display(),
                        e
                    );
                } else {
                    eprintln!("New snapshot written to: {}", new_path.display());
                }

                return Err(SnapshotMismatch {
                    snapshot_name: snapshot_name.to_string(),
                    diff_path,
                    new_path,
                });
            }
            Ok(()) // Success
        }
        Err(_) => {
            // No existing snapshot found - create new one using insta
            let target_dir = std::path::Path::new("tests/snapshots");
            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_path(target_dir);
            settings.bind(|| {
                insta::assert_snapshot!(snapshot_name, value);
            });
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
