use std::process::Command;
use std::str;
use ustar_test_utils::assert_snapshot_gz;

/// Test helper to run ustar-dumper and capture output
fn run_ustar_parser(input_file: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ustar-dumper", "--", input_file])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(format!("Command failed: {}", stderr).into())
    }
}

/// Test helper to run ustar-dumper with tree flag and capture output
fn run_ustar_parser_with_tree(input_file: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ustar-dumper", "--", "--tree", input_file])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(format!("Command failed: {}", stderr).into())
    }
}

/// Test helper to run ustar-dumper with stdin and capture output
fn run_ustar_parser_stdin(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "ustar-dumper"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.take() {
        use std::io::Write;
        let mut stdin = stdin;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(format!("Command failed: {}", stderr).into())
    }
}

#[test]
fn test_cli_output_format() {
    // Test different line ending formats to ensure cross-platform compatibility
    let test_cases = [
        ("lf", "\n", "cli_output_lf"),
        ("crlf", "\r\n", "cli_output_crlf"),
    ];

    for (name, line_ending, snapshot_name) in test_cases {
        let input = format!(
            "data_test{}_item \"hello world\"{}",
            line_ending, line_ending
        );
        let output = run_ustar_parser_stdin(&input)
            .expect(&format!("Failed to run ustar-dumper with {} input", name));
        assert_snapshot_gz(&format!("ustar_dumper_tests__{}", snapshot_name), &output);
    }
}

#[test]
fn test_cli_whitespace_visualization() {
    let output = run_ustar_parser_stdin("data_test\n_item \"hello world\"\n")
        .expect("Failed to run ustar-dumper");

    // Check that spaces in quoted strings are highlighted with dots
    // The dot character is surrounded by ANSI escape sequences, so we check for the parts
    assert!(
        output.contains("hello") && output.contains("world"),
        "Should contain hello and world"
    );
    // Check that there's some kind of highlighting between them (ANSI codes)
    assert!(
        output.contains("\x1b["),
        "Should contain ANSI escape sequences for highlighting"
    );
}

#[test]
fn test_cli_test_input_star_file() {
    let output = run_ustar_parser("tests/test_data/test_input.star")
        .expect("Failed to run ustar-dumper on test_input.star");

    assert_snapshot_gz("ustar_dumper_tests__test_input_star_output", &output);
}

#[test]
fn test_cli_simple_star_file() {
    let output = run_ustar_parser("tests/test_data/simple_star_file.star")
        .expect("Failed to run ustar-dumper on simple_star_file.star");

    assert_snapshot_gz("ustar_dumper_tests__simple_star_file_output", &output);
}

#[test]
fn test_cli_semicolon_bounded_file() {
    let output = run_ustar_parser("tests/test_data/semicolon_bounded.star")
        .expect("Failed to run ustar-dumper on semicolon_bounded.star");

    // Check source information
    assert!(
        output.contains("source: tests/test_data/semicolon_bounded.star"),
        "Should show correct source file"
    );

    // Should contain semicolon bounded text handling
    assert!(
        output.contains("semi_colon_string"),
        "Should identify semicolon bounded text"
    );

    // Should successfully parse
    assert!(output.contains("lines:"), "Should show line count");
    assert!(output.contains("symbols:"), "Should show symbol count");
}

#[test]
fn test_cli_mixed_content_file() {
    let output = run_ustar_parser("tests/test_data/mixed_content.star")
        .expect("Failed to run ustar-dumper on mixed_content.star");

    // Check source information
    assert!(
        output.contains("source: tests/test_data/mixed_content.star"),
        "Should show correct source file"
    );

    // Should handle mixed content types
    assert!(output.contains("lines:"), "Should show line count");
    assert!(output.contains("symbols:"), "Should show symbol count");
}

#[test]
fn test_cli_stdin_input() {
    let simple_input = "data_minimal\n_test_item 123\n";
    let output =
        run_ustar_parser_stdin(simple_input).expect("Failed to run ustar-dumper with stdin");

    // Check stdin is properly identified
    assert!(
        output.contains("source: -"),
        "Should identify stdin as source"
    );

    // Should parse the minimal input
    assert!(
        output.contains("data_minimal"),
        "Should contain the data block name"
    );
    assert!(output.contains("123"), "Should contain the test value");
    assert!(output.contains("lines:"), "Should show line count");
    assert!(output.contains("symbols:"), "Should show symbol count");
}

#[test]
fn test_cli_error_handling() {
    // Test with invalid input
    let result = run_ustar_parser_stdin("invalid syntax here");

    // Should fail gracefully - we expect this to return an error
    assert!(result.is_err(), "Should fail on invalid syntax");
}

#[test]
fn test_cli_comprehensive_example_without_tree() {
    let output = run_ustar_parser("tests/test_data/comprehensive_example.star")
        .expect("Failed to run ustar-dumper on comprehensive_example.star");

    assert_snapshot_gz(
        "ustar_dumper_tests__comprehensive_example_without_tree",
        &output,
    );
}

#[test]
fn test_cli_comprehensive_example_with_tree() {
    let output = run_ustar_parser_with_tree("tests/test_data/comprehensive_example.star")
        .expect("Failed to run ustar-dumper with tree on comprehensive_example.star");

    assert_snapshot_gz(
        "ustar_dumper_tests__comprehensive_example_with_tree",
        &output,
    );
}

#[test]
fn test_cli_simple_example_with_tree() {
    let input = "data_test _value 'hello world'\n";
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "ustar-dumper", "--", "--tree"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn ustar-dumper");

    if let Some(stdin) = child.stdin.take() {
        use std::io::Write;
        let mut stdin = stdin;
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for output");
    let output_str = String::from_utf8(output.stdout).expect("Failed to parse stdout");

    assert_snapshot_gz("ustar_dumper_tests__simple_example_with_tree", &output_str);
}
