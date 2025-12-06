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

                    panic!(
                        "Snapshot mismatch for '{}':\n\nDiff available at: {}\nUncompressed snapshot at: {}\nNew snapshot at: {}\n\nExpected:\n{}\n\nActual:\n{}\n",
                        snapshot_name,
                        diff_path.display(),
                        uncompressed_path.display(),
                        new_path.display(),
                        expected_content,
                        actual_content
                    );
                }
                return; // Success - exit early
            }
            Err(_) => continue,
        }
    }

    // If no existing snapshot found, create new snapshot in the correct calling crate's directory
    // Determine the correct target directory based on the caller
    let pkg_name = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
    let target_dir = if pkg_name == "ustar-parser" || pkg_name == "ustar-tools" {
        std::path::Path::new("tests/snapshots")
    } else {
        // Fall back to trying the possible paths in order
        if possible_paths[0].parent().unwrap().exists() {
            std::path::Path::new("ustar-parser/tests/snapshots")
        } else if possible_paths[1].parent().unwrap().exists() {
            std::path::Path::new("ustar-tools/tests/snapshots")
        } else {
            std::path::Path::new("tests/snapshots")
        }
    };

    // Use insta with explicit settings to control where snapshots are created
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(target_dir);
    settings.bind(|| {
        insta::assert_snapshot!(snapshot_name, value);
    });
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

/// Create a unified diff between expected and actual content
fn create_diff(expected: &str, actual: &str, snapshot_name: &str) -> String {
    let mut diff = String::new();
    diff.push_str(&format!("--- {}.snap (expected)\n", snapshot_name));
    diff.push_str(&format!("+++ {}.snap (actual)\n", snapshot_name));

    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    // Simple line-by-line diff - could be enhanced with proper diff algorithm
    let max_lines = expected_lines.len().max(actual_lines.len());
    let mut context_start: usize = 0;
    let mut in_diff_section = false;

    for i in 0..max_lines {
        let exp_line = expected_lines.get(i).unwrap_or(&"");
        let act_line = actual_lines.get(i).unwrap_or(&"");

        if exp_line != act_line {
            if !in_diff_section {
                // Start a new diff hunk, show some context
                diff.push_str(&format!(
                    "@@ -{},{} +{},{} @@\n",
                    context_start.saturating_sub(3) + 1,
                    expected_lines
                        .len()
                        .saturating_sub(context_start.saturating_sub(3)),
                    context_start.saturating_sub(3) + 1,
                    actual_lines
                        .len()
                        .saturating_sub(context_start.saturating_sub(3))
                ));

                // Add context lines before the diff
                for j in context_start.saturating_sub(3)..i {
                    if let Some(line) = expected_lines.get(j) {
                        diff.push_str(&format!(" {}\n", line));
                    }
                }
                in_diff_section = true;
            }

            if !exp_line.is_empty() {
                diff.push_str(&format!("-{}\n", exp_line));
            }
            if !act_line.is_empty() {
                diff.push_str(&format!("+{}\n", act_line));
            }
        } else if in_diff_section {
            // Add context line in diff section
            diff.push_str(&format!(" {}\n", exp_line));

            // Check if we should end the diff section (look ahead for more differences)
            let mut found_more_diffs = false;
            for j in (i + 1)..(i + 4).min(max_lines) {
                let future_exp = expected_lines.get(j).unwrap_or(&"");
                let future_act = actual_lines.get(j).unwrap_or(&"");
                if future_exp != future_act {
                    found_more_diffs = true;
                    break;
                }
            }

            if !found_more_diffs {
                in_diff_section = false;
                context_start = i + 1;
            }
        }
    }

    if diff.is_empty() {
        diff.push_str("No differences found (this shouldn't happen)\n");
    }

    diff
}
