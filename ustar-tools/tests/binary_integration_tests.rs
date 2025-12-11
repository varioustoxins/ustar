// Smoke tests to verify CLI binaries work after dependency reorganization
// Note: ustar-dumper is already tested comprehensively in ustar_dumper_tests.rs

use std::process::Command;
use ustar_test_utils::assert_snapshot_gz;

// Simple smoke tests to verify the binaries execute without errors

/// Helper function to test download binaries with reproducible seed-based downloads
fn test_download_binary_functionality(binary_name: &str, temp_dir_name: &str) {
    use std::fs;

    // Create a temporary directory for the test
    let temp_dir = std::env::temp_dir().join(temp_dir_name);
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to clean temp directory");
    }
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    // Download 1 file with seed 42 (default seed)
    let output1 = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            binary_name,
            "--",
            "1",
            "--output-dir",
            temp_dir.to_str().unwrap(),
            "--seed",
            "42",
        ])
        .output()
        .unwrap_or_else(|_| panic!("Failed to run {} first time", binary_name));

    assert!(
        output1.status.success(),
        "{} first run should succeed",
        binary_name
    );

    // Check that exactly 1 file was downloaded
    let entries1: Vec<_> = fs::read_dir(&temp_dir)
        .expect("Failed to read temp directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect directory entries");
    assert_eq!(entries1.len(), 1, "Should download exactly 1 file");
    let first_file = entries1[0].file_name();

    // Download 1 more file with same seed (should skip existing, get a different one)
    let output2 = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            binary_name,
            "--",
            "1",
            "--output-dir",
            temp_dir.to_str().unwrap(),
            "--seed",
            "42",
        ])
        .output()
        .unwrap_or_else(|_| panic!("Failed to run {} second time", binary_name));

    assert!(
        output2.status.success(),
        "{} second run should succeed",
        binary_name
    );

    // Check that we now have 2 different files
    let entries2: Vec<_> = fs::read_dir(&temp_dir)
        .expect("Failed to read temp directory after second run")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to collect directory entries");
    assert_eq!(entries2.len(), 2, "Should have 2 files after second run");

    // Verify the files are different
    let filenames: Vec<_> = entries2.iter().map(|e| e.file_name()).collect();
    assert!(
        filenames.contains(&first_file),
        "First file should still exist"
    );
    assert_ne!(
        filenames[0], filenames[1],
        "Files should have different names"
    );

    // Clean up
    fs::remove_dir_all(&temp_dir).expect("Failed to clean up temp directory");
}

#[test]
fn test_binaries_help_commands() {
    let binaries_with_help = [
        "ustar-benchmark",
        "ustar-parse-debugger",
        "download-bmrb-stars",
        "download-cod-cifs",
        "download-pdbs",
        "ustar-grammar-railroad",
        "sas-demo",
    ];

    for binary in &binaries_with_help {
        let output = Command::new("cargo")
            .args(&["run", "--bin", binary, "--", "--help"])
            .output()
            .unwrap_or_else(|_| panic!("Failed to run {} --help", binary));

        assert!(output.status.success(), "{} --help should succeed", binary);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.is_empty(), "{} help should produce output", binary);
        assert!(
            stdout.contains("Usage") || stdout.contains("USAGE") || stdout.contains("Arguments"),
            "{} help should contain usage information",
            binary
        );
    }
}

#[test]
fn test_sas_demo_basic_execution() {
    // sas-demo doesn't use clap, so test with actual file
    let test_file = "../ustar-parser/tests/test_data/comprehensive_example.star";

    let output = Command::new("cargo")
        .args(&["run", "--bin", "sas-demo", "--", test_file])
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
    test_download_binary_functionality("download-pdbs", "test_download_pdbs");
}

#[test]
fn test_download_bmrb_stars_basic_functionality() {
    test_download_binary_functionality("download-bmrb-stars", "test_download_bmrb");
}

#[test]
fn test_download_cod_cifs_basic_functionality() {
    test_download_binary_functionality("download-cod-cifs", "test_download_cod");
}

#[test]
fn test_ustar_parse_debugger_invalid_file() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "ustar-parse-debugger",
            "--",
            "tests/test_data/invalid_syntax.star",
        ])
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
#[ignore] // Ignored due to very long execution time for grammar railroad generation
fn test_ustar_grammar_railroad_svg_generation() {
    use std::fs;
    use std::path::Path;

    // Create a temporary directory for output
    let temp_dir = std::env::temp_dir().join("test_grammar_railroad");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to clean temp directory");
    }
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    // Find a grammar file to process (look for generated ones, prefer ASCII for speed)
    let grammar_files = [
        "../ustar-parser/src/star_ascii.pest", // Try ASCII first as it's simpler
        "../ustar-parser/src/star_extended.pest",
        "../ustar-parser/src/star_unicode.pest",
    ];

    let mut grammar_file = None;
    for file in &grammar_files {
        if Path::new(file).exists() {
            grammar_file = Some(file);
            break;
        }
    }

    let grammar_file = grammar_file.expect("No grammar file found - run cargo build first");
    let output_svg = temp_dir.join("test_grammar.svg");

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
        "ustar-grammar-railroad should execute successfully"
    );

    // Check that SVG file was created
    assert!(output_svg.exists(), "Should create SVG output file");

    // Check SVG file size (should be reasonable for a grammar diagram)
    let metadata = fs::metadata(&output_svg).expect("Failed to read SVG metadata");
    assert!(
        metadata.len() > 1000,
        "SVG file should be substantial (>1KB)"
    );
    assert!(
        metadata.len() < 50_000_000,
        "SVG file shouldn't be too large (<50MB)"
    );

    // Read and validate the SVG content using usvg for proper SVG validation
    let svg_content = fs::read_to_string(&output_svg).expect("Failed to read SVG file");

    // Basic XML structure checks
    assert!(
        svg_content.contains("<svg"),
        "Should contain SVG root element"
    );
    assert!(
        svg_content.contains("</svg>"),
        "Should have closing SVG tag"
    );

    // Use usvg to validate SVG semantics and structure
    let tree = usvg::Tree::from_str(&svg_content, &usvg::Options::default())
        .expect("Generated SVG should be valid and parseable by usvg");

    // Verify the SVG has some actual content (not just empty)
    let svg_node = tree.root();
    assert!(
        svg_node.has_children(),
        "SVG should contain graphical elements"
    );

    // Check that the SVG has reasonable dimensions
    let size = tree.size();
    assert!(size.width() > 0.0, "SVG should have positive width");
    assert!(size.height() > 0.0, "SVG should have positive height");
    assert!(
        size.width() < 100000.0,
        "SVG width should be reasonable (<100000px)"
    );
    assert!(
        size.height() < 100000.0,
        "SVG height should be reasonable (<100000px)"
    );

    // Clean up
    fs::remove_dir_all(&temp_dir).expect("Failed to clean up temp directory");
}
