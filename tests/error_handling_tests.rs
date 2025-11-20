use ustar::{parse, default_config, ErrorFormatMode};

/// Test data with various parsing errors for snapshot testing
const ERROR_CASES: &[(&str, &str)] = &[
    ("unclosed_string", indoc::indoc! {"
        data_test
        _entry.id                      1ABC
        _entry.description             \"This is an unclosed string that will cause errors
        _entry.author                  'Smith, J.'
    "}),
    
    ("tag_instead_of_value", indoc::indoc! {"
        data_test
        _entry.id                      1ABC
        _entry.description             _another_tag
        _entry.author                  'Smith, J.'
    "}),
    
    // Additional test cases (commented out for now):
    
    // ("multiple_expected_tokens", indoc::indoc! {"
    //     data_test
    //     _entry.id 
    //     _entry.description             'test'
    // "}),
    
    // ("missing_value_simple", indoc::indoc! {"
    //     data_test
    //     _entry.id 
    // "}),
    
    // ("unclosed_string_simple", indoc::indoc! {"
    //     data_test
    //     _entry.description \"unclosed string
    // "}),
    
    // ("context_lines_error", indoc::indoc! {"
    //     data_test
    //     line2
    //     line3  
    //     line4
    //     line5
    //     _entry.invalid_syntax @#$
    //     line7
    //     line8
    //     line9
    // "}),
    
    // ("invalid_loop", indoc::indoc! {"
    //     data_test
    //     loop_
    //     _atom_site.id
    //     _atom_site.x
    //     ATOM 1 2.5
    //     ATOM 
    //     _entry.title 'test'
    // "}),
];

/// Helper function to test error formatting for a specific mode
fn test_error_format_mode(mode: ErrorFormatMode, context_lines: usize, snapshot_prefix: &str) {
    for (case_name, input) in ERROR_CASES {
        let config = default_config();
        
        let result = parse(input, &config);
        assert!(result.is_err(), "Expected error for case: {}", case_name);
        
        let error = result.unwrap_err();
        let formatted = error.format_error(mode, context_lines);
        
        insta::assert_snapshot!(
            format!("{}_{}", snapshot_prefix, case_name), 
            formatted
        );
    }
}

#[test]
fn test_basic_error_format_snapshots() {
    test_error_format_mode(ErrorFormatMode::Basic, 5, "basic_error");
}

#[test]
fn test_ascii_error_format_snapshots() {
    test_error_format_mode(ErrorFormatMode::Ascii, 3, "ascii_error");
}

#[test] 
#[cfg(feature = "extended-errors")]
fn test_fancy_error_format_snapshots() {
    test_error_format_mode(ErrorFormatMode::Fancy, 3, "fancy_error");
}

