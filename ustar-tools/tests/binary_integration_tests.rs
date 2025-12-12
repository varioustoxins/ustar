// Smoke tests to verify CLI binaries work after dependency reorganization
// Note: ustar-dumper is already tested comprehensively in ustar_dumper_tests.rs

use std::process::Command;
use ustar_test_utils::assert_snapshot_gz;

// Simple smoke tests to verify the binaries execute without errors

/// Helper function to test download functionality using mocked data sources
fn test_download_functionality_with_mocks<F>(downloader_type: &str, file_ext: &str, test_fn: F)
where
    F: FnOnce(
        &std::path::Path,
    ) -> Result<
        Vec<(String, std::path::PathBuf)>,
        ustar_tools::downloader_common::DownloadError,
    >,
{
    // Create temporary directory
    let temp_dir = std::env::temp_dir().join(format!("mock_{}_test", downloader_type));
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).expect("Failed to clean temp directory");
    }
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    // Run the test function
    let batch = test_fn(&temp_dir).expect("Download should succeed with mock data");

    // Verify results
    assert_eq!(batch.len(), 2, "Should download exactly 2 files");

    // Verify files exist and have correct extensions
    for (id, path) in &batch {
        assert!(
            path.exists(),
            "Downloaded file {} should exist at {:?}",
            id,
            path
        );
        assert!(
            path.extension().unwrap().to_str().unwrap() == file_ext,
            "File should have {} extension",
            file_ext
        );

        let content = std::fs::read_to_string(path).unwrap();
        assert!(!content.is_empty(), "File should not be empty");
        assert!(
            content.contains("data_"),
            "File should contain STAR/CIF data"
        );
    }

    // Verify no duplicates
    let ids: Vec<&String> = batch.iter().map(|(id, _)| id).collect();
    assert_ne!(ids[0], ids[1], "Should download different files");

    // Clean up
    std::fs::remove_dir_all(&temp_dir).expect("Failed to clean up temp directory");
}

