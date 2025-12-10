use rstest::rstest;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use ustar::{ConfigKey, ConfigValue, EncodingMode, ErrorFormatMode, ParserConfig};
use ustar_test_utils::ensure_test_data_available;

struct TestResult {
    files_tested: usize,
    failures: Vec<String>,
    skipped: Vec<String>,
    known_failures: Vec<String>,
}

impl TestResult {
    fn new() -> Self {
        Self {
            files_tested: 0,
            failures: Vec::new(),
            skipped: Vec::new(),
            known_failures: Vec::new(),
        }
    }

    fn assert_success(&self, test_name: &str, encoding_mode: Option<EncodingMode>) {
        let mode_str = if let Some(mode) = encoding_mode {
            format!(" with {:?} encoding", mode)
        } else {
            String::new()
        };

        assert!(
            self.files_tested > 0,
            "No files found for {} test",
            test_name
        );

        assert!(
            self.failures.is_empty(),
            "Failed to parse {} file(s) in {}{} (skipped {}, known failures {}): {}",
            self.failures.len(),
            test_name,
            mode_str,
            self.skipped.len(),
            self.known_failures.len(),
            self.failures.join(", ")
        );
    }
}

fn create_config(encoding_mode: EncodingMode) -> ParserConfig {
    let mut config: ParserConfig = HashMap::new();
    config.insert(ConfigKey::Encoding, ConfigValue::Encoding(encoding_mode));
    config.insert(
        ConfigKey::ErrorFormat,
        ConfigValue::ErrorFormat(ErrorFormatMode::Fancy),
    );
    config.insert(ConfigKey::ContextLines, ConfigValue::Usize(5));
    config.insert(ConfigKey::DecomposedStrings, ConfigValue::Bool(true));
    config.insert(ConfigKey::AutoDetectBom, ConfigValue::Bool(true));
    config
}

fn test_directory_files(
    dir_path: &Path,
    file_extension: &str,
    encoding_mode: EncodingMode,
    unicode_files: &[&str],
    known_failures: &[&str],
) -> TestResult {
    let mut result = TestResult::new();

    if !dir_path.exists() || !dir_path.is_dir() {
        panic!("Test directory not found: {:?}", dir_path);
    }

    let config = create_config(encoding_mode);
    let is_ascii_mode = matches!(encoding_mode, EncodingMode::Ascii);

    let entries = fs::read_dir(dir_path)
        .unwrap_or_else(|e| panic!("Failed to read directory {:?}: {}", dir_path, e));

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some(file_extension) {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();

            // Skip Unicode files when testing with ASCII mode
            if is_ascii_mode && unicode_files.contains(&filename.as_str()) {
                result.files_tested += 1;
                result.skipped.push(filename);
                continue;
            }

            result.files_tested += 1;

            let data =
                fs::read(&path).unwrap_or_else(|e| panic!("Failed to read file {:?}: {}", path, e));
            let content = String::from_utf8_lossy(&data).to_string();

            match ustar::parse(&content, &config) {
                Ok(_) => {
                    // Silent success
                }
                Err(e) => {
                    if known_failures.contains(&filename.as_str()) {
                        result.known_failures.push(filename.clone());
                    } else {
                        result.failures.push(format!("{}: {}", filename, e));
                    }
                }
            }
        }
    }

    result
}

#[test]
fn test_bmrb_star_files_can_be_parsed() {
    let dir = Path::new("tests/test_data/bmrb_stars");
    ensure_test_data_available(dir).expect("Failed to verify test data integrity for BMRB stars");

    let result = test_directory_files(dir, "str", EncodingMode::Unicode, &[], &[]);
    result.assert_success("BMRB STAR", Some(EncodingMode::Unicode));
}

#[test]
fn test_cod_cif_files_can_be_parsed() {
    let dir = Path::new("tests/test_data/cod_cifs");
    ensure_test_data_available(dir).expect("Failed to verify test data integrity for COD CIFs");

    let result = test_directory_files(dir, "cif", EncodingMode::Unicode, &[], &[]);
    result.assert_success("COD CIF", Some(EncodingMode::Unicode));
}

#[test]
fn test_nef_examples_can_be_parsed() {
    let dir = Path::new("tests/test_data/nef_examples");
    ensure_test_data_available(dir).expect("Failed to verify test data integrity for NEF examples");

    let result = test_directory_files(dir, "nef", EncodingMode::Unicode, &[], &[]);
    result.assert_success("NEF examples", Some(EncodingMode::Unicode));
}

#[test]
fn test_nef_specification_files_can_be_parsed() {
    let nef_dir = Path::new("tests/test_data/nef_spec");

    ensure_test_data_available(nef_dir)
        .expect("Failed to verify test data integrity for NEF specification files");

    let known_failures = vec!["CCPN_XPLOR_test1.nef"];
    let result = test_directory_files(nef_dir, "nef", EncodingMode::Ascii, &[], &known_failures);

    result.assert_success("NEF specification", Some(EncodingMode::Ascii));
}

#[rstest]
#[case::ascii(EncodingMode::Ascii)]
#[case::unicode(EncodingMode::Unicode)]
fn test_nef_site_files_can_be_parsed(#[case] encoding_mode: EncodingMode) {
    let nef_dir = Path::new("tests/test_data/nef_spec");

    ensure_test_data_available(nef_dir)
        .expect("Failed to verify test data integrity for NEF site files");

    // Files known to have parsing failures due to quote handling issues
    let known_failures = vec![
        "CCPN_H1GI_clean.nef",
        "CCPN_H1GI_clean_extended.nef",
        "CCPN_Sec5Part3.nef",
    ];

    let result = test_directory_files(nef_dir, "nef", encoding_mode, &[], &known_failures);
    result.assert_success("NEF site", Some(encoding_mode));
}

#[test]
fn test_mmcif_files_can_be_parsed() {
    let mmcif_dir: PathBuf = ["tests", "test_data", "mmcif"].iter().collect();
    let result = test_directory_files(&mmcif_dir, "cif", EncodingMode::Ascii, &[], &[]);
    result.assert_success("mmCIF", Some(EncodingMode::Ascii));
}

#[rstest]
#[case::ascii(EncodingMode::Ascii)]
#[case::unicode(EncodingMode::Unicode)]
fn test_mmcif_dictionaries_can_be_parsed(#[case] encoding_mode: EncodingMode) {
    let dicts_dir: PathBuf = ["tests", "test_data", "dicts"].iter().collect();

    // File known to contain Unicode characters (skip for ASCII mode)
    let unicode_files = vec!["mmcif_ndb_ntc.dic"];

    // Files with known parsing failures (grammar issues, not Unicode-related)
    let known_failures = vec!["mmcif_img.dic", "mmcif_nef.dic"];

    let result = test_directory_files(
        &dicts_dir,
        "dic",
        encoding_mode,
        &unicode_files,
        &known_failures,
    );
    result.assert_success("mmCIF dictionaries", Some(encoding_mode));
}

#[test]
fn test_pdb_mmcif_files_can_be_parsed() {
    let dir = Path::new("tests/test_data/pdb_mmcifs");
    ensure_test_data_available(dir).expect("Failed to verify test data integrity for PDB mmCIFs");

    let result = test_directory_files(dir, "cif", EncodingMode::Unicode, &[], &[]);
    result.assert_success("PDB mmCIF", Some(EncodingMode::Unicode));
}
