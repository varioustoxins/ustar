use ustar::{parse, ParserConfig, ConfigKey, ConfigValue, EncodingMode, default_config};
use std::collections::HashMap;

#[test]
fn test_ascii_mode_basic() {
    let input = "data_test\n_item value\n";
    let result = parse(input, &default_config());
    assert!(result.is_ok(), "ASCII mode should parse basic ASCII input");
}

#[test]
fn test_unicode_mode_with_unicode_chars() {
    // Test with Unicode characters in unquoted value
    let input = "data_test\nloop_\n_item\nαβɣ\nstop_\n";
    
    let mut config: ParserConfig = HashMap::new();
    config.insert(ConfigKey::Encoding, ConfigValue::Encoding(EncodingMode::Unicode));
    config.insert(ConfigKey::DecomposedStrings, ConfigValue::Bool(true));
    
    let result = parse(input, &config);
    assert!(result.is_ok(), "Unicode mode should parse Unicode characters: {:?}", result);
}

#[test]
fn test_bom_auto_detection() {
    // UTF-8 BOM is U+FEFF which is 3 bytes: EF BB BF
    let input = "\u{FEFF}data_test\n_item value\n";
    
    // Even with ASCII mode config, BOM should switch to Unicode
    let result = parse(input, &default_config());
    assert!(result.is_ok(), "BOM should trigger Unicode mode");
}

#[test]
fn test_extended_ascii_with_nbsp() {
    // Non-breaking space should act as whitespace in extended ASCII mode
    let input = "data_test\nloop_\n_item\nvalue1\nvalue2\u{00A0}with\u{00A0}nbsp\nstop_\n";
    
    let mut config: ParserConfig = HashMap::new();
    config.insert(ConfigKey::Encoding, ConfigValue::Encoding(EncodingMode::ExtendedAscii));
    config.insert(ConfigKey::DecomposedStrings, ConfigValue::Bool(true));
    
    let result = parse(input, &config);
    assert!(result.is_ok(), "Extended ASCII mode should handle non-breaking space: {:?}", result);
}

#[test]
fn test_parse_default_convenience() {
    let input = "data_test\n_item value\n";
    let result = ustar::parse_default(input);
    assert!(result.is_ok(), "parse_default should work for basic input");
}
