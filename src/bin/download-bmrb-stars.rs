impl std::fmt::Display for BmrbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BmrbError::RequestError(e) => write!(f, "Request error: {}", e),
            BmrbError::IoError(e) => write!(f, "IO error: {}", e),
            BmrbError::NoEntriesFound => write!(f, "No BMRB entries found"),
            BmrbError::DownloadFailed(msg) => write!(f, "Download failed: {}", msg),
        }
    }
}

use clap::Parser;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use reqwest;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::collections::HashSet;

#[derive(Debug)]
pub enum BmrbError {
    RequestError(reqwest::Error),
    IoError(std::io::Error),
    NoEntriesFound,
    DownloadFailed(String),
}

impl From<reqwest::Error> for BmrbError {
    fn from(err: reqwest::Error) -> Self {
        BmrbError::RequestError(err)
    }
}

impl From<std::io::Error> for BmrbError {
    fn from(err: std::io::Error) -> Self {
        BmrbError::IoError(err)
    }
}

impl std::error::Error for BmrbError {}

pub struct BmrbDownloader {
    output_dir: PathBuf,
    verbose: bool,
}

impl BmrbDownloader {
    pub fn new() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            verbose: true,
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

    pub async fn get_entry_ids(&self) -> Result<Vec<String>, BmrbError> {
        let url = "https://bmrb.io/ftp/pub/bmrb/entry_directories/";
        if self.verbose {
            println!("Fetching list of available BMRB FTP directories from {}...", url);
        }
        let response = reqwest::get(url).await?;
        if response.status() != reqwest::StatusCode::OK {
            return Err(BmrbError::DownloadFailed(
                format!("Failed to fetch FTP directory list: HTTP {}", response.status())
            ));
        }
        let html = response.text().await?;
        // Parse directory names like bmr12345/
        let mut entries = Vec::new();
        for cap in html.match_indices("bmr") {
            let start = cap.0;
            let rest = &html[start..];
            if let Some(end) = rest.find('/') {
                let dir = &rest[..end];
                if dir.len() > 3 && dir[3..].chars().all(|c| c.is_ascii_digit()) {
                    entries.push(dir[3..].to_string());
                }
            }
        }
        entries.sort();
        entries.dedup();
        if self.verbose {
            println!("Found {} BMRB FTP directories", entries.len());
        }
        Ok(entries)
    }

    pub async fn download_entry(&self, bmrb_id: &str) -> Result<PathBuf, BmrbError> {
        // Try the _3.str file first, as this is the main NMR-STAR file
        let ftp_url = format!(
            "https://bmrb.io/ftp/pub/bmrb/entry_directories/bmr{0}/bmr{0}_3.str",
            bmrb_id
        );
        if self.verbose {
            println!("Trying FTP URL: {}", ftp_url);
        }
        let ftp_response = reqwest::get(&ftp_url).await?;
        let (content, use_3) = if ftp_response.status() == reqwest::StatusCode::OK {
            (ftp_response.text().await?, true)
        } else {
            // Fallback: try bmr{id}.str (legacy or alternate naming)
            let alt_url = format!(
                "https://bmrb.io/ftp/pub/bmrb/entry_directories/bmr{0}/bmr{0}.str",
                bmrb_id
            );
            if self.verbose {
                println!("_3.str not found, trying alternate: {}", alt_url);
            }
            let alt_response = reqwest::get(&alt_url).await?;
            if alt_response.status() != reqwest::StatusCode::OK {
                return Err(BmrbError::DownloadFailed(
                    format!("Failed to download entry {}: HTTP {}", bmrb_id, alt_response.status())
                ));
            }
            (alt_response.text().await?, false)
        };
        fs::create_dir_all(&self.output_dir)?;
        // Save as bmr{id}_3.str if that was successful, else as bmr{id}.str
        let filename = if use_3 {
            format!("bmr{}_3.str", bmrb_id)
        } else {
            format!("bmr{}.str", bmrb_id)
        };
        let filepath = self.output_dir.join(&filename);
        let mut file = fs::File::create(&filepath)?;
        file.write_all(content.as_bytes())?;
        if self.verbose {
            println!("Successfully saved {} ({} bytes)", filepath.display(), content.len());
        }
        Ok(filepath)
    }

    pub async fn download_unique_random_batch(&self, count: usize, seed: u64) -> Result<Vec<(String, PathBuf)>, BmrbError> {
        let mut entries = self.get_entry_ids().await?;
        if entries.is_empty() {
            return Err(BmrbError::NoEntriesFound);
        }
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        entries.shuffle(&mut rng);
        let mut results = Vec::new();
        let mut tried = HashSet::new();
        for bmrb_id in entries.into_iter() {
            if results.len() >= count {
                break;
            }
            if !tried.insert(bmrb_id.clone()) {
                continue;
            }
            let filename = format!("bmr{}.str", bmrb_id);
            let filepath = self.output_dir.join(&filename);
            if filepath.exists() {
                if self.verbose {
                    println!("Already exists, skipping: {}", filepath.display());
                }
                continue;
            }
            match self.download_entry(&bmrb_id).await {
                Ok(path) => results.push((bmrb_id, path)),
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Number of files to download
    #[arg(default_value_t = 50, value_name = "COUNT")]
    count: usize,
    /// Output directory
    #[arg(short, long, default_value = "tests/test_data/bmrb_stars")]
    output_dir: String,
    /// Enable verbose output
    #[arg(long)]
    verbose: bool,
    /// List available BMRB files and which are downloaded
    #[arg(long)]
    list: bool,
    /// Random number seed for reproducible shuffling
    #[arg(long, default_value_t = 42)]
    seed: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let downloader = BmrbDownloader::new()
        .output_dir(&cli.output_dir)
        .verbose(cli.verbose);

    if cli.list {
        let entries = downloader.get_entry_ids().await?;
        let mut downloaded = HashSet::new();
        if let Ok(dir_entries) = std::fs::read_dir(&cli.output_dir) {
            for entry in dir_entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".str") {
                        if let Some(stem) = name.strip_prefix("bmr") {
                            let id = stem.trim_end_matches(".str");
                            downloaded.insert(id.to_string());
                        }
                    }
                }
            }
        }
        if cli.verbose {
            println!("[VERBOSE] Total available BMRB files: {}", entries.len());
        } else {
            println!("Total available BMRB files: {}", entries.len());
        }
        let mut downloaded_count = 0;
        for bmrb_id in &entries {
            let is_downloaded = downloaded.contains(bmrb_id);
            if is_downloaded {
                downloaded_count += 1;
            }
            if cli.verbose {
                println!("[VERBOSE] {}{}", bmrb_id, if is_downloaded { " [downloaded]" } else { "" });
            } else {
                println!("{}{}", bmrb_id, if is_downloaded { " [downloaded]" } else { "" });
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
        println!("[VERBOSE] Downloading {} unique random BMRB NMR-STAR files to {}...", cli.count, cli.output_dir);
    } else {
        println!("Downloading {} unique random BMRB NMR-STAR files to {}...", cli.count, cli.output_dir);
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
