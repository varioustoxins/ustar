use clap::Parser;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use reqwest;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::collections::HashSet;

#[derive(Debug)]
pub enum PdbError {
    RequestError(reqwest::Error),
    IoError(std::io::Error),
    NoEntriesFound,
    DownloadFailed(String),
    JsonError(serde_json::Error),
}

impl From<reqwest::Error> for PdbError {
    fn from(err: reqwest::Error) -> Self {
        PdbError::RequestError(err)
    }
}

impl From<std::io::Error> for PdbError {
    fn from(err: std::io::Error) -> Self {
        PdbError::IoError(err)
    }
}

impl From<serde_json::Error> for PdbError {
    fn from(err: serde_json::Error) -> Self {
        PdbError::JsonError(err)
    }
}

impl std::fmt::Display for PdbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdbError::RequestError(e) => write!(f, "Request error: {}", e),
            PdbError::IoError(e) => write!(f, "IO error: {}", e),
            PdbError::NoEntriesFound => write!(f, "No PDB entries found"),
            PdbError::DownloadFailed(msg) => write!(f, "Download failed: {}", msg),
            PdbError::JsonError(e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl std::error::Error for PdbError {}

/// Response from PDB search API
#[derive(Debug, Deserialize)]
struct SearchResponse {
    result_set: Vec<ResultEntry>,
}

#[derive(Debug, Deserialize)]
struct ResultEntry {
    identifier: String,
}

/// Structure to hold PDB download configuration
pub struct PdbDownloader {
    base_url: String,
    search_url: String,
    output_dir: PathBuf,
    verbose: bool,
    compressed: bool,
}

impl PdbDownloader {
    /// Create a new PDB downloader with default settings
    pub fn new() -> Self {
        Self {
            base_url: "https://files.rcsb.org/download".to_string(),
            search_url: "https://search.rcsb.org/rcsbsearch/v2/query".to_string(),
            output_dir: PathBuf::from("."),
            verbose: true,
            compressed: false,
        }
    }

    /// Set the output directory
    pub fn output_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Set verbose mode
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set whether to download compressed files
    pub fn compressed(mut self, compressed: bool) -> Self {
        self.compressed = compressed;
        self
    }

    /// Get a list of all PDB entry IDs (or a subset if too many)
    pub async fn get_entry_ids(&self, max_entries: Option<usize>) -> Result<Vec<String>, PdbError> {
        if self.verbose {
            println!("Fetching list of PDB entries with pagination...");
        }

        let mut all_entries = Vec::new();
        let mut start = 0;
        let page_size = 10_000;
        let mut page = 1;
        let client = reqwest::Client::new();
        loop {
            let query = serde_json::json!({
                "query": {
                    "type": "terminal",
                    "service": "text",
                    "parameters": {
                        "attribute": "rcsb_entry_container_identifiers.entry_id",
                        "operator": "exists"
                    }
                },
                "return_type": "entry",
                "request_options": {
                    "paginate": {
                        "start": start,
                        "rows": page_size
                    }
                }
            });
            if self.verbose {
                println!("[VERBOSE] Query #{} URL: {}", page, &self.search_url);
                println!("[VERBOSE] Query #{} body: {}", page, query);
            }
            let response = client
                .post(&self.search_url)
                .header("Content-Type", "application/json")
                .json(&query)
                .send()
                .await?;
            if response.status() != reqwest::StatusCode::OK {
                return Err(PdbError::DownloadFailed(
                    format!("Failed to fetch entry list: HTTP {}", response.status())
                ));
            }
            let search_response: SearchResponse = response.json().await?;
            let count = search_response.result_set.len();
            if self.verbose {
                println!("[VERBOSE] Query #{} returned {} entries", page, count);
            }
            if count == 0 {
                break;
            }
            all_entries.extend(
                search_response
                    .result_set
                    .into_iter()
                    .map(|entry| entry.identifier)
            );
            if count < page_size {
                break;
            }
            start += page_size;
            page += 1;
            if let Some(max) = max_entries {
                if all_entries.len() >= max {
                    all_entries.truncate(max);
                    break;
                }
            }
        }
        if self.verbose {
            println!("[VERBOSE] Total entries fetched: {}", all_entries.len());
        }
        Ok(all_entries)
    }


    /// Get current PDB holdings (list of all valid PDB IDs)
    pub async fn get_current_holdings(&self) -> Result<Vec<String>, PdbError> {
        if self.verbose {
            println!("Fetching current PDB holdings list...");
        }

        // PDB provides a simple text file with all current IDs
        let url = "https://files.rcsb.org/pub/pdb/holdings/current_holdings.txt";
        let response = reqwest::get(url).await?;

        if response.status() != reqwest::StatusCode::OK {
            return Err(PdbError::DownloadFailed(
                format!("Failed to fetch holdings: HTTP {}", response.status())
            ));
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
    }

    /// Download a specific PDB entry in mmCIF format
    pub async fn download_entry(&self, pdb_id: &str) -> Result<PathBuf, PdbError> {
        let pdb_id = pdb_id.to_lowercase();
        
        // Determine file extension and URL based on compression preference
        let (extension, url) = if self.compressed {
            ("cif.gz", format!("{}/{}.cif.gz", self.base_url, pdb_id))
        } else {
            ("cif", format!("{}/{}.cif", self.base_url, pdb_id))
        };

        if self.verbose {
            println!("[VERBOSE] Downloading PDB entry {} in mmCIF format...", pdb_id);
            println!("[VERBOSE] Download URL: {}", url);
        }

        let response = reqwest::get(&url).await?;

        if response.status() != reqwest::StatusCode::OK {
            // Try alternative URL pattern (sometimes PDB uses different conventions)
            let alt_url = format!("https://files.rcsb.org/view/{}.{}", pdb_id, extension);
            if self.verbose {
                println!("[VERBOSE] First URL failed, trying alternative: {}", alt_url);
            }
            let alt_response = reqwest::get(&alt_url).await?;
            if alt_response.status() != reqwest::StatusCode::OK {
                return Err(PdbError::DownloadFailed(
                    format!("Failed to download PDB entry {}: HTTP {}", pdb_id, alt_response.status())
                ));
            }
            if self.verbose {
                println!("[VERBOSE] Downloaded from alternative URL: {}", alt_url);
            }
            self.save_response(alt_response, &pdb_id, extension).await
        } else {
            self.save_response(response, &pdb_id, extension).await
        }
    }

    /// Save the response content to a file
    async fn save_response(&self, response: reqwest::Response, pdb_id: &str, extension: &str) -> Result<PathBuf, PdbError> {
        let content = response.bytes().await?;

        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir)?;

        // Save the file
        let filename = format!("{}.{}", pdb_id, extension);
        let filepath = self.output_dir.join(&filename);

        let mut file = fs::File::create(&filepath)?;
        file.write_all(&content)?;

        if self.verbose {
            println!("Successfully saved {} ({} bytes)", filepath.display(), content.len());
        }

        Ok(filepath)
    }


    /// Download unique random mmCIF files, skipping those already in output_dir
    pub async fn download_unique_random_batch(&self, count: usize, seed: u64) -> Result<Vec<(String, PathBuf)>, PdbError> {
        let mut entries: Vec<String> = match self.get_current_holdings().await {
            Ok(e) => e,
            Err(_) => self.get_entry_ids(None).await?
        };
        if entries.is_empty() {
            return Err(PdbError::NoEntriesFound);
        }
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        entries.shuffle(&mut rng);
        let mut results: Vec<(String, PathBuf)> = Vec::new();
        let mut tried = std::collections::HashSet::new();
        for pdb_id in entries.into_iter() {
            if results.len() >= count {
                break;
            }
            if !tried.insert(pdb_id.clone()) {
                continue;
            }
            let ext = if self.compressed { "cif.gz" } else { "cif" };
            let filename = format!("{}.{}", pdb_id, ext);
            let filepath = self.output_dir.join(&filename);
            if filepath.exists() {
                if self.verbose {
                    println!("Already exists, skipping: {}", filepath.display());
                }
                continue;
            }
            match self.download_entry(&pdb_id).await {
                Ok(path) => results.push((pdb_id, path)),
                Err(e) => {
                    if self.verbose {
                        eprintln!("Failed to download: {}", e);
                    }
                }
            }
        }
        Ok(results)
    }

}

/// --- Minimal CLI ---
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Number of files to download
    #[arg(default_value_t = 50, value_name = "COUNT")]
    count: usize,
    /// Output directory
    #[arg(short, long, default_value = "tests/test_data/pdb_mmcifs")]
    output_dir: String,
    /// Download compressed .cif.gz files
    #[arg(long)]
    compressed: bool,
    /// Enable verbose output
    #[arg(long)]
    verbose: bool,
    /// List available mmCIF files and which are downloaded
    #[arg(long)]
    list: bool,
    /// Random number seed for reproducible shuffling
    #[arg(long, default_value_t = 42)]
    seed: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let downloader = PdbDownloader::new()
        .output_dir(&cli.output_dir)
        .verbose(cli.verbose)
        .compressed(cli.compressed);

    if cli.list {
        // List available mmCIF files and which are downloaded
        let entries = match downloader.get_current_holdings().await {
            Ok(e) => e,
            Err(_) => downloader.get_entry_ids(Some(10000)).await?
        };
        let ext = if cli.compressed { "cif.gz" } else { "cif" };
        let mut downloaded = HashSet::new();
        if let Ok(dir_entries) = std::fs::read_dir(&cli.output_dir) {
            for entry in dir_entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(ext) {
                        if let Some(stem) = name.strip_suffix(&format!(".{}", ext)) {
                            downloaded.insert(stem.to_lowercase());
                        }
                    }
                }
            }
        }
        if cli.verbose {
            println!("[VERBOSE] Total available mmCIF files: {}", entries.len());
        } else {
            println!("Total available mmCIF files: {}", entries.len());
        }
        let mut downloaded_count = 0;
        for pdb_id in &entries {
            let is_downloaded = downloaded.contains(&pdb_id.to_lowercase());
            if is_downloaded {
                downloaded_count += 1;
            }
            if cli.verbose {
                println!("[VERBOSE] {}{}", pdb_id, if is_downloaded { " [downloaded]" } else { "" });
            } else {
                println!("{}{}", pdb_id, if is_downloaded { " [downloaded]" } else { "" });
            }
        }
        if cli.verbose {
            println!("[VERBOSE] Total downloaded: {} / {}", downloaded_count, entries.len());
        } else {
            println!("\nTotal downloaded: {} / {}", downloaded_count, entries.len());
        }
        return Ok(());
    }

    if cli.verbose {
        println!("[VERBOSE] Downloading {} unique random mmCIF files to {}...", cli.count, cli.output_dir);
    } else {
        println!("Downloading {} unique random mmCIF files to {}...", cli.count, cli.output_dir);
    }
    let batch = downloader.download_unique_random_batch(cli.count, cli.seed).await?;
    if cli.verbose {
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