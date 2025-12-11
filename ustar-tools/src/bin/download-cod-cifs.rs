use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use ustar_tools::downloader_common::{
    CommonDownloaderCli, DataSource, DownloadError, DownloaderConfig, GenericDownloader,
};

/// COD-specific data source implementation
pub struct CodDataSource {
    verbose: bool,
}

impl CodDataSource {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl DataSource for CodDataSource {
    fn get_available_entries(&self) -> Result<Vec<String>, DownloadError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            DownloadError::DownloadFailed(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            if self.verbose {
                println!("Getting list of valid COD IDs...");
            }

            // Get the first page to determine total range
            let base_url = "http://www.crystallography.net/cod/result.php";
            let mut page = 1;
            let mut all_ids = Vec::new();

            // Compile regex once outside the loop
            let cod_regex = Regex::new(r"cod/(\d{7})\.cif")
                .map_err(|e| DownloadError::DownloadFailed(format!("Regex error: {}", e)))?;

            loop {
                let url = format!(
                    "{}?start={}&stop=50000&selection=id",
                    base_url,
                    (page - 1) * 50 + 1
                );

                if self.verbose {
                    println!("Fetching page {} from COD database...", page);
                }

                let response = reqwest::get(&url).await?;
                if response.status() != reqwest::StatusCode::OK {
                    return Err(DownloadError::DownloadFailed(format!(
                        "Failed to fetch COD page: HTTP {}",
                        response.status()
                    )));
                }

                let html = response.text().await?;

                let page_ids: Vec<String> = cod_regex
                    .captures_iter(&html)
                    .map(|cap| cap[1].to_string())
                    .collect();

                if page_ids.is_empty() {
                    break;
                }

                all_ids.extend(page_ids);

                // Add delay to be respectful to the COD server
                sleep(Duration::from_millis(500)).await;

                // For simplicity, limit to first few pages to avoid overwhelming the server
                page += 1;
                if page > 10 {
                    // Limit to prevent excessive requests
                    break;
                }
            }

            if self.verbose {
                println!("Found {} COD entries", all_ids.len());
            }

            if all_ids.is_empty() {
                return Err(DownloadError::NoEntriesFound);
            }

            // Remove duplicates and sort
            all_ids.sort_unstable();
            all_ids.dedup();

            Ok(all_ids)
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
            let url = format!("http://www.crystallography.net/cod/{}.cif", entry_id);

            if self.verbose {
                println!(
                    "[VERBOSE] Downloading COD entry {} from {}...",
                    entry_id, url
                );
            }

            let response = reqwest::get(&url).await?;

            if response.status() != reqwest::StatusCode::OK {
                return Err(DownloadError::DownloadFailed(format!(
                    "Failed to download COD entry {}: HTTP {}",
                    entry_id,
                    response.status()
                )));
            }

            let content = response.text().await?;

            // Create output directory if it doesn't exist
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(output_path, content)?;

            if self.verbose {
                let metadata = fs::metadata(output_path)?;
                println!(
                    "Successfully saved {} ({} bytes)",
                    output_path.display(),
                    metadata.len()
                );
            }

            // Small delay to be respectful to the server
            sleep(Duration::from_millis(200)).await;

            Ok(output_path.clone())
        })
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Download COD (Crystallography Open Database) CIF files", long_about = None)]
struct Cli {
    #[command(flatten)]
    common: CommonDownloaderCli,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            common: CommonDownloaderCli {
                count: 50,
                output_dir: "tests/test_data/cod_cif_files".to_string(),
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
        .file_extension("cif");

    let data_source = CodDataSource::new(cli.common.verbose);
    let downloader = GenericDownloader::new(config, data_source);

    if cli.common.list {
        downloader.list_files()?;
        return Ok(());
    }

    if cli.common.verbose {
        println!(
            "[VERBOSE] Downloading {} unique random COD CIF files to {}...",
            cli.common.count, cli.common.output_dir
        );
    } else {
        println!(
            "Downloading {} unique random COD CIF files to {}...",
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
