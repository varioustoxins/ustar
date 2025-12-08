//! Test the automatic test data download functionality

use ustar_test_utils::ensure_test_data_available;

#[test]
fn test_ensure_data_available_with_missing_files() {
    let test_data_dir = "tests/test_data/nef_spec";

    // This test checks if the download system works
    // If no-large-tests is enabled, it should fail
    // If no-large-tests is disabled (default), it should download and succeed
    let result = ensure_test_data_available(test_data_dir);

    #[cfg(feature = "no-large-tests")]
    {
        // With no-large-tests, should fail
        assert!(
            result.is_err(),
            "Should fail when no-large-tests is enabled and files are missing"
        );
        println!("✅ no-large-tests correctly prevented download");
    }

    #[cfg(not(feature = "no-large-tests"))]
    {
        use std::path::Path;

        match result {
            Ok(_) => {
                println!("✅ Download succeeded");
                // Verify some files were actually downloaded
                let nef_spec_dir = Path::new(test_data_dir);
                assert!(nef_spec_dir.exists());

                // Check that at least one expected file exists
                let expected_files = ["CCPN_H1GI_clean.nef", "XPLOR_test1.nef"];
                let mut found_files = 0;
                for file in &expected_files {
                    if nef_spec_dir.join(file).exists() {
                        found_files += 1;
                        println!("Found downloaded file: {}", file);
                    }
                }
                assert!(found_files > 0, "No expected files were downloaded");
            }
            Err(e) => {
                panic!("Download failed when it should have succeeded: {}", e);
            }
        }
    }
}
