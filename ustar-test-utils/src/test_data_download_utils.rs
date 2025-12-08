//! Test data download utilities for ustar parser.
//!
//! This module is provided because size limits prevent the complete project
//! from being stored on crates.io. Therefore if test data files are missing
//! when running cargo test the missing files will be automatically downloaded
//! from the GitHub repository.
//!
//! This module provides functionality to:
//! - Automatically discover test directories with checksums.sha1 files
//! - Verify test data integrity using SHA-1 checksums  
//! - Download missing test data files from GitHub when needed
//! - Ensure test data is available before running tests
//! - Support for disabling downloads via --features no-large-tests

use sha1::{Digest, Sha1};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::sync::OnceLock;

/// Error type for test data operations
#[derive(Debug)]
pub enum TestDataError {
    /// IO error during file operations
    Io(std::io::Error),
    /// Checksum verification failed
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },
    /// Checksum file is missing or invalid
    InvalidChecksumFile(String),
    /// Test data directory not found
    DirectoryNotFound(String),
}

impl std::fmt::Display for TestDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestDataError::Io(e) => write!(f, "IO error: {}", e),
            TestDataError::ChecksumMismatch {
                file,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Checksum mismatch for {}: expected {}, got {}",
                    file, expected, actual
                )
            }
            TestDataError::InvalidChecksumFile(path) => {
                write!(f, "Invalid or missing checksum file: {}", path)
            }
            TestDataError::DirectoryNotFound(path) => {
                write!(f, "Test data directory not found: {}", path)
            }
        }
    }
}

impl std::error::Error for TestDataError {}

impl From<std::io::Error> for TestDataError {
    fn from(error: std::io::Error) -> Self {
        TestDataError::Io(error)
    }
}

/// Calculate SHA-1 hash of a file
fn calculate_file_sha1<P: AsRef<Path>>(file_path: P) -> Result<String, TestDataError> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha1::new();
    std::io::copy(&mut file, &mut hasher)?;
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Parse a single line from checksums.sha1 file
fn parse_checksum_line(line: &str) -> Option<(String, String)> {
    // Expected format: "hash  filename"
    let parts: Vec<&str> = line.splitn(2, "  ").collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

/// Verify checksums for all files in a test data directory
///
/// Reads the `checksums.sha1` file and verifies each file listed.
/// Uses the Rust `sha1` crate for reliable cross-platform verification.
pub fn verify_test_data_checksums<P: AsRef<Path>>(test_data_dir: P) -> Result<(), TestDataError> {
    let test_data_dir = test_data_dir.as_ref();

    if !test_data_dir.exists() {
        return Err(TestDataError::DirectoryNotFound(
            test_data_dir.display().to_string(),
        ));
    }

    let checksum_file = test_data_dir.join("checksums.sha1");
    if !checksum_file.exists() {
        return Err(TestDataError::InvalidChecksumFile(
            checksum_file.display().to_string(),
        ));
    }

    let checksum_content = fs::read_to_string(&checksum_file)?;

    for (line_num, line) in checksum_content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (expected_hash, filename) = parse_checksum_line(line).ok_or_else(|| {
            TestDataError::InvalidChecksumFile(format!(
                "Invalid checksum format at line {} in {}",
                line_num + 1,
                checksum_file.display()
            ))
        })?;

        let file_path = test_data_dir.join(&filename);
        if !file_path.exists() {
            return Err(TestDataError::DirectoryNotFound(format!(
                "Referenced file not found: {}",
                filename
            )));
        }

        let actual_hash = calculate_file_sha1(&file_path)?;
        if expected_hash != actual_hash {
            return Err(TestDataError::ChecksumMismatch {
                file: filename,
                expected: expected_hash,
                actual: actual_hash,
            });
        }
    }

    Ok(())
}

/// Get the list of expected files from checksums.sha1
fn get_expected_files<P: AsRef<Path>>(test_data_dir: P) -> Result<Vec<String>, TestDataError> {
    let checksum_file = test_data_dir.as_ref().join("checksums.sha1");
    let content = fs::read_to_string(&checksum_file)?;

    let mut files = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((_, filename)) = parse_checksum_line(line) {
            files.push(filename);
        }
    }

    Ok(files)
}

/// Check which files are missing from a test data directory
fn get_missing_files<P: AsRef<Path>>(test_data_dir: P) -> Result<Vec<String>, TestDataError> {
    let test_data_dir = test_data_dir.as_ref();
    let expected_files = get_expected_files(test_data_dir)?;

    let missing_files: Vec<String> = expected_files
        .into_iter()
        .filter(|filename| !test_data_dir.join(filename).exists())
        .collect();

    Ok(missing_files)
}

/// Discover all test data directories that have checksums.sha1 files
fn discover_test_data_directories<P: AsRef<Path>>(
    base_dir: P,
) -> Result<Vec<std::path::PathBuf>, TestDataError> {
    let base_dir = base_dir.as_ref();

    if !base_dir.exists() {
        return Ok(Vec::new());
    }

    let mut directories = Vec::new();

    // Walk through immediate subdirectories
    let read_dir = fs::read_dir(base_dir).map_err(TestDataError::Io)?;

    for entry in read_dir {
        let entry = entry.map_err(TestDataError::Io)?;
        let path = entry.path();

        if path.is_dir() {
            let checksum_file = path.join("checksums.sha1");
            if checksum_file.exists() {
                directories.push(path);
            }
        }
    }

    Ok(directories)
}

/// Ensure test data is available and verified
///
/// This function:
/// 1. Dynamically discovers test data directories with checksums.sha1 files
/// 2. Checks all discovered directories for missing files
/// 3. Downloads missing data if needed (unless disabled)
/// 4. Verifies checksums of all files
///
/// Can be called with either a specific directory or a base directory to scan
pub fn ensure_test_data_available<P: AsRef<Path>>(path: P) -> Result<(), TestDataError> {
    let path = path.as_ref();

    // Determine if this is a specific directory or base directory to scan
    let (_base_dir, specific_dirs) = if path.join("checksums.sha1").exists() {
        // This is a specific test data directory
        (path.parent().unwrap_or(path), vec![path.to_path_buf()])
    } else {
        // This is a base directory - discover all test data directories
        let discovered = discover_test_data_directories(path)?;
        (path, discovered)
    };

    if specific_dirs.is_empty() {
        return Ok(()); // No test data directories found, nothing to do
    }

    // Check all directories for missing files
    let mut all_missing_files = Vec::new();
    let mut dirs_with_missing = Vec::new();

    for dir in &specific_dirs {
        let missing_files = get_missing_files(dir)?;
        if !missing_files.is_empty() {
            println!(
                "Missing files in {}: {}",
                dir.file_name().unwrap_or_default().to_string_lossy(),
                missing_files.join(", ")
            );
            all_missing_files.extend(missing_files.iter().cloned());
            dirs_with_missing.push(dir.clone());
        }
    }

    // If any files are missing, attempt download
    if !all_missing_files.is_empty() {
        if !cfg!(feature = "no-large-tests") {
            println!("Attempting to download test data from GitHub...");

            match download_test_data_from_github() {
                Ok(_) => {
                    println!("Test data download completed successfully!");

                    // Re-check all directories that had missing files
                    for dir in &dirs_with_missing {
                        let still_missing = get_missing_files(dir)?;
                        if !still_missing.is_empty() {
                            return Err(TestDataError::DirectoryNotFound(format!(
                                "Download completed but still missing files in {}: {}",
                                dir.display(),
                                still_missing.join(", ")
                            )));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: Failed to download test data: {}", e);
                    eprintln!("To skip large tests, run: cargo test --features no-large-tests");
                    eprintln!("To download manually:");
                    eprintln!("  git clone --depth=1 https://github.com/varioustoxins/ustar.git temp_ustar");
                    eprintln!("  cp -r temp_ustar/ustar-parser/tests/test_data/* <your-project>/tests/test_data/");
                    eprintln!("  rm -rf temp_ustar");

                    return Err(TestDataError::DirectoryNotFound(format!(
                        "Missing test data files and download failed: {}",
                        e
                    )));
                }
            }
        } else {
            return Err(TestDataError::DirectoryNotFound(format!(
                "Missing test data files: {}. Download disabled by --features no-large-tests.",
                all_missing_files.join(", ")
            )));
        }
    }

    // Verify checksums of all discovered directories
    for dir in &specific_dirs {
        verify_test_data_checksums(dir)?;
    }

    Ok(())
}

static DOWNLOAD_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

/// Download missing test data from GitHub repository
/// Uses sparse checkout to only download test data directories
fn download_test_data_from_github() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure download only happens once per test run using OnceLock
    let result = DOWNLOAD_RESULT.get_or_init(|| perform_download().map_err(|e| e.to_string()));

    // Return the cached result
    result.clone().map_err(|e| e.into())
}

/// Perform the actual test data download from GitHub
fn perform_download() -> Result<(), Box<dyn std::error::Error>> {
    // Use tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(download_github_archive())
}

/// Download and extract test data from GitHub ZIP archive
async fn download_github_archive() -> Result<(), Box<dyn std::error::Error>> {
    let archive_url = "https://github.com/varioustoxins/ustar/archive/refs/heads/main.zip";

    println!("Downloading repository archive from GitHub...");

    // Download the ZIP archive directly to memory (no temporary files needed)
    let response = reqwest::get(archive_url).await?;
    if !response.status().is_success() {
        return Err(format!("Failed to download archive: HTTP {}", response.status()).into());
    }

    let zip_bytes = response.bytes().await?;
    println!(
        "Downloaded {} bytes, extracting test data...",
        zip_bytes.len()
    );

    // Extract test data from ZIP
    let cursor = Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;

    // Determine current package root directory for target
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let target_base = format!("{}/tests/test_data", manifest_dir);

    // Create target directory if it doesn't exist
    fs::create_dir_all(&target_base)?;

    let mut extracted_files = 0;
    let mut extracted_dirs = std::collections::HashSet::new();

    // Extract files from the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = file.name();

        // Only extract files from ustar-main/ustar-parser/tests/test_data/
        if let Some(relative_path) = extract_test_data_path(file_path) {
            let target_path = format!("{}/{}", target_base, relative_path);

            // Track which directories we're extracting
            if let Some(dir) = relative_path.split('/').next() {
                if extracted_dirs.insert(dir.to_string()) {
                    println!("Extracting {}...", dir);
                }
            }

            // Create parent directories
            if let Some(parent) = Path::new(&target_path).parent() {
                fs::create_dir_all(parent)?;
            }

            // Extract the file
            let mut target_file = fs::File::create(&target_path)?;
            std::io::copy(&mut file, &mut target_file)?;
            extracted_files += 1;
        }
    }

    println!(
        "Extracted {} files from {} directories",
        extracted_files,
        extracted_dirs.len()
    );
    Ok(())
}

/// Extract the test data relative path from a ZIP archive path
/// Converts "ustar-main/ustar-parser/tests/test_data/nef_spec/file.nef"
/// to "nef_spec/file.nef"
fn extract_test_data_path(archive_path: &str) -> Option<String> {
    // Look for the test_data directory in the path
    if let Some(test_data_pos) = archive_path.find("/tests/test_data/") {
        let after_test_data = &archive_path[test_data_pos + "/tests/test_data/".len()..];
        if !after_test_data.is_empty() && !after_test_data.ends_with('/') {
            return Some(after_test_data.to_string());
        }
    }
    None
}

/// Get OS-provided temporary directory for future use
/// Currently not needed as we download directly to memory, but useful for large archives
#[allow(dead_code)]
fn get_temp_dir() -> std::path::PathBuf {
    std::env::temp_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_expected_files() {
        let temp_dir = TempDir::new().unwrap();
        let checksum_content = "abc123  file1.txt\ndef456  file2.txt\n";
        fs::write(temp_dir.path().join("checksums.sha1"), checksum_content).unwrap();

        let files = get_expected_files(temp_dir.path()).unwrap();
        assert_eq!(files, vec!["file1.txt", "file2.txt"]);
    }

    #[test]
    fn test_missing_files_detection() {
        let temp_dir = TempDir::new().unwrap();
        let checksum_content = "abc123  file1.txt\ndef456  file2.txt\n";
        fs::write(temp_dir.path().join("checksums.sha1"), checksum_content).unwrap();

        // Create only one file
        fs::write(temp_dir.path().join("file1.txt"), "content").unwrap();

        let missing = get_missing_files(temp_dir.path()).unwrap();
        assert_eq!(missing, vec!["file2.txt"]);
    }

    #[test]
    fn test_sha1_verification_success() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test file
        let test_content = "Hello, world!";
        fs::write(temp_dir.path().join("test.txt"), test_content).unwrap();

        // Calculate the expected SHA-1 hash for "Hello, world!"
        let expected_hash = "943a702d06f34599aee1f8da8ef9f7296031d699";

        let checksum_content = format!("{}  test.txt\n", expected_hash);
        fs::write(temp_dir.path().join("checksums.sha1"), checksum_content).unwrap();

        // Should pass verification
        let result = verify_test_data_checksums(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_sha1_verification_failure() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test file
        fs::write(temp_dir.path().join("test.txt"), "Hello, world!").unwrap();

        // Use wrong hash
        let wrong_hash = "0000000000000000000000000000000000000000";
        let checksum_content = format!("{}  test.txt\n", wrong_hash);
        fs::write(temp_dir.path().join("checksums.sha1"), checksum_content).unwrap();

        // Should fail verification
        let result = verify_test_data_checksums(temp_dir.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            TestDataError::ChecksumMismatch {
                file,
                expected,
                actual,
            } => {
                assert_eq!(file, "test.txt");
                assert_eq!(expected, wrong_hash);
                assert_eq!(actual, "943a702d06f34599aee1f8da8ef9f7296031d699");
            }
            _ => panic!("Expected ChecksumMismatch error"),
        }
    }

    #[test]
    fn test_parse_checksum_line() {
        assert_eq!(
            parse_checksum_line("abc123  file.txt"),
            Some(("abc123".to_string(), "file.txt".to_string()))
        );

        assert_eq!(parse_checksum_line("invalid line"), None);
    }

    #[test]
    fn test_discover_test_data_directories() {
        let temp_dir = TempDir::new().unwrap();

        // Create test structure with some directories that have checksums.sha1 and some that don't
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = temp_dir.path().join("dir2");
        let dir3 = temp_dir.path().join("dir3");

        fs::create_dir_all(&dir1).unwrap();
        fs::create_dir_all(&dir2).unwrap();
        fs::create_dir_all(&dir3).unwrap();

        // Only dir1 and dir3 have checksums.sha1
        fs::write(dir1.join("checksums.sha1"), "abc123  file1.txt\n").unwrap();
        fs::write(dir3.join("checksums.sha1"), "def456  file3.txt\n").unwrap();

        // dir2 has no checksum file
        fs::write(dir2.join("some_other_file.txt"), "content").unwrap();

        let discovered = discover_test_data_directories(temp_dir.path()).unwrap();

        // Should find dir1 and dir3, but not dir2
        assert_eq!(discovered.len(), 2);

        let dir_names: Vec<String> = discovered
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(dir_names.contains(&"dir1".to_string()));
        assert!(dir_names.contains(&"dir3".to_string()));
        assert!(!dir_names.contains(&"dir2".to_string()));
    }
}
