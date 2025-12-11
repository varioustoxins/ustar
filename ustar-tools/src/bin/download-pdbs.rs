use clap::Parser;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use ustar_tools::downloader_common::{
    CommonDownloaderCli, DataSource, DownloadError, DownloaderConfig, GenericDownloader,
};

/// PDB-specific data source implementation
pub struct PdbDataSource {
    base_url: String,
    compressed: bool,
    verbose: bool,
}

impl PdbDataSource {
    pub fn new(compressed: bool, verbose: bool) -> Self {
        Self {
            base_url: "https://files.rcsb.org/download".to_string(),
            compressed,
            verbose,
        }
    }
}

impl DataSource for PdbDataSource {
    fn get_available_entries(&self) -> Result<Vec<String>, DownloadError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            DownloadError::DownloadFailed(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            if self.verbose {
                println!("Fetching current PDB holdings list...");
            }

            let url = "https://files.rcsb.org/pub/pdb/holdings/current_holdings.txt";
            let response = reqwest::get(url).await?;

            if response.status() != reqwest::StatusCode::OK {
                return Err(DownloadError::DownloadFailed(format!(
                    "Failed to fetch holdings: HTTP {}",
                    response.status()
                )));
            }

            let text = response.text().await?;
            let entries: Vec<String> = text
                .lines()
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(|line| line.to_lowercase())
                .collect();

            if self.verbose {
                println!("Found {} PDB entries in current holdings", entries.len());
            }

            Ok(entries)
        })
    }

    fn download_entry(
        &self,
        pdb_id: &str,
        output_path: &PathBuf,
    ) -> Result<PathBuf, DownloadError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            DownloadError::DownloadFailed(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            let pdb_id = pdb_id.to_lowercase();

            let (extension, url) = if self.compressed {
                ("cif.gz", format!("{}/{}.cif.gz", self.base_url, pdb_id))
            } else {
                ("cif", format!("{}/{}.cif", self.base_url, pdb_id))
            };

            if self.verbose {
                println!(
                    "[VERBOSE] Downloading PDB entry {} in mmCIF format...",
                    pdb_id
                );
                println!("[VERBOSE] Download URL: {}", url);
            }

            let response = reqwest::get(&url).await?;

            if response.status() != reqwest::StatusCode::OK {
                let alt_url = format!("https://files.rcsb.org/view/{}.{}", pdb_id, extension);
                if self.verbose {
                    println!(
                        "[VERBOSE] First URL failed, trying alternative: {}",
                        alt_url
                    );
                }
                let alt_response = reqwest::get(&alt_url).await?;
                if alt_response.status() != reqwest::StatusCode::OK {
                    return Err(DownloadError::DownloadFailed(format!(
                        "Failed to download PDB entry {}: HTTP {}",
                        pdb_id,
                        alt_response.status()
                    )));
                }
                return self.save_response(alt_response, output_path).await;
            }

            self.save_response(response, output_path).await
        })
    }
}

impl PdbDataSource {
    async fn save_response(
        &self,
        response: reqwest::Response,
        output_path: &PathBuf,
    ) -> Result<PathBuf, DownloadError> {
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
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Download PDB mmCIF files", long_about = None)]
struct Cli {
    #[command(flatten)]
    common: CommonDownloaderCli,
    /// Download compressed .cif.gz files
    #[arg(long)]
    compressed: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            common: CommonDownloaderCli {
                count: 50,
                output_dir: "tests/test_data/pdb_mmcifs".to_string(),
                verbose: false,
                list: false,
                seed: 42,
            },
            compressed: false,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let file_extension = if cli.compressed {
        "cif.gz".to_string()
    } else {
        "cif".to_string()
    };

    let config = DownloaderConfig::new()
        .output_dir(&cli.common.output_dir)
        .verbose(cli.common.verbose)
        .file_extension(file_extension);

    let data_source = PdbDataSource::new(cli.compressed, cli.common.verbose);
    let downloader = GenericDownloader::new(config, data_source);

    if cli.common.list {
        downloader.list_files()?;
        return Ok(());
    }

    if cli.common.verbose {
        println!(
            "[VERBOSE] Downloading {} unique random mmCIF files to {}...",
            cli.common.count, cli.common.output_dir
        );
    } else {
        println!(
            "Downloading {} unique random mmCIF files to {}...",
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
