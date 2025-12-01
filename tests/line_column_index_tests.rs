use ustar::line_column_index::LineColumnIndex;

#[test]
fn test_simple_input() {
    let input = "hello\nworld\ntest";
    let index = LineColumnIndex::new(input);

    // Line 1: "hello" (positions 0-4)
    assert_eq!(index.offset_to_line_col(0), (1, 1)); // 'h'
    assert_eq!(index.offset_to_line_col(4), (1, 5)); // 'o'

    // Line 2: "world" (positions 6-10)
    assert_eq!(index.offset_to_line_col(6), (2, 1)); // 'w'
    assert_eq!(index.offset_to_line_col(10), (2, 5)); // 'd'

    // Line 3: "test" (positions 12-15)
    assert_eq!(index.offset_to_line_col(12), (3, 1)); // 't'
    assert_eq!(index.offset_to_line_col(15), (3, 4)); // 't'
}

#[test]
fn test_line_only_lookup() {
    let input = "line1\nline2\nline3";
    let index = LineColumnIndex::new(input);

    assert_eq!(index.offset_to_line_col(0).0, 1);
    assert_eq!(index.offset_to_line_col(5).0, 1); // newline position
    assert_eq!(index.offset_to_line_col(6).0, 2);
    assert_eq!(index.offset_to_line_col(12).0, 3);
}

#[test]
fn test_empty_lines() {
    let input = "a\n\nb\n";
    let index = LineColumnIndex::new(input);

    assert_eq!(index.offset_to_line_col(0), (1, 1)); // 'a'
    assert_eq!(index.offset_to_line_col(1), (1, 2)); // '\n'
    assert_eq!(index.offset_to_line_col(2), (2, 1)); // empty line
    assert_eq!(index.offset_to_line_col(3), (3, 1)); // 'b'
}
