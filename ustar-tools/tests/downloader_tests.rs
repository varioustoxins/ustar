// Integration tests for download functionality using mocked HTTP clients
// This allows us to test the download logic without network dependencies

use std::path::PathBuf;
use std::sync::Arc;
use ustar_test_utils::MockHttpClient;
use ustar_tools::downloader_common::{
    DataSource, DownloadError, DownloaderConfig, GenericDownloader,
};

// Mock PDB data source for testing
struct MockPdbDataSource {
    http_client: Arc<MockHttpClient>,
}

impl MockPdbDataSource {
    fn new_with_fixtures() -> Self {
        let http_client = Arc::new(
            MockHttpClient::new()
                .with_file_response(
                    "https://files.rcsb.org/pub/pdb/holdings/current_holdings.txt",
                    "tests/fixtures/pdb_holdings.txt",
                )
                .with_file_response(
                    "https://files.rcsb.org/download/1abc.cif",
                    "tests/fixtures/1abc.cif",
                )
                .with_file_response(
                    "https://files.rcsb.org/download/2def.cif",
                    "tests/fixtures/1abc.cif", // Reuse same fixture
                ),
        );

        Self { http_client }
    }
}

impl DataSource for MockPdbDataSource {
    fn get_available_entries(&self) -> Result<Vec<String>, DownloadError> {
        let text = self
            .http_client
            .get("https://files.rcsb.org/pub/pdb/holdings/current_holdings.txt")?;
        let entries: Vec<String> = text
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.to_lowercase())
            .collect();
        Ok(entries)
    }

    fn download_entry(
        &self,
        pdb_id: &str,
        output_path: &PathBuf,
    ) -> Result<PathBuf, DownloadError> {
        let url = format!("https://files.rcsb.org/download/{}.cif", pdb_id);
        let content = self.http_client.get(&url)?;

        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(output_path, content)?;
        Ok(output_path.clone())
    }
}

#[test]
fn test_mock_pdb_downloader_basic_functionality() {
    let data_source = MockPdbDataSource::new_with_fixtures();

    // Test getting available entries
    let entries = data_source.get_available_entries().unwrap();
    assert_eq!(entries, vec!["1abc", "2def", "3ghi", "4test"]);

    // Test downloading a file
    let temp_dir = std::env::temp_dir().join("mock_pdb_test");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let output_path = temp_dir.join("1abc.cif");

    let result = data_source.download_entry("1abc", &output_path).unwrap();
    assert_eq!(result, output_path);
    assert!(output_path.exists());

    // Verify file content
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("data_1ABC"));
    assert!(content.contains("_entry.id    1ABC"));

    // Clean up
    std::fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn test_generic_downloader_with_mock_data_source() {
    let data_source = MockPdbDataSource::new_with_fixtures();

    let temp_dir = std::env::temp_dir().join("mock_downloader_test");
    let config = DownloaderConfig::new()
        .output_dir(temp_dir.to_str().unwrap())
        .verbose(false)
        .file_extension("cif");

    let downloader = GenericDownloader::new(config, data_source);

    // Download 2 files with deterministic seed
    let batch = downloader
        .download_unique_random_batch(2, 42)
        .expect("Download should succeed with mock data");

    assert_eq!(batch.len(), 2);

    // Verify files exist
    for (id, path) in &batch {
        assert!(
            path.exists(),
            "Downloaded file {} should exist at {:?}",
            id,
            path
        );
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("data_"), "File should contain CIF data");
    }

    // Verify no duplicates
    let ids: Vec<&String> = batch.iter().map(|(id, _)| id).collect();
    assert_ne!(ids[0], ids[1], "Should download different files");

    // Clean up
    std::fs::remove_dir_all(&temp_dir).unwrap();
}

// Mock BMRB data source for testing
struct MockBmrbDataSource {
    http_client: Arc<MockHttpClient>,
}

impl MockBmrbDataSource {
    fn new_with_fixtures() -> Self {
        let http_client = Arc::new(
            MockHttpClient::new()
                .with_file_response(
                    "https://bmrb.io/ftp/pub/bmrb/entry_directories/",
                    "tests/fixtures/bmrb_directory_list.html",
                )
                .with_file_response(
                    "https://bmrb.io/ftp/pub/bmrb/entry_directories/bmr1000/bmr1000.str",
                    "tests/fixtures/bmr1000.str",
                ),
        );

        Self { http_client }
    }
}

