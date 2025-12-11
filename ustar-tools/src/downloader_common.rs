// Common downloader traits and utilities for STAR/CIF file downloads

use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Common CLI structure for all downloaders
#[derive(clap::Parser, Debug)]
pub struct CommonDownloaderCli {
    /// Number of files to download
    #[arg(default_value_t = 50, value_name = "COUNT")]
    pub count: usize,
    /// Output directory
    #[arg(short, long, default_value = "tests/test_data")]
    pub output_dir: String,
    /// Enable verbose output
    #[arg(long)]
    pub verbose: bool,
    /// List available files and which are downloaded
    #[arg(long)]
    pub list: bool,
    /// Random number seed for reproducible shuffling
    #[arg(long, default_value_t = 42)]
    pub seed: u64,
}

/// Error type for download operations
#[derive(Debug)]
pub enum DownloadError {
    RequestError(reqwest::Error),
    IoError(std::io::Error),
    NoEntriesFound,
    DownloadFailed(String),
    JsonError(serde_json::Error),
}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        DownloadError::RequestError(err)
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(err: std::io::Error) -> Self {
        DownloadError::IoError(err)
    }
}

impl From<serde_json::Error> for DownloadError {
    fn from(err: serde_json::Error) -> Self {
        DownloadError::JsonError(err)
    }
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::RequestError(e) => write!(f, "Request error: {}", e),
            DownloadError::IoError(e) => write!(f, "IO error: {}", e),
            DownloadError::NoEntriesFound => write!(f, "No entries found"),
            DownloadError::DownloadFailed(msg) => write!(f, "Download failed: {}", msg),
            DownloadError::JsonError(e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl std::error::Error for DownloadError {}

/// Configuration for a downloader
pub struct DownloaderConfig {
    pub output_dir: PathBuf,
    pub verbose: bool,
    pub file_extension: String,
}

impl DownloaderConfig {
    pub fn new() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            verbose: true,
            file_extension: "cif".to_string(),
        }
    }

    pub fn output_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.output_dir = dir.into();
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn file_extension<S: Into<String>>(mut self, ext: S) -> Self {
        self.file_extension = ext.into();
        self
    }
}

/// Trait for different data source strategies
pub trait DataSource {
    fn get_available_entries(&self) -> Result<Vec<String>, DownloadError>;
    fn download_entry(
        &self,
        entry_id: &str,
        output_path: &PathBuf,
    ) -> Result<PathBuf, DownloadError>;
}

/// Common downloader implementation
pub struct GenericDownloader<T: DataSource> {
    config: DownloaderConfig,
    data_source: T,
}

impl<T: DataSource> GenericDownloader<T> {
    pub fn new(config: DownloaderConfig, data_source: T) -> Self {
        Self {
            config,
            data_source,
        }
    }

    /// Download unique random files, skipping those already in output_dir
    pub fn download_unique_random_batch(
        &self,
        count: usize,
        seed: u64,
    ) -> Result<Vec<(String, PathBuf)>, DownloadError> {
        let mut entries = self.data_source.get_available_entries()?;

        if entries.is_empty() {
            return Err(DownloadError::NoEntriesFound);
        }

        // Shuffle entries deterministically using seed
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        entries.shuffle(&mut rng);

        let mut results: Vec<(String, PathBuf)> = Vec::new();
        let mut tried = HashSet::new();

        for entry_id in entries.into_iter() {
            if results.len() >= count {
                break;
            }

            if !tried.insert(entry_id.clone()) {
                continue;
            }

            let filename = format!("{}.{}", entry_id, self.config.file_extension);
            let filepath = self.config.output_dir.join(&filename);

            if filepath.exists() {
                if self.config.verbose {
                    println!("Already exists, skipping: {}", filepath.display());
                }
                continue;
            }

            match self.data_source.download_entry(&entry_id, &filepath) {
                Ok(path) => results.push((entry_id, path)),
                Err(e) => {
                    if self.config.verbose {
                        eprintln!("Failed to download {}: {}", entry_id, e);
                    }
                }
            }
        }

        Ok(results)
    }

    /// List available files and show which are downloaded
    pub fn list_files(&self) -> Result<(), DownloadError> {
        let entries = self.data_source.get_available_entries()?;

        // Build set of already downloaded files
        let mut downloaded = HashSet::new();
        if let Ok(dir_entries) = fs::read_dir(&self.config.output_dir) {
            for entry in dir_entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(&self.config.file_extension) {
                        if let Some(stem) =
                            name.strip_suffix(&format!(".{}", self.config.file_extension))
                        {
                            downloaded.insert(stem.to_lowercase());
                        }
                    }
                }
            }
        }

        if self.config.verbose {
            println!("[VERBOSE] Total available files: {}", entries.len());
        } else {
            println!("Total available files: {}", entries.len());
        }

        let mut downloaded_count = 0;
        for entry_id in &entries {
            let is_downloaded = downloaded.contains(&entry_id.to_lowercase());
            if is_downloaded {
                downloaded_count += 1;
            }

            let status = if is_downloaded { " [downloaded]" } else { "" };
            if self.config.verbose {
                println!("[VERBOSE] {}{}", entry_id, status);
            } else {
                println!("{}{}", entry_id, status);
            }
        }

        if self.config.verbose {
            println!(
                "[VERBOSE] Total downloaded: {} / {}",
                downloaded_count,
                entries.len()
            );
        } else {
            println!(
                "\nTotal downloaded: {} / {}",
                downloaded_count,
                entries.len()
            );
        }

        Ok(())
    }
}
