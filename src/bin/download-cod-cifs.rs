use rand::Rng;
use reqwest;
// use rand::seq::SliceRandom; // No longer needed, use rand::Rng and shuffle as in download-pdbs
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use clap::Parser;
use scraper::{Html, Selector};


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Number of CIF files to download
    #[arg(default_value_t = 50, value_name = "COUNT")]
    count: usize,
    /// Output directory
    #[arg(short, long, default_value = "tests/test_data/cod_cif_files")]
    output_dir: String,
    /// Enable verbose output
    #[arg(long)]
    verbose: bool,
    /// List available COD CIF files and which are downloaded
    #[arg(long)]
    list: bool,
    /// Random number seed for reproducible shuffling
    #[arg(long, default_value_t = 42)]
    seed: u64,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    fs::create_dir_all(&cli.output_dir)?;

    if cli.verbose {
        println!("Getting list of valid COD IDs...");
    }
    let valid_ids = get_cod_ids().await?;
    if cli.verbose {
        println!("Found {} valid COD IDs", valid_ids.len());
    }
    if valid_ids.is_empty() {
        eprintln!("No valid COD IDs found!");
        return Ok(());
    }

    if cli.list {
        // List available COD CIF files and which are downloaded
        let mut downloaded = HashSet::new();
        let ext = "cif";
        if let Ok(dir_entries) = std::fs::read_dir(&cli.output_dir) {
            for entry in dir_entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("cod_") && name.ends_with(ext) {
                        if let Some(stem) = name.strip_prefix("cod_").and_then(|n| n.strip_suffix(&format!(".{}", ext))) {
                            downloaded.insert(stem.to_string());
                        }
                    }
                }
            }
        }
        if cli.verbose {
            println!("[VERBOSE] Total available COD CIF files: {}", valid_ids.len());
        } else {
            println!("Total available COD CIF files: {}", valid_ids.len());
        }
        let mut downloaded_count = 0;
        for cod_id in &valid_ids {
            let is_downloaded = downloaded.contains(cod_id);
            if is_downloaded {
                downloaded_count += 1;
            }
            if cli.verbose {
                println!("[VERBOSE] {}{}", cod_id, if is_downloaded { " [downloaded]" } else { "" });
            } else {
                println!("{}{}", cod_id, if is_downloaded { " [downloaded]" } else { "" });
            }
        }
        if cli.verbose {
            println!("[VERBOSE] Total downloaded: {} / {}", downloaded_count, valid_ids.len());
        } else {
            println!("\nTotal downloaded: {} / {}", downloaded_count, valid_ids.len());
        }
        return Ok(());
    }

    if cli.verbose {
        println!("[VERBOSE] Downloading {} unique random COD CIF files to {}...", cli.count, cli.output_dir);
    } else {
        println!("Downloading {} unique random COD CIF files to {}...", cli.count, cli.output_dir);
    }
    let batch = download_unique_random_cod_batch(valid_ids, cli.count, &cli.output_dir, cli.seed, cli.verbose).await?;
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

async fn get_cod_ids() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    // Step 1: Fetch the search results page (wildcard search for all entries)
    let url = "https://www.crystallography.net/cod/result?text=%25";
    println!("Fetching COD search results page...");
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(format!("Failed to fetch COD search page: HTTP {}", response.status()).into());
    }
    let html = response.text().await?;

    // Step 2: Parse HTML to find the download link for 'list of cod numbers'
    let document = Html::parse_document(&html);
    let selector = Selector::parse("a").unwrap();
    let mut download_url = None;
    for element in document.select(&selector) {
        if let Some(text) = element.text().next() {
            if text.to_ascii_lowercase().contains("list of cod numbers") {
                if let Some(href) = element.value().attr("href") {
                    download_url = Some(href.to_string());
                    break;
                }
            }
        }
    }
    let download_url = match download_url {
        Some(url) => {
            if url.starts_with("http") {
                url
            } else {
                format!("https://www.crystallography.net/{}", url.trim_start_matches('/'))
            }
        },
        None => return Err("Could not find download link for COD numbers list".into()),
    };

    println!("Fetching COD numbers list from: {}", download_url);
    let response = client.get(&download_url).send().await?;
    if !response.status().is_success() {
        return Err(format!("Failed to fetch COD numbers list: HTTP {}", response.status()).into());
    }
    let text = response.text().await?;
    let re = Regex::new(r"\d{7}")?;
    let ids: Vec<String> = re.find_iter(&text)
        .map(|m| m.as_str().to_string())
        .collect();
    let unique_ids: HashSet<String> = ids.into_iter().collect();
    Ok(unique_ids.into_iter().collect())
}


use rand::SeedableRng;

async fn download_unique_random_cod_batch(
    valid_ids: Vec<String>,
    n: usize,
    output_dir: &str,
    seed: u64,
    verbose: bool,
) -> Result<Vec<(String, std::path::PathBuf)>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let mut ids = valid_ids;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    // Fisher-Yates shuffle, as in download-pdbs
    for i in (1..ids.len()).rev() {
        let j = rng.gen_range(0..=i);
        ids.swap(i, j);
    }
    let mut results = Vec::new();
    let mut tried = HashSet::new();
    for cod_id in ids.into_iter() {
        if results.len() >= n {
            break;
        }
        if !tried.insert(cod_id.clone()) {
            continue;
        }
        let filename = format!("cod_{}.cif", cod_id);
        let filepath = Path::new(output_dir).join(&filename);
        if filepath.exists() {
            if verbose {
                println!("Already exists, skipping: {}", filepath.display());
            }
            continue;
        }
        match client.get(&format!("http://www.crystallography.net/cod/{}.cif", cod_id)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let content = response.text().await?;
                    fs::write(&filepath, content)?;
                    if verbose {
                        println!("Downloaded: {}", filename);
                    }
                    results.push((cod_id, filepath));
                    sleep(Duration::from_secs(1)).await;
                } else if verbose {
                    println!("Failed to download COD {}: HTTP {}", cod_id, response.status());
                }
            },
            Err(e) => {
                if verbose {
                    println!("Failed to download COD {}: {}", cod_id, e);
                }
            }
        }
    }
    Ok(results)
}