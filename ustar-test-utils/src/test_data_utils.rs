//! Test data management utilities for ustar parser.
//!
//! This module provides functionality to:
//! - Verify test data integrity using SHA-1 checksums
//! - Download missing test data files when needed
//! - Ensure test data is available before running tests

use sha1::{Digest, Sha1};
use std::fs;
use std::path::Path;

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

/// Ensure test data is available and verified
///
/// This function:
/// 1. Checks if test data directory exists
/// 2. Verifies if all expected files are present
/// 3. Runs checksum verification
/// 4. If files are missing, provides instructions or attempts download (future phase)
///
/// For now, this is a placeholder that will skip tests gracefully when data is missing.
pub fn ensure_test_data_available<P: AsRef<Path>>(test_data_dir: P) -> Result<(), TestDataError> {
    let test_data_dir = test_data_dir.as_ref();

    // Phase 2: Basic checksum verification
    // Check if directory exists
    if !test_data_dir.exists() {
        return Err(TestDataError::DirectoryNotFound(
            test_data_dir.display().to_string(),
        ));
    }

    // Check if checksum file exists
    let checksum_file = test_data_dir.join("checksums.sha1");
    if !checksum_file.exists() {
        return Err(TestDataError::InvalidChecksumFile(
            checksum_file.display().to_string(),
        ));
    }

    // Check for missing files
    let missing_files = get_missing_files(test_data_dir)?;
    if !missing_files.is_empty() {
        // For now, return an error with helpful message
        // In Phase 3, this will trigger download
        return Err(TestDataError::DirectoryNotFound(format!(
            "Missing test data files in {}: {}. Use --features large-tests to enable download.",
            test_data_dir.display(),
            missing_files.join(", ")
        )));
    }

    // Verify checksums of existing files
    verify_test_data_checksums(test_data_dir)?;

    Ok(())
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
}
