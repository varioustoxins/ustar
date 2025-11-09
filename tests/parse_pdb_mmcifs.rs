use std::fs;
use std::path::Path;

#[test]
fn parse_all_mmcif_files_in_pdb_mmcifs() {
    let dir = Path::new("tests/test_data/pdb_mmcifs");
    assert!(dir.exists() && dir.is_dir(), "Directory {:?} does not exist", dir);
    let mut found = false;
    for entry in fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("entry failed");
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "cif" || ext == "gz" { // support both .cif and .cif.gz
                found = true;
                let data = fs::read(&path).expect(&format!("Failed to read file {:?}", path));
                let content = if ext == "gz" {
                    let mut decoder = flate2::read::GzDecoder::new(&data[..]);
                    let mut s = String::new();
                    use std::io::Read;
                    decoder.read_to_string(&mut s).expect("Failed to decompress");
                    s
                } else {
                    String::from_utf8_lossy(&data).to_string()
                };
                match ustar::parse(&content, &ustar::default_config()) {
                    Ok(_) => {},
                    Err(e) => panic!("Failed to parse {:?}: {}", path, e),
                }
            }
        }
    }
    assert!(found, "No .cif or .cif.gz files found in {:?}", dir);
}
