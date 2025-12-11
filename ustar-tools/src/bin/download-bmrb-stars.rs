use clap::Parser;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use ustar_tools::downloader_common::{
    CommonDownloaderCli, DataSource, DownloadError, DownloaderConfig, GenericDownloader,
};

/// BMRB-specific data source implementation
pub struct BmrbDataSource {
    verbose: bool,
}

impl BmrbDataSource {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl DataSource for BmrbDataSource {
    fn get_available_entries(&self) -> Result<Vec<String>, DownloadError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            DownloadError::DownloadFailed(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            let url = "https://bmrb.io/ftp/pub/bmrb/entry_directories/";
            if self.verbose {
                println!(
                    "Fetching list of available BMRB FTP directories from {}...",
                    url
                );
            }

            let response = reqwest::get(url).await?;
            if response.status() != reqwest::StatusCode::OK {
                return Err(DownloadError::DownloadFailed(format!(
                    "Failed to fetch FTP directory list: HTTP {}",
                    response.status()
                )));
            }

            let html = response.text().await?;
            let mut entries = Vec::new();

            // Parse directory names like bmr12345/
            for cap in html.match_indices("bmr") {
                let start = cap.0;
                let rest = &html[start..];
                if let Some(end) = rest.find('/') {
                    let dir = &rest[..end];
                    if dir.len() > 3 && dir[3..].chars().all(|c| c.is_ascii_digit()) {
                        entries.push(dir.to_string());
                    }
                }
            }

            if self.verbose {
                println!("Found {} BMRB directories", entries.len());
            }

            if entries.is_empty() {
                return Err(DownloadError::NoEntriesFound);
            }

            Ok(entries)
        })
    }

    fn download_entry(
        &self,
        entry_id: &str,
        output_path: &PathBuf,
    ) -> Result<PathBuf, DownloadError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            DownloadError::DownloadFailed(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            let url = format!(
                "https://bmrb.io/ftp/pub/bmrb/entry_directories/{}/{}.str",
                entry_id, entry_id
            );

            if self.verbose {
                println!(
                    "[VERBOSE] Downloading BMRB entry {} from {}...",
                    entry_id, url
                );
            }

            let response = reqwest::get(&url).await?;

            if response.status() != reqwest::StatusCode::OK {
                return Err(DownloadError::DownloadFailed(format!(
                    "Failed to download BMRB entry {}: HTTP {}",
                    entry_id,
                    response.status()
                )));
            }

            let content = response.bytes().await?;

            // Create output directory if it doesn't exist
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = fs::File::create(output_path)?;
            file.write_all(&content)?;

            if self.verbose {
                println!(
                    "Successfully saved {} ({} bytes)",
                    output_path.display(),
                    content.len()
                );
            }

            Ok(output_path.clone())
        })
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Download BMRB NMR-STAR files", long_about = None)]
struct Cli {
    #[command(flatten)]
    common: CommonDownloaderCli,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            common: CommonDownloaderCli {
                count: 50,
                output_dir: "tests/test_data/bmrb_stars".to_string(),
                verbose: false,
                list: false,
                seed: 42,
            },
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = DownloaderConfig::new()
        .output_dir(&cli.common.output_dir)
        .verbose(cli.common.verbose)
        .file_extension("str");

    let data_source = BmrbDataSource::new(cli.common.verbose);
    let downloader = GenericDownloader::new(config, data_source);

    if cli.common.list {
        downloader.list_files()?;
        return Ok(());
    }

    if cli.common.verbose {
        println!(
            "[VERBOSE] Downloading {} unique random BMRB files to {}...",
            cli.common.count, cli.common.output_dir
        );
    } else {
        println!(
            "Downloading {} unique random BMRB files to {}...",
            cli.common.count, cli.common.output_dir
        );
    }

    let batch = downloader.download_unique_random_batch(cli.common.count, cli.common.seed)?;

    if cli.common.verbose {
        println!("[VERBOSE] Downloaded {} files:", batch.len());
        for (id, path) in &batch {
            println!("[VERBOSE] {} -> {}", id, path.display());
        }
    } else {
        for (id, path) in batch {
            println!("Downloaded {} to {}", id, path.display());
        }
    }

    Ok(())
}
