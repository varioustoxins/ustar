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

/// Custom assertion that works with gzipped snapshots
pub fn assert_snapshot_gz(name: &str, value: &str) {
    // Try multiple possible paths where snapshots might be stored
    let snapshot_name = format!("{}.snap", name);
    let possible_paths = [
        std::path::Path::new("tests/snapshots").join(&snapshot_name),
        std::path::Path::new("../tests/snapshots").join(&snapshot_name), // workspace root
        std::path::Path::new("../../tests/snapshots").join(&snapshot_name), // from nested
    ];

    // Try to read existing compressed snapshot from any of the paths
    for snapshot_path in &possible_paths {
        match read_snapshot(&snapshot_path) {
            Ok(expected) => {
                // Compare the content - extract just the content part after "---"
                let expected_content = extract_snapshot_content(&expected);
                let actual_content = value;

                if expected_content.trim() != actual_content.trim() {
                    panic!(
                        "Snapshot mismatch for '{}':\n\nExpected:\n{}\n\nActual:\n{}\n",
                        name, expected_content, actual_content
                    );
                }
                return; // Success - exit early
            }
            Err(_) => continue,
        }
    }

    // If no existing snapshot found in any location, fall back to regular insta
    insta::assert_snapshot!(name, value);
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
