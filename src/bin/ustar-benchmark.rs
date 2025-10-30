use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use ustar::{StarParser, Rule};
use pest::Parser as PestParser;
use clap::Parser;

#[derive(Parser)]
#[command(name = "ustar-benchmark")]
#[command(about = "Benchmark STAR file parsing performance with baseline comparison")]
#[command(version = "0.1.0")]
struct Args {
    /// STAR file to benchmark
    file_path: String,
    
    /// Number of parsing iterations
    #[arg(short, long, default_value = "100")]
    iterations: usize,
    
    /// Show detailed timing information
    #[arg(short, long)]
    verbose: bool,
    
    /// Number of warmup cycles before measurement
    #[arg(short, long, default_value = "10")]
    warmup: usize,
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.file_path).exists() {
        eprintln!("Error: File '{}' does not exist", args.file_path);
        std::process::exit(1);
    }

    // Read the file once
    let content = match fs::read_to_string(&args.file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", args.file_path, e);
            std::process::exit(1);
        }
    };

    let file_size = content.len();
    
    // Establish baseline performance using simple_star_file.star
    let baseline_per_byte = establish_baseline();
    
    println!("STAR File Parsing Benchmark");
    println!("==========================");
    println!("File: {}", args.file_path);
    println!("Size: {} bytes ({:.2} KB)", file_size, file_size as f64 / 1024.0);
    println!("Iterations: {}", args.iterations);
    println!("Warmup cycles: {}", args.warmup);
    println!("Baseline: {:.2} ns/byte (from comprehensive_example.star)", baseline_per_byte);
    println!();

    // Warmup parse to ensure the file is valid
    print!("Validating file... ");
    match StarParser::parse(Rule::star_file, &content) {
        Ok(_) => println!("✓ Valid STAR file"),
        Err(e) => {
            eprintln!("✗ Parse error: {}", e);
            std::process::exit(1);
        }
    }

    // Warmup phase
    if args.warmup > 0 {
        println!();
        println!("Running warmup ({} cycles)...", args.warmup);
        for i in 0..args.warmup {
            if let Err(e) = StarParser::parse(Rule::star_file, &content) {
                eprintln!("Parse error during warmup iteration {}: {}", i + 1, e);
                std::process::exit(1);
            }
            if args.verbose && (i + 1) % (args.warmup / 5).max(1) == 0 {
                println!("  Warmup {}/{}", i + 1, args.warmup);
            }
        }
    }

    println!();
    println!("Running benchmark...");

    let mut parse_times: Vec<Duration> = Vec::with_capacity(args.iterations);
    let mut total_duration = Duration::new(0, 0);

    for i in 0..args.iterations {
        let start_time = Instant::now();
        
        match StarParser::parse(Rule::star_file, &content) {
            Ok(_) => {
                let elapsed = start_time.elapsed();
                parse_times.push(elapsed);
                total_duration += elapsed;
                
                if args.verbose && (i + 1) % (args.iterations / 10).max(1) == 0 {
                    println!("  Iteration {}/{}: {:.3}ms", 
                        i + 1, args.iterations, elapsed.as_secs_f64() * 1000.0);
                }
            }
            Err(e) => {
                eprintln!("Parse error on iteration {}: {}", i + 1, e);
                std::process::exit(1);
            }
        }
    }

    // Calculate statistics
    parse_times.sort();
    
    let total_ms = total_duration.as_secs_f64() * 1000.0;
    let avg_ms = total_ms / args.iterations as f64;
    let min_ms = parse_times[0].as_secs_f64() * 1000.0;
    let max_ms = parse_times[args.iterations - 1].as_secs_f64() * 1000.0;
    let median_ms = if args.iterations % 2 == 0 {
        (parse_times[args.iterations / 2 - 1].as_secs_f64() + 
         parse_times[args.iterations / 2].as_secs_f64()) * 500.0
    } else {
        parse_times[args.iterations / 2].as_secs_f64() * 1000.0
    };

    // Calculate percentiles
    let p95_idx = ((args.iterations as f64) * 0.95) as usize;
    let p99_idx = ((args.iterations as f64) * 0.99) as usize;
    let p95_ms = parse_times[p95_idx.min(args.iterations - 1)].as_secs_f64() * 1000.0;
    let p99_ms = parse_times[p99_idx.min(args.iterations - 1)].as_secs_f64() * 1000.0;

    // Calculate throughput
    let avg_throughput_bytes_per_sec = (file_size as f64) / (avg_ms / 1000.0);
    let peak_throughput_bytes_per_sec = (file_size as f64) / (min_ms / 1000.0);

    println!();
    println!("Benchmark Results");
    println!("=================");
    println!("Total time:     {:.3}ms", total_ms);
    println!("Average time:   {:.3}ms", avg_ms);
    println!("Median time:    {:.3}ms", median_ms);
    println!("Min time:       {:.3}ms", min_ms);
    println!("Max time:       {:.3}ms", max_ms);
    println!("95th percentile: {:.3}ms", p95_ms);
    println!("99th percentile: {:.3}ms", p99_ms);
    println!();
    println!("Throughput");
    println!("==========");
    println!("Average:        {}", format_throughput(avg_throughput_bytes_per_sec));
    println!("Peak (min time): {}", format_throughput(peak_throughput_bytes_per_sec));
    println!();
    println!("Performance per byte: {:.2} ns/byte", (avg_ms * 1_000_000.0) / file_size as f64);

    // Performance classification based on baseline deviation
    let actual_per_byte = (avg_ms * 1_000_000.0) / file_size as f64;
    let performance_ratio = actual_per_byte / baseline_per_byte;
    let deviation_percent = (performance_ratio - 1.0) * 100.0;
    
    println!();
    println!("Baseline Comparison");
    println!("==================");
    println!("Expected (baseline): {:.2} ns/byte", baseline_per_byte);
    println!("Actual:              {:.2} ns/byte", actual_per_byte);
    println!("Performance ratio:   {:.2}x baseline", performance_ratio);
    println!("Deviation:           {:+.1}%", deviation_percent);
    
    println!();
    print!("Performance: ");
    match performance_ratio {
        r if r <= 1.1 => println!("🚀 Excellent (within 10% of baseline)"),
        r if r <= 1.3 => println!("✅ Good (within 30% of baseline)"),
        r if r <= 1.5 => println!("⚠️  Moderate (within 50% of baseline)"),
        r if r <= 3.0 => println!("🐌 Slow (within 200% of baseline)"),
        _ => println!("🚨 Very Slow (> 200% of baseline)"),
    }

    if args.verbose {
        println!();
        println!("Detailed Timing Distribution");
        println!("============================");
        let buckets = create_timing_histogram(&parse_times);
        for (range, count) in buckets {
            let percentage = (count as f64 / args.iterations as f64) * 100.0;
            println!("{:>12}: {:>4} ({:>5.1}%) {}", 
                range, count, percentage, "█".repeat((percentage / 2.0) as usize));
        }
    }
}

