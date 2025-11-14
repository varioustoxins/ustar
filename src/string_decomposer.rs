//! Transform MutablePairs to decompose strings into delimiter + content + delimiter tokens
//!
//! This module provides in-place transformation of string MutablePairs into three separate tokens:
//! 1. Opening delimiter (DOUBLE_QUOTE, SINGLE_QUOTE, or NEWLINE_SEMICOLON)
//! 2. Content (as "string" rule name)
//! 3. Closing delimiter (DOUBLE_QUOTE, SINGLE_QUOTE, or NEWLINE_SEMICOLON)
//!
//! All offsets are preserved from the original string.

use crate::mutable_pair::MutablePair;

/// Decompose string MutablePairs in-place
pub fn decompose_strings(pair: &mut MutablePair) {
    match pair.rule_name() {
        "double_quote_string" => {
            decompose_delimited_string(pair, "\"", "DOUBLE_QUOTE");
        }
        "single_quote_string" => {
            decompose_delimited_string(pair, "'", "SINGLE_QUOTE");
        }
        "semi_colon_string" => {
            decompose_delimited_string(pair, "\n;", "NEWLINE_SEMICOLON");
        }
        "non_quoted_string" => {
            // Convert non_quoted_string to string rule
            pair.rule_name = "string".to_owned();
        }
        _ => {
            // Recursively process children
            for child in &mut pair.children {
                decompose_strings(child);
            }
        }
    }
}

/// Decompose delimited string into [delimiter, string, delimiter]
/// Works for single-char delimiters (quotes) and multi-char delimiters (newline-semicolon)
fn decompose_delimited_string(pair: &mut MutablePair, delimiter: &str, delimiter_name: &str) {
    let content = &pair.content;
    let start_pos = pair.start;
    let delimiter_len = delimiter.len();
    
    if content.len() >= 2 * delimiter_len 
        && content.starts_with(delimiter) 
        && content.ends_with(delimiter) 
    {
        let inner_content = &content[delimiter_len..content.len() - delimiter_len];
        
        // Create three new children
        let opening_delimiter = MutablePair::new(
            delimiter_name,
            delimiter.to_string(),
            start_pos,
            start_pos + delimiter_len,
        );
        
        let string_content = MutablePair::new(
            "string",
            inner_content.to_string(),
            start_pos + delimiter_len,
            start_pos + delimiter_len + inner_content.len(),
        );
        
        let closing_delimiter = MutablePair::new(
            delimiter_name,
            delimiter.to_string(),
            start_pos + delimiter_len + inner_content.len(),
            start_pos + content.len(),
        );
        
        // Replace children with decomposed tokens
        pair.children = vec![opening_delimiter, string_content, closing_delimiter];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decompose_double_quoted() {
        let mut pair = MutablePair::new(
            "double_quote_string",
            "\"hello world\"".to_string(),
            0,
            13,
        );
        
        decompose_strings(&mut pair);
        
        let expected = MutablePair::with_children(
            "double_quote_string",
            "\"hello world\"".to_string(),
            0,
            13,
            vec![
                MutablePair::new("DOUBLE_QUOTE", "\"".to_string(), 0, 1),
                MutablePair::new("string", "hello world".to_string(), 1, 12),
                MutablePair::new("DOUBLE_QUOTE", "\"".to_string(), 12, 13),
            ],
        );
        
        assert_eq!(format!("{}", pair), format!("{}", expected));
    }
    
    #[test]
    fn test_decompose_single_quoted() {
        let mut pair = MutablePair::new(
            "single_quote_string",
            "'test'".to_string(),
            5,
            11,
        );
        
        decompose_strings(&mut pair);
        
        let expected = MutablePair::with_children(
            "single_quote_string",
            "'test'".to_string(),
            5,
            11,
            vec![
                MutablePair::new("SINGLE_QUOTE", "'".to_string(), 5, 6),
                MutablePair::new("string", "test".to_string(), 6, 10),
                MutablePair::new("SINGLE_QUOTE", "'".to_string(), 10, 11),
            ],
        );
        
        assert_eq!(format!("{}", pair), format!("{}", expected));
    }
    
    #[test]
    fn test_convert_non_quoted_to_string() {
        let mut pair = MutablePair::new(
            "non_quoted_string",
            "simple".to_string(),
            10,
            16,
        );
        
        decompose_strings(&mut pair);
        
        let expected = MutablePair::new(
            "string",
            "simple".to_string(),
            10,
            16,
        );
        
        assert_eq!(format!("{}", pair), format!("{}", expected));
    }
    
    #[test]
    fn test_decompose_strings_in_nested_structure() {
        // Simulate a data_loop with nested string values
        let mut loop_pair = MutablePair::with_children(
            "data_loop",
            "loop_\n_name1\n_name2\n\"value1\"\n'value2'\nsimple".to_string(),
            0,
            48,
            vec![
                MutablePair::new("loop_", "loop_".to_string(), 0, 5),
                MutablePair::new("data_name", "_name1".to_string(), 6, 12),
                MutablePair::new("data_name", "_name2".to_string(), 13, 19),
                MutablePair::new("double_quote_string", "\"value1\"".to_string(), 20, 28),
                MutablePair::new("single_quote_string", "'value2'".to_string(), 29, 37),
                MutablePair::new("non_quoted_string", "simple".to_string(), 38, 44),
            ],
        );
        
        decompose_strings(&mut loop_pair);
        
        let expected = MutablePair::with_children(
            "data_loop",
            "loop_\n_name1\n_name2\n\"value1\"\n'value2'\nsimple".to_string(),
            0,
            48,
            vec![
                MutablePair::new("loop_", "loop_".to_string(), 0, 5),
                MutablePair::new("data_name", "_name1".to_string(), 6, 12),
                MutablePair::new("data_name", "_name2".to_string(), 13, 19),
                MutablePair::with_children(
                    "double_quote_string",
                    "\"value1\"".to_string(),
                    20,
                    28,
                    vec![
                        MutablePair::new("DOUBLE_QUOTE", "\"".to_string(), 20, 21),
                        MutablePair::new("string", "value1".to_string(), 21, 27),
                        MutablePair::new("DOUBLE_QUOTE", "\"".to_string(), 27, 28),
                    ],
                ),
                MutablePair::with_children(
                    "single_quote_string",
                    "'value2'".to_string(),
                    29,
                    37,
                    vec![
                        MutablePair::new("SINGLE_QUOTE", "'".to_string(), 29, 30),
                        MutablePair::new("string", "value2".to_string(), 30, 36),
                        MutablePair::new("SINGLE_QUOTE", "'".to_string(), 36, 37),
                    ],
                ),
                MutablePair::new("string", "simple".to_string(), 38, 44),
            ],
        );
        
        assert_eq!(format!("{}", loop_pair), format!("{}", expected));
    }
}
