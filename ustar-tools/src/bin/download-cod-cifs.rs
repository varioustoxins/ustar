use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use ustar_tools::downloader_common::{
    CommonDownloaderCli, DataSource, DownloadError, DownloaderConfig, GenericDownloader,
    HttpClient, ReqwestClient,
};

/// COD-specific data source implementation
pub struct CodDataSource {
    verbose: bool,
    http_client: Arc<dyn HttpClient>,
}

impl CodDataSource {
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            http_client: Arc::new(ReqwestClient),
        }
    }

    #[cfg(test)]
    pub fn with_client(verbose: bool, client: Arc<dyn HttpClient>) -> Self {
        Self {
            verbose,
            http_client: client,
        }
    }
}

impl DataSource for CodDataSource {
    fn get_available_entries(&self) -> Result<Vec<String>, DownloadError> {
        if self.verbose {
            println!("Getting list of valid COD IDs...");
        }

        // For mock compatibility, just fetch one page
        let url = "http://www.crystallography.net/cod/result.php?start=1&stop=50000&selection=id";

        if self.verbose {
            println!("Fetching COD database page...");
        }

        let html = self.http_client.get(url)?;

        // Compile regex for parsing COD IDs
        let cod_regex = Regex::new(r"cod/(\d{7})\.cif")
            .map_err(|e| DownloadError::DownloadFailed(format!("Regex error: {}", e)))?;

        let all_ids: Vec<String> = cod_regex
            .captures_iter(&html)
            .map(|cap| cap[1].to_string())
            .collect();

        if self.verbose {
            println!("Found {} COD entries", all_ids.len());
        }

        if all_ids.is_empty() {
            return Err(DownloadError::NoEntriesFound);
        }

        // Remove duplicates and sort
        let mut unique_ids = all_ids;
        unique_ids.sort_unstable();
        unique_ids.dedup();

        Ok(unique_ids)
    }

    fn download_entry(
        &self,
        entry_id: &str,
        output_path: &PathBuf,
    ) -> Result<PathBuf, DownloadError> {
        let url = format!("http://www.crystallography.net/cod/{}.cif", entry_id);

        if self.verbose {
            println!(
                "[VERBOSE] Downloading COD entry {} from {}...",
                entry_id, url
            );
        }

        let content = self.http_client.get(&url)?;

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

        Ok(output_path.clone())
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