fn create_timing_histogram(times: &[Duration]) -> Vec<(String, usize)> {
    let min_ns = times[0].as_nanos();
    let max_ns = times[times.len() - 1].as_nanos();
    
    if max_ns == min_ns {
        return vec![(format!("{:.3}ms", times[0].as_secs_f64() * 1000.0), times.len())];
    }

    let bucket_count = 10usize;
    let bucket_size = (max_ns - min_ns) / bucket_count as u128;
    let mut buckets = vec![0usize; bucket_count];
    let mut ranges = Vec::new();

    for i in 0..bucket_count {
        let start_ns = min_ns + (i as u128 * bucket_size);
        let end_ns = if i == bucket_count - 1 { max_ns } else { start_ns + bucket_size };
        let start_ms = start_ns as f64 / 1_000_000.0;
        let end_ms = end_ns as f64 / 1_000_000.0;
        ranges.push(format!("{:.3}-{:.3}ms", start_ms, end_ms));
    }

    for time in times {
        let time_ns = time.as_nanos();
        let bucket_idx = if time_ns == max_ns {
            bucket_count - 1
        } else {
            (((time_ns - min_ns) / bucket_size) as usize).min(bucket_count - 1)
        };
        buckets[bucket_idx] += 1;
    }

    ranges.into_iter().zip(buckets.into_iter()).collect()
}

fn format_throughput(bytes_per_sec: f64) -> String {
    const UNITS: &[(&str, f64)] = &[
        ("GB/sec", 1_000_000_000.0),
        ("MB/sec", 1_000_000.0),
        ("KB/sec", 1_000.0),
        ("B/sec", 1.0),
    ];
    
    for &(unit, divisor) in UNITS {
        if bytes_per_sec >= divisor {
            return format!("{:.2} {}", bytes_per_sec / divisor, unit);
        }
    }
    
    // Fallback for very small values
    format!("{:.2} B/sec", bytes_per_sec)
}

fn establish_baseline() -> f64 {
    let baseline_file = "examples/comprehensive_example.star";
    let baseline_iterations = 50;
    let baseline_warmup = 10;
    
    // Check if baseline file exists
    if !Path::new(baseline_file).exists() {
        eprintln!("Warning: Baseline file '{}' not found. Using default baseline of 100 ns/byte", baseline_file);
        return 100.0;
    }
    
    // Read baseline file
    let baseline_content = match fs::read_to_string(baseline_file) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Warning: Could not read baseline file. Using default baseline of 100 ns/byte");
            return 100.0;
        }
    };
    
    let baseline_size = baseline_content.len();
    
    // Validate baseline file
    if let Err(_) = StarParser::parse(Rule::star_file, &baseline_content) {
        eprintln!("Warning: Baseline file is not valid. Using default baseline of 100 ns/byte");
        return 100.0;
    }
    
    // Warmup for baseline benchmark
    for _ in 0..baseline_warmup {
        let _ = StarParser::parse(Rule::star_file, &baseline_content);
    }
    
    // Run baseline benchmark
    let mut baseline_times = Vec::with_capacity(baseline_iterations);
    
    for _ in 0..baseline_iterations {
        let start_time = Instant::now();
        if StarParser::parse(Rule::star_file, &baseline_content).is_ok() {
            baseline_times.push(start_time.elapsed());
        }
    }
    
    if baseline_times.is_empty() {
        eprintln!("Warning: Baseline benchmark failed. Using default baseline of 100 ns/byte");
        return 100.0;
    }
    
    // Calculate baseline performance per byte
    let avg_baseline_duration: Duration = baseline_times.iter().sum::<Duration>() / baseline_times.len() as u32;
    let baseline_ms = avg_baseline_duration.as_secs_f64() * 1000.0;
    let baseline_per_byte = (baseline_ms * 1_000_000.0) / baseline_size as f64;
    
    baseline_per_byte
}