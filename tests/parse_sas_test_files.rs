use std::fs;
use std::path::Path;

// loop1 - missing value in row 2 - currently ustar doesn't count values
// loop2 - loop with no rows - curently ustar doesn't count this as an error
// loop3 - loop with no headers - an error in ustar
// loop4 - loop with no body - an error in ustar
// loop5 - missing closing triple quote -  triple quotes not supported by ustar
// warning.cif - contains triple quotes - not supported by ustar

static KNOWN_FAILURES: &[&str] = &[
    "loop3.str",
    "loop4.str",
    "loop5.str",
    "warning.cif",
    "warning.str",
];

#[test]
fn parse_all_sas_test_files() {
    let dir = Path::new("tests/test_data/sas_test_files");
    assert!(
        dir.exists() && dir.is_dir(),
        "Directory {:?} does not exist",
        dir
    );
    let mut found = false;

    for entry in fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("entry failed");
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "str" || ext == "cif" || ext == "dic" {
                found = true;
                let data = fs::read(&path).unwrap_or_else(|_| panic!("Failed to read file {:?}", path));
                let content = String::from_utf8_lossy(&data).to_string();
                let filename = path.file_name().unwrap().to_string_lossy();
                match ustar::parse(&content, &ustar::default_config()) {
                    Ok(_) => {
                        if KNOWN_FAILURES.contains(&filename.as_ref()) {
                            panic!(
                                "File {:?} was expected to fail but parsed successfully",
                                path
                            );
                        }
                    }
                    Err(e) => {
                        if !KNOWN_FAILURES.contains(&filename.as_ref()) {
                            panic!("Failed to parse {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    }
    assert!(found, "No .str, .cif, or .dic files found in {:?}", dir);
}
