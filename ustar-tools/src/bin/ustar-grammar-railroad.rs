use clap::Parser;
use pest_railroad::generate_diagram;
use std::fs;
use std::path::PathBuf;

/// Generate railroad diagrams for USTAR grammar files
#[derive(Parser)]
#[command(name = "ustar-grammar-railroad")]
#[command(about = "Generate railroad diagrams for USTAR grammar files")]
struct Cli {
    /// Grammar file to process
    grammar_file: PathBuf,

    /// Output SVG file (defaults to input filename with .svg extension)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Determine output file path
    let output_file = if let Some(output) = cli.output {
        output
    } else {
        // Default: replace input extension with .svg
        cli.grammar_file.with_extension("svg")
    };

    if cli.verbose {
        println!("Input grammar file: {}", cli.grammar_file.display());
        println!("Output SVG file: {}", output_file.display());
    }

    // Read grammar file
    let grammar_content = fs::read_to_string(&cli.grammar_file).map_err(|e| {
        format!(
            "Failed to read grammar file {}: {}",
            cli.grammar_file.display(),
            e
        )
    })?;

    if cli.verbose {
        println!("Grammar file loaded, {} bytes", grammar_content.len());
    }

    // Generate railroad diagram
    match generate_diagram(&grammar_content) {
        Ok((diagram, warnings)) => {
            // Print any warnings
            if !warnings.is_empty() {
                if cli.verbose {
                    println!("Warnings:");
                    for warning in &warnings {
                        println!("  {}", warning);
                    }
                } else {
                    println!(
                        "Note: {} warnings about unsupported grammar features",
                        warnings.len()
                    );
                }
            }

            // Convert diagram to SVG
            let svg = format!("{}", diagram);
            fs::write(&output_file, &svg)?;

            println!("Generated railroad diagram: {}", output_file.display());
        }
        Err(e) => {
            eprintln!("Failed to generate railroad diagram: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
