use std::collections::HashMap;
use std::fs;
use std::path::Path;
use ustar::{ConfigKey, ConfigValue, EncodingMode, ErrorFormatMode, ParserConfig};
use ustar_test_utils::ensure_test_data_available;

#[test]
fn parse_ascii_dict_files() {
    let dir = Path::new("tests/test_data/dicts");

    // Verify test data is available and checksums are valid
    ensure_test_data_available(dir)
        .expect("Failed to verify test data integrity for dictionary files");

    assert!(
        dir.exists() && dir.is_dir(),
        "Directory {:?} does not exist",
        dir
    );

    // Create ASCII config with fancy error formatting
    let mut config: ParserConfig = HashMap::new();
    config.insert(
        ConfigKey::Encoding,
        ConfigValue::Encoding(EncodingMode::Ascii),
    );
    config.insert(
        ConfigKey::ErrorFormat,
        ConfigValue::ErrorFormat(ErrorFormatMode::Fancy),
    );
    config.insert(ConfigKey::ContextLines, ConfigValue::Usize(5));
    config.insert(ConfigKey::DecomposedStrings, ConfigValue::Bool(true));
    config.insert(ConfigKey::AutoDetectBom, ConfigValue::Bool(true));

    // Files that are known to work with ASCII encoding
    let ascii_compatible_files = [
        "mmcif_biosync.dic",
        "mmcif_ddl.dic",
        "mmcif_em.dic",
        "mmcif_ihm_ext.dic",
        "mmcif_ihm_flr_ext.dic",
        "mmcif_investigation_fraghub.dic",
        "mmcif_nmr-star.dic",
        "mmcif_pdbx_v40.dic",
        "mmcif_pdbx_v50.dic",
        "mmcif_pdbx_v5_next.dic",
        "mmcif_pdbx_vrpt.dic",
        "mmcif_sas.dic",
        "mmcif_std.dic",
        "mmcif_sym.dic",
    ];

    let mut found = false;
    let mut errors = Vec::new();

    for filename in ascii_compatible_files {
        let file_path = dir.join(filename);
        if file_path.exists() {
            found = true;
            let data = fs::read(&file_path).expect(&format!("Failed to read file {:?}", file_path));
            let content = String::from_utf8_lossy(&data).to_string();
            match ustar::parse(&content, &config) {
                Ok(_) => {}
                Err(e) => errors.push(format!("Failed to parse {:?}: {}", file_path, e)),
            }
        }
    }

    assert!(found, "No ASCII-compatible .dic files found in {:?}", dir);

    if !errors.is_empty() {
        panic!(
            "Parsing errors in {} ASCII-compatible files:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
}

#[test]
fn parse_unicode_dict_files() {
    let dir = Path::new("tests/test_data/dicts");

    // Verify test data is available and checksums are valid
    ensure_test_data_available(dir)
        .expect("Failed to verify test data integrity for dictionary files");

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

    // Files that require Unicode encoding
    let unicode_required_files = ["mmcif_img.dic", "mmcif_nef.dic", "mmcif_ndb_ntc.dic"];

    let mut found = false;
    let mut errors = Vec::new();

    for filename in unicode_required_files {
        let file_path = dir.join(filename);
        if file_path.exists() {
            found = true;
            let data = fs::read(&file_path).expect(&format!("Failed to read file {:?}", file_path));
            let content = String::from_utf8_lossy(&data).to_string();
            match ustar::parse(&content, &config) {
                Ok(_) => {}
                Err(e) => errors.push(format!("Failed to parse {:?}: {}", file_path, e)),
            }
        }
    }

    assert!(found, "No Unicode-required .dic files found in {:?}", dir);

    if !errors.is_empty() {
        panic!(
            "Parsing errors in {} Unicode-required files:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
}
