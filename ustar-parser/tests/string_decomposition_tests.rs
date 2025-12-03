use std::fs;
use ustar::{parse_default, string_decomposer::decompose_strings};

mod snapshot_utils;

#[test]
fn test_semicolon_string_decomposition_lf_vs_crlf() {
    // Test LF version
    let lf_input = fs::read_to_string("tests/test_data/semicolon_test_lf.star")
        .expect("Failed to read LF test file");
    let mut lf_mutable = parse_default(&lf_input).expect("Failed to parse LF test data");
    decompose_strings(&mut lf_mutable);

    // Test CRLF version
    let crlf_input = fs::read_to_string("tests/test_data/semicolon_test_crlf.star")
        .expect("Failed to read CRLF test file");
    let mut crlf_mutable = parse_default(&crlf_input).expect("Failed to parse CRLF test data");
    decompose_strings(&mut crlf_mutable);

    // Snapshot the LF structure after decomposition
    snapshot_utils::assert_snapshot_gz(
        "string_decomposition_tests__lf_semicolon_decomposed",
        &format!("{:#?}", lf_mutable),
    );

    // Snapshot the CRLF structure after decomposition
    snapshot_utils::assert_snapshot_gz(
        "string_decomposition_tests__crlf_semicolon_decomposed",
        &format!("{:#?}", crlf_mutable),
    );
}