impl DataSource for MockBmrbDataSource {
    fn get_available_entries(&self) -> Result<Vec<String>, DownloadError> {
        let html = self
            .http_client
            .get("https://bmrb.io/ftp/pub/bmrb/entry_directories/")?;
        let mut entries = std::collections::HashSet::new();

        // Parse directory names like bmr12345/
        for cap in html.match_indices("bmr") {
            let start = cap.0;
            let rest = &html[start..];
            if let Some(end) = rest.find('/') {
                let dir = &rest[..end];
                if dir.len() > 3
                    && dir.starts_with("bmr")
                    && dir[3..].chars().all(|c| c.is_ascii_digit())
                {
                    entries.insert(dir.to_string());
                }
            }
        }

        if entries.is_empty() {
            return Err(DownloadError::NoEntriesFound);
        }

        let mut entries_vec: Vec<_> = entries.into_iter().collect();
        entries_vec.sort();
        Ok(entries_vec)
    }

    fn download_entry(
        &self,
        entry_id: &str,
        output_path: &PathBuf,
    ) -> Result<PathBuf, DownloadError> {
        let url = format!(
            "https://bmrb.io/ftp/pub/bmrb/entry_directories/{}/{}.str",
            entry_id, entry_id
        );
        let content = self.http_client.get(&url)?;

        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(output_path, content)?;
        Ok(output_path.clone())
    }
}

// Mock COD data source for testing
struct MockCodDataSource {
    http_client: Arc<MockHttpClient>,
}

impl MockCodDataSource {
    fn new_with_fixtures() -> Self {
        let http_client = Arc::new(
            MockHttpClient::new()
                .with_file_response(
                    "http://www.crystallography.net/cod/result.php?start=1&stop=50000&selection=id",
                    "tests/fixtures/cod_search_page.html",
                )
                .with_file_response(
                    "http://www.crystallography.net/cod/1000001.cif",
                    "tests/fixtures/1000001.cif",
                ),
        );

        Self { http_client }
    }
}

impl DataSource for MockCodDataSource {
    fn get_available_entries(&self) -> Result<Vec<String>, DownloadError> {
        let url = "http://www.crystallography.net/cod/result.php?start=1&stop=50000&selection=id";
        let html = self.http_client.get(url)?;

        // Simple regex parsing for test fixtures
        let mut entries = Vec::new();
        for cap in html.match_indices("/cod/") {
            let start = cap.0;
            let rest = &html[start + 5..]; // Skip "/cod/"
            if let Some(end) = rest.find('.') {
                let id = &rest[..end];
                if id.chars().all(|c| c.is_ascii_digit()) {
                    entries.push(id.to_string());
                }
            }
        }

        if entries.is_empty() {
            return Err(DownloadError::NoEntriesFound);
        }

        Ok(entries)
    }

    fn download_entry(
        &self,
        entry_id: &str,
        output_path: &PathBuf,
    ) -> Result<PathBuf, DownloadError> {
        let url = format!("http://www.crystallography.net/cod/{}.cif", entry_id);
        let content = self.http_client.get(&url)?;

        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(output_path, content)?;
        Ok(output_path.clone())
    }
}

#[test]
fn test_mock_bmrb_downloader() {
    let data_source = MockBmrbDataSource::new_with_fixtures();

    // Test getting available entries
    let entries = data_source.get_available_entries().unwrap();
    assert_eq!(entries, vec!["bmr1000", "bmr2000", "bmr3000"]);

    // Test downloading a file
    let temp_dir = std::env::temp_dir().join("mock_bmrb_test");
    let output_path = temp_dir.join("bmr1000.str");

    let result = data_source.download_entry("bmr1000", &output_path).unwrap();
    assert_eq!(result, output_path);
    assert!(output_path.exists());

    // Verify file content
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("data_1000"));
    assert!(content.contains("save_entry_information"));

    // Clean up
    std::fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn test_mock_cod_downloader() {
    let data_source = MockCodDataSource::new_with_fixtures();

    // Test getting available entries
    let entries = data_source.get_available_entries().unwrap();
    assert_eq!(entries, vec!["1000001", "2000002", "3000003", "4000004"]);

    // Test downloading a file
    let temp_dir = std::env::temp_dir().join("mock_cod_test");
    let output_path = temp_dir.join("1000001.cif");

    let result = data_source.download_entry("1000001", &output_path).unwrap();
    assert_eq!(result, output_path);
    assert!(output_path.exists());

    // Verify file content
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("data_1000001"));
    assert!(content.contains("_database_code_COD"));

    // Clean up
    std::fs::remove_dir_all(&temp_dir).unwrap();
}
