use std::fs;
use std::path::Path;

#[test]
fn parse_all_bmrb_star_files() {
    let dir = Path::new("tests/test_data/bmrb_stars_tmp");
    assert!(dir.exists() && dir.is_dir(), "Directory {:?} does not exist", dir);
    let mut found = false;
    for entry in fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("entry failed");
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "str" {
                found = true;
                let data = fs::read(&path).expect(&format!("Failed to read file {:?}", path));
                let content = String::from_utf8_lossy(&data).to_string();
                match ustar::parse(&content, &ustar::default_config()) {
                    Ok(_) => {},
                    Err(e) => panic!("Failed to parse {:?}: {}", path, e),
                }
            }
        }
    }
    assert!(found, "No .str files found in {:?}", dir);
}
