use std::collections::HashMap;
use std::fs;
use std::path::Path;
use ustar::{ConfigKey, ConfigValue, EncodingMode, ErrorFormatMode, ParserConfig};
use ustar_test_utils::ensure_test_data_available;

#[test]
fn parse_all_nef_example_files() {
    let dir = Path::new("tests/test_data/nef_examples");

    // Verify test data is available and checksums are valid
    ensure_test_data_available(dir).expect("Failed to verify test data integrity for NEF examples");

    assert!(
        dir.exists() && dir.is_dir(),
        "Directory {:?} does not exist",
        dir
    );

    // Create Unicode config with fancy error formatting
    let mut config: ParserConfig = HashMap::new();
    config.insert(
        ConfigKey::Encoding,
        ConfigValue::Encoding(EncodingMode::Unicode),
    );
    config.insert(
        ConfigKey::ErrorFormat,
        ConfigValue::ErrorFormat(ErrorFormatMode::Fancy),
    );
    config.insert(ConfigKey::ContextLines, ConfigValue::Usize(5));
    config.insert(ConfigKey::DecomposedStrings, ConfigValue::Bool(true));
    config.insert(ConfigKey::AutoDetectBom, ConfigValue::Bool(true));

    let mut found = false;
    let mut errors = Vec::new();
    for entry in fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("entry failed");
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "nef" {
                found = true;
                let data = fs::read(&path).expect(&format!("Failed to read file {:?}", path));
                let content = String::from_utf8_lossy(&data).to_string();
                match ustar::parse(&content, &config) {
                    Ok(_) => {}
                    Err(e) => errors.push(format!("Failed to parse {:?}: {}", path, e)),
                }
            }
        }
    }
    assert!(found, "No .nef files found in {:?}", dir);

    if !errors.is_empty() {
        panic!(
            "Parsing errors in {} files:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
}

#[test]
fn parse_all_nef_spec_files() {
    let dir = Path::new("tests/test_data/nef_spec");

    // Verify test data is available and checksums are valid
    ensure_test_data_available(dir).expect("Failed to verify test data integrity for NEF spec");

    assert!(
        dir.exists() && dir.is_dir(),
        "Directory {:?} does not exist",
        dir
    );

    // Create Unicode config with fancy error formatting
    let mut config: ParserConfig = HashMap::new();
    config.insert(
        ConfigKey::Encoding,
        ConfigValue::Encoding(EncodingMode::Unicode),
    );
    config.insert(
        ConfigKey::ErrorFormat,
        ConfigValue::ErrorFormat(ErrorFormatMode::Fancy),
    );
    config.insert(ConfigKey::ContextLines, ConfigValue::Usize(5));
    config.insert(ConfigKey::DecomposedStrings, ConfigValue::Bool(true));
    config.insert(ConfigKey::AutoDetectBom, ConfigValue::Bool(true));

    let mut found = false;
    let mut errors = Vec::new();
    for entry in fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("entry failed");
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "nef" {
                found = true;
                let data = fs::read(&path).expect(&format!("Failed to read file {:?}", path));
                let content = String::from_utf8_lossy(&data).to_string();
                match ustar::parse(&content, &config) {
                    Ok(_) => {}
                    Err(e) => errors.push(format!("Failed to parse {:?}: {}", path, e)),
                }
            }
        }
    }
    assert!(found, "No .nef files found in {:?}", dir);

    if !errors.is_empty() {
        panic!(
            "Parsing errors in {} files:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
}