#[test]
fn test_binaries_help_commands() {
    use std::path::Path;

    // Build all binaries first
    println!("Building binaries...");
    let build_output = Command::new("cargo")
        .args(&["build", "--bins"])
        .output()
        .expect("Failed to build binaries");

    if !build_output.status.success() {
        panic!(
            "Failed to build binaries: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    let binaries_with_help = [
        "ustar-benchmark",
        "ustar-parse-debugger",
        "download-bmrb-stars",
        "download-cod-cifs",
        "download-pdbs",
        "ustar-grammar-railroad",
        "sas-demo",
    ];

    // Find the target directory
    let target_dir = Path::new("../target/debug");

    for binary in &binaries_with_help {
        println!("Testing {} --help...", binary);

        let binary_path = target_dir.join(binary);
        if !binary_path.exists() {
            panic!("Binary {} not found at {:?}", binary, binary_path);
        }

        let output = Command::new(&binary_path)
            .arg("--help")
            .output()
            .unwrap_or_else(|_| panic!("Failed to run {} --help", binary));

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("{} --help failed: {}", binary, stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.is_empty(), "{} help should produce output", binary);
        assert!(
            stdout.contains("Usage") || stdout.contains("USAGE") || stdout.contains("Arguments"),
            "{} help should contain usage information",
            binary
        );

        println!("✓ {} --help works correctly", binary);
    }
}

#[test]
fn test_sas_demo_basic_execution() {
    use std::path::Path;

    // Build the binary first
    let build_output = Command::new("cargo")
        .args(&["build", "--bin", "sas-demo"])
        .output()
        .expect("Failed to build sas-demo");

    if !build_output.status.success() {
        panic!(
            "Failed to build sas-demo: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    let test_file = "../ustar-parser/tests/test_data/comprehensive_example.star";
    let binary_path = Path::new("../target/debug/sas-demo");

    let output = Command::new(binary_path)
        .arg(test_file)
        .output()
        .expect("Failed to run sas-demo");

    assert!(
        output.status.success(),
        "sas-demo should execute successfully"
    );

    let stdout = String::from_utf8(output.stdout).expect("Failed to parse stdout");
    assert!(!stdout.is_empty(), "sas-demo should produce output");
    assert!(
        stdout.contains("<start_stream>"),
        "sas-demo should show SAS stream events"
    );

    // Use snapshot testing for consistent output verification
    assert_snapshot_gz(
        "binary_integration_tests__sas_demo_comprehensive_example",
        &stdout,
    );
}

#[test]
fn test_download_pdbs_basic_functionality() {
    test_download_functionality_with_mocks("pdb", "cif", |temp_dir| {
        // Create mock PDB data source
        use std::sync::Arc;
        use ustar_test_utils::MockHttpClient;
        use ustar_tools::downloader_common::{
            DataSource, DownloadError, DownloaderConfig, GenericDownloader,
        };

        // Simple mock PDB implementation
        struct MockPdbSource {
            http_client: Arc<MockHttpClient>,
        }

        impl MockPdbSource {
            fn new() -> Self {
                let client = Arc::new(
                    MockHttpClient::new()
                        .with_response(
                            "https://files.rcsb.org/pub/pdb/holdings/current_holdings.txt",
                            "1abc\n2def\n3ghi",
                        )
                        .with_response(
                            "https://files.rcsb.org/download/1abc.cif",
                            "data_1ABC\n_entry.id 1ABC\n",
                        )
                        .with_response(
                            "https://files.rcsb.org/download/2def.cif",
                            "data_2DEF\n_entry.id 2DEF\n",
                        ),
                );
                Self {
                    http_client: client,
                }
            }
        }

        impl DataSource for MockPdbSource {
            fn get_available_entries(&self) -> Result<Vec<String>, DownloadError> {
                let text = self
                    .http_client
                    .get("https://files.rcsb.org/pub/pdb/holdings/current_holdings.txt")?;
                Ok(text
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| l.to_lowercase())
                    .collect())
            }

            fn download_entry(
                &self,
                entry_id: &str,
                output_path: &std::path::PathBuf,
            ) -> Result<std::path::PathBuf, DownloadError> {
                let url = format!("https://files.rcsb.org/download/{}.cif", entry_id);
                let content = self.http_client.get(&url)?;
                std::fs::create_dir_all(output_path.parent().unwrap())?;
                std::fs::write(output_path, content)?;
                Ok(output_path.clone())
            }
        }

        let config = DownloaderConfig::new()
            .output_dir(temp_dir.to_str().unwrap())
            .verbose(false)
            .file_extension("cif");

        let downloader = GenericDownloader::new(config, MockPdbSource::new());
        downloader.download_unique_random_batch(2, 42)
    });
}

#[test]
fn test_download_bmrb_stars_basic_functionality() {
    test_download_functionality_with_mocks("bmrb", "str", |temp_dir| {
        // Create mock BMRB data source
        use std::sync::Arc;
        use ustar_test_utils::MockHttpClient;
        use ustar_tools::downloader_common::{
            DataSource, DownloadError, DownloaderConfig, GenericDownloader,
        };

        // Simple mock BMRB implementation
        struct MockBmrbSource {
            http_client: Arc<MockHttpClient>,
        }

        impl MockBmrbSource {
            fn new() -> Self {
                let client = Arc::new(
                    MockHttpClient::new()
                        .with_response(
                            "https://bmrb.io/ftp/pub/bmrb/entry_directories/",
                            "<html><a href=\"bmr1000/\">bmr1000/</a><a href=\"bmr2000/\">bmr2000/</a></html>"
                        )
                        .with_response(
                            "https://bmrb.io/ftp/pub/bmrb/entry_directories/bmr1000/bmr1000.str",
                            "data_1000\nsave_entry_information\n_Entry.ID 1000\nsave_"
                        )
                        .with_response(
                            "https://bmrb.io/ftp/pub/bmrb/entry_directories/bmr2000/bmr2000.str",
                            "data_2000\nsave_entry_information\n_Entry.ID 2000\nsave_"
                        )
                );
                Self {
                    http_client: client,
                }
            }
        }

        impl DataSource for MockBmrbSource {
            fn get_available_entries(&self) -> Result<Vec<String>, DownloadError> {
                let html = self
                    .http_client
                    .get("https://bmrb.io/ftp/pub/bmrb/entry_directories/")?;
                let mut entries = Vec::new();
                for cap in html.match_indices("bmr") {
                    let start = cap.0;
                    let rest = &html[start..];
                    if let Some(end) = rest.find('/') {
                        let dir = &rest[..end];
                        if dir.len() > 3 && dir[3..].chars().all(|c| c.is_ascii_digit()) {
                            entries.push(dir.to_string());
                        }
                    }
                }
                Ok(entries)
            }

            fn download_entry(
                &self,
                entry_id: &str,
                output_path: &std::path::PathBuf,
            ) -> Result<std::path::PathBuf, DownloadError> {
                let url = format!(
                    "https://bmrb.io/ftp/pub/bmrb/entry_directories/{}/{}.str",
                    entry_id, entry_id
                );
                let content = self.http_client.get(&url)?;
                std::fs::create_dir_all(output_path.parent().unwrap())?;
                std::fs::write(output_path, content)?;
                Ok(output_path.clone())
            }
        }

        let config = DownloaderConfig::new()
            .output_dir(temp_dir.to_str().unwrap())
            .verbose(false)
            .file_extension("str");

        let downloader = GenericDownloader::new(config, MockBmrbSource::new());
        downloader.download_unique_random_batch(2, 42)
    });
}

#[test]
fn test_download_cod_cifs_basic_functionality() {
    test_download_functionality_with_mocks("cod", "cif", |temp_dir| {
        // Create mock COD data source
        use std::sync::Arc;
        use ustar_test_utils::MockHttpClient;
        use ustar_tools::downloader_common::{
            DataSource, DownloadError, DownloaderConfig, GenericDownloader,
        };

        // Simple mock COD implementation
        struct MockCodSource {
            http_client: Arc<MockHttpClient>,
        }

        impl MockCodSource {
            fn new() -> Self {
                let client = Arc::new(
                    MockHttpClient::new()
                        .with_response(
                            "http://www.crystallography.net/cod/result.php?start=1&stop=50000&selection=id",
                            "<html><a href=\"/cod/1000001.cif\">1000001</a><a href=\"/cod/2000002.cif\">2000002</a></html>"
                        )
                        .with_response(
                            "http://www.crystallography.net/cod/1000001.cif",
                            "data_1000001\n_database_code_COD 1000001\n"
                        )
                        .with_response(
                            "http://www.crystallography.net/cod/2000002.cif",
                            "data_2000002\n_database_code_COD 2000002\n"
                        )
                );
                Self {
                    http_client: client,
                }
            }
        }

        impl DataSource for MockCodSource {
            fn get_available_entries(&self) -> Result<Vec<String>, DownloadError> {
                let html = self.http_client.get(
                    "http://www.crystallography.net/cod/result.php?start=1&stop=50000&selection=id",
                )?;
                let mut entries = Vec::new();
                for cap in html.match_indices("/cod/") {
                    let start = cap.0;
                    let rest = &html[start + 5..];
                    if let Some(end) = rest.find('.') {
                        let id = &rest[..end];
                        if id.chars().all(|c| c.is_ascii_digit()) {
                            entries.push(id.to_string());
                        }
                    }
                }
                Ok(entries)
            }

            fn download_entry(
                &self,
                entry_id: &str,
                output_path: &std::path::PathBuf,
            ) -> Result<std::path::PathBuf, DownloadError> {
                let url = format!("http://www.crystallography.net/cod/{}.cif", entry_id);
                let content = self.http_client.get(&url)?;
                std::fs::create_dir_all(output_path.parent().unwrap())?;
                std::fs::write(output_path, content)?;
                Ok(output_path.clone())
            }
        }

        let config = DownloaderConfig::new()
            .output_dir(temp_dir.to_str().unwrap())
            .verbose(false)
            .file_extension("cif");

        let downloader = GenericDownloader::new(config, MockCodSource::new());
        downloader.download_unique_random_batch(2, 42)
    });
}

#[test]
fn test_ustar_parse_debugger_invalid_file() {
    use std::path::Path;

    // Build the binary first
    let build_output = Command::new("cargo")
        .args(&["build", "--bin", "ustar-parse-debugger"])
        .output()
        .expect("Failed to build ustar-parse-debugger");

    if !build_output.status.success() {
        panic!(
            "Failed to build ustar-parse-debugger: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    let binary_path = Path::new("../target/debug/ustar-parse-debugger");
    let output = Command::new(binary_path)
        .arg("tests/test_data/invalid_syntax.star")
        .output()
        .expect("Failed to run ustar-parse-debugger");

    // The debugger should handle invalid syntax gracefully and provide diagnostic output
    assert!(
        output.status.success(),
        "ustar-parse-debugger should handle invalid syntax gracefully"
    );

    let stdout = String::from_utf8(output.stdout).expect("Failed to parse stdout");
    assert!(!stdout.is_empty(), "Should produce diagnostic output");

    // Should contain debugging information
    assert!(
        stdout.contains("Error") || stdout.contains("position") || stdout.contains("line"),
        "Should contain debugging information about parse failure"
    );

    assert_snapshot_gz(
        "binary_integration_tests__ustar_parse_debugger_invalid_file",
        &stdout,
    );
}

#[test]
fn test_ustar_grammar_railroad_svg_generation() {
    use std::fs;
    use std::path::Path;

    // Create a temporary directory for output
    let temp_dir = std::env::temp_dir().join("test_grammar_railroad");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to clean temp directory");
    }
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    // Test all three grammar files
    let grammar_files = [
        "../ustar-parser/src/star_ascii.pest",
        "../ustar-parser/src/star_extended.pest",
        "../ustar-parser/src/star_unicode.pest",
    ];

    for (i, grammar_file) in grammar_files.iter().enumerate() {
        if !Path::new(grammar_file).exists() {
            continue; // Skip if grammar file doesn't exist
        }

        let output_svg = temp_dir.join(format!("test_grammar_{}.svg", i));

        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "ustar-grammar-railroad",
                "--",
                grammar_file,
                "--output",
                output_svg.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to run ustar-grammar-railroad");

        assert!(
            output.status.success(),
            "ustar-grammar-railroad should execute successfully for {}",
            grammar_file
        );

        // Check that SVG file was created
        assert!(
            output_svg.exists(),
            "Should create SVG output file for {}",
            grammar_file
        );

        // Check SVG file size (should be reasonable for a grammar diagram)
        let metadata = fs::metadata(&output_svg).expect("Failed to read SVG metadata");
        assert!(
            metadata.len() > 1000,
            "SVG file should be substantial (>1KB) for {}",
            grammar_file
        );
        assert!(
            metadata.len() < 50_000_000,
            "SVG file shouldn't be too large (<50MB) for {}",
            grammar_file
        );

        // Read and validate the SVG content using usvg for proper SVG validation
        let svg_content = fs::read_to_string(&output_svg).expect("Failed to read SVG file");

        // Basic XML structure checks
        assert!(
            svg_content.contains("<svg"),
            "Should contain SVG root element for {}",
            grammar_file
        );
        assert!(
            svg_content.contains("</svg>"),
            "Should have closing SVG tag for {}",
            grammar_file
        );

        // Use usvg to validate SVG semantics and structure
        let tree = usvg::Tree::from_str(&svg_content, &usvg::Options::default()).expect(&format!(
            "Generated SVG should be valid and parseable by usvg for {}",
            grammar_file
        ));

        // Verify the SVG has some actual content (not just empty)
        let svg_node = tree.root();
        assert!(
            svg_node.has_children(),
            "SVG should contain graphical elements for {}",
            grammar_file
        );

        // Check that the SVG has reasonable dimensions
        let size = tree.size();
        assert!(
            size.width() > 0.0,
            "SVG should have positive width for {}",
            grammar_file
        );
        assert!(
            size.height() > 0.0,
            "SVG should have positive height for {}",
            grammar_file
        );
        assert!(
            size.width() < 100000.0,
            "SVG width should be reasonable (<100000px) for {}",
            grammar_file
        );
        assert!(
            size.height() < 100000.0,
            "SVG height should be reasonable (<100000px) for {}",
            grammar_file
        );
    }

    // Clean up
    fs::remove_dir_all(&temp_dir).expect("Failed to clean up temp directory");
}
