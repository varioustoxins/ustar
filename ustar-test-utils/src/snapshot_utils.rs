use flate2::read::GzDecoder;
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

/// Custom assertion that works with gzipped snapshots.
/// Works like `insta::assert_snapshot!` but reads from `.snap.gz` files.
///
/// The `snapshot_name` should be the full name as it appears in the snapshot file
/// (e.g., "sas_walker_tests__loop_walker_output" for file
/// "sas_walker_tests__loop_walker_output.snap.gz")
pub fn assert_snapshot_gz(snapshot_name: &str, value: &str) {
    let filename = format!("{}.snap", snapshot_name);
    let possible_paths = [
        // For ustar-parser tests run from workspace root
        std::path::Path::new("ustar-parser/tests/snapshots").join(&filename),
        // For ustar-tools tests run from workspace root
        std::path::Path::new("ustar-tools/tests/snapshots").join(&filename),
        // For tests run from crate directory
        std::path::Path::new("tests/snapshots").join(&filename),
    ];

    // Try to read existing compressed snapshot from any of the paths
    for snapshot_path in &possible_paths {
        match read_snapshot(snapshot_path) {
            Ok(expected) => {
                // Compare the content - extract just the content part after "---"
                let expected_content = extract_snapshot_content(&expected);
                let actual_content = value;

                if expected_content.trim() != actual_content.trim() {
                    panic!(
                        "Snapshot mismatch for '{}':\n\nExpected:\n{}\n\nActual:\n{}\n",
                        snapshot_name, expected_content, actual_content
                    );
                }
                return; // Success - exit early
            }
            Err(_) => continue,
        }
    }

    // If no existing snapshot found, fall back to regular insta to create new snapshot
    insta::assert_snapshot!(snapshot_name, value);
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
