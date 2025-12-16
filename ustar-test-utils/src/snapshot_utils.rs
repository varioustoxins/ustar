use flate2::read::GzDecoder;
use similar::{ChangeTag, TextDiff};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Print message only in verbose mode - controlled by insta settings
macro_rules! verbose_println {
    ($($arg:tt)*) => {
        // Only print during test failures or when explicitly verbose
        // For now, we'll just use this for development and can be controlled via test output
        #[cfg(debug_assertions)]
        eprintln!($($arg)*);
    };
}

/// Read a snapshot file, automatically decompressing if it's gzipped
/// Returns the FULL file content including headers
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

/// Ensure all .snap.gz files have corresponding .snap files with identical content
/// This allows insta to work with uncompressed .snap files while maintaining compressed storage
fn ensure_snapshots_synchronized(snapshot_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Read the directory and find all .snap.gz files
    if !snapshot_dir.exists() {
        return Ok(()); // No snapshot directory yet
    }

    for entry in std::fs::read_dir(snapshot_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process .snap.gz files
        if path.extension() == Some("gz".as_ref()) && path.to_string_lossy().ends_with(".snap.gz") {
            // Determine the corresponding .snap file path
            let snap_path = path.with_extension(""); // Remove .gz extension, leaving .snap

            // Check if we need to decompress
            let should_decompress = !snap_path.exists() || {
                // Compare content to ensure they match
                match (read_snapshot(&snap_path), read_snapshot(&path)) {
                    (Ok(snap_content), Ok(gz_content)) => snap_content != gz_content,
                    _ => true, // If we can't read either file, decompress to be safe
                }
            };

            if should_decompress {
                // Decompress .snap.gz to .snap
                let gz_file = File::open(&path)?;
                let mut decoder = GzDecoder::new(gz_file);
                let mut content = String::new();
                decoder.read_to_string(&mut content)?;

                // Write the decompressed content to .snap file
                std::fs::write(&snap_path, &content)?;

                verbose_println!("Synchronized {} -> {}", path.display(), snap_path.display());
            }
        }
    }

    Ok(())
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

    // Ensure all .snap.gz files are decompressed to .snap files for insta to use
    if let Err(e) = ensure_snapshots_synchronized(&snapshot_dir) {
        eprintln!("Warning: Failed to synchronize snapshots: {}", e);
    }

    // Use insta's actual comparison logic by catching panics
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(&snapshot_dir);
    settings.set_prepend_module_to_snapshot(false);

    let result = std::panic::catch_unwind(|| {
        settings.bind(|| {
            insta::assert_snapshot!(snapshot_name, value);
        });
    });

    match result {
        Ok(_) => {
            // Snapshot matches! No need to do anything - compression handled by acceptance script
            Ok(())
        }
        Err(_) => {
            // Snapshot mismatch or doesn't exist - insta has already created .snap.new
            // Always create diff and old files after insta runs, regardless of the failure path
            create_review_files(&snapshot_path, snapshot_name);

            Err(SnapshotMismatch {
                snapshot_name: snapshot_name.to_string(),
                diff_path: snapshot_path.with_extension("snap.diff"),
                new_path: snapshot_path.with_extension("snap.new"),
            })
        }
    }
}

/// Create .snap.diff and .snap.old files for review after a snapshot mismatch
fn create_review_files(snapshot_path: &std::path::Path, snapshot_name: &str) {
    // With prepend_module_to_snapshot(false), insta uses the snapshot_name directly
    let new_path = snapshot_path.with_extension("snap.new");
    let diff_path = snapshot_path.with_extension("snap.diff");
    let old_path = snapshot_path.with_extension("snap.old");

    // Create .snap.old file (current snapshot decompressed) for easy comparison
    if !old_path.exists() {
        if let Ok(expected_content) = read_snapshot(snapshot_path) {
            if let Err(e) = std::fs::write(&old_path, &expected_content) {
                eprintln!("Failed to write .snap.old file: {}", e);
            } else {
                verbose_println!("Current snapshot saved to: {}", old_path.display());
            }
        }
    }

    // Create diff file if .snap.new exists and we have an existing snapshot
    if new_path.exists() && !diff_path.exists() {
        verbose_println!("Creating diff for {}", snapshot_name);
        if let (Ok(new_content), Ok(expected_content)) = (
            std::fs::read_to_string(&new_path),
            read_snapshot(snapshot_path),
        ) {
            // Don't strip headers - show diff of full files including metadata
            let diff_content = create_diff(&expected_content, &new_content, snapshot_name);
            if let Err(e) = std::fs::write(&diff_path, &diff_content) {
                eprintln!("Failed to write diff file: {}", e);
            } else {
                verbose_println!("Diff written to: {}", diff_path.display());
            }
        } else {
            verbose_println!("Could not read files for diff creation");
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
