use clap::Parser;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use ustar::{StarParser, Rule};
use ustar::dump_extractors::{DumpExtractor, PairExtractor};
use pest::Parser as PestParser;
use pest::iterators::Pairs;
use tabled::{Table, Tabled, settings::Style};

#[derive(Parser)]
#[command(name = "ustar-parser")]
#[command(about = "A STAR format parser with detailed parse tree visualization")]
#[command(version = "0.1.0")]
struct Args {
    /// Input file to parse (use '-' or omit for stdin)
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,
}

/// Structure to hold information about a parsed symbol for table display
#[derive(Tabled)]
struct SymbolInfo {
    #[tabled(rename = "symbol", display_with = "display_symbol")]
    symbol_number: usize,
    #[tabled(rename = "level", display_with = "display_level")]
    level: usize,
    #[tabled(rename = "rule name", display_with = "display_rule")]
    rule_name: String,
    #[tabled(rename = "offsets", display_with = "display_positions")]
    positions: String,
    #[tabled(rename = "line:col", display_with = "display_line_col")]
    line_col: String,
    #[tabled(rename = "content", display_with = "display_content")]
    content: String,
}

fn display_symbol(symbol: &usize) -> String { format!("#{}", symbol) }
fn display_level(level: &usize) -> String { format!("[{}]", level) }
fn display_rule(rule: &String) -> String { rule.clone() }
fn display_positions(pos: &String) -> String { pos.clone() }
fn display_line_col(line_col: &String) -> String { line_col.clone() }
fn display_content(content: &String) -> String { 
    // Return content with quotes - we'll process it in display_parse_tree
    format!("\"{}\"", content)
}

/// Apply ANSI coloring to whitespace only within quoted content
fn apply_content_coloring(content_part: &str) -> String {
    let mut result = String::new();
    let mut in_quotes = false;
    let mut chars = content_part.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '"' {
            in_quotes = !in_quotes;
            // Keep the quote marks in the display
            result.push(ch);
            continue;
        }
        
        if in_quotes {
            match ch {
                ' ' => result.push_str("\x1b[38;5;250m·\x1b[0m"), // Very light grey dot for space
                '\t' => result.push_str("\x1b[38;5;250m→\x1b[0m"), // Very light grey arrow for tab
                '.' if chars.peek() == Some(&'.') => {
                    // Handle ellipsis
                    chars.next(); // consume second dot
                    if chars.peek() == Some(&'.') {
                        chars.next(); // consume third dot
                        result.push_str("\x1b[38;5;250m...\x1b[0m"); // Very light grey ellipsis
                    } else {
                        result.push_str("..");
                    }
                },
                _ => {
                    // Handle escape sequences
                    if ch == '\\' && chars.peek().is_some() {
                        let next_ch = chars.next().unwrap();
                        match next_ch {
                            'n' => result.push_str("\x1b[38;5;250m\\n\x1b[0m"), // Very light grey \n
                            'r' => result.push_str("\x1b[38;5;250m\\r\x1b[0m"), // Very light grey \r
                            _ => {
                                result.push('\\');
                                result.push(next_ch);
                            }
                        }
                    } else {
                        result.push(ch);
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Helper function to convert byte position to line and column numbers (1-indexed)
fn get_line_col(input: &str, pos: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    
    for (i, ch) in input.char_indices() {
        if i >= pos {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    
    (line, col)
}

/// Collect symbol information into a vector for table display
fn collect_symbol_info(
    pairs: Pairs<Rule>,
    input: &str,
    symbol_counter: &mut usize,
    indent_level: usize,
    symbols: &mut Vec<SymbolInfo>,
) {
    let extractor = PairExtractor::new();
    
    for pair in pairs {
        *symbol_counter += 1;
        let current_symbol = *symbol_counter;
        
        let rule_name = format!("{:?}", extractor.extract_rule(&pair));
        let start_pos = extractor.extract_start(&pair);
        let end_pos = extractor.extract_end(&pair);
        let content = extractor.extract_str(&pair);
        
        // Calculate line and column positions
        let (start_line, start_col) = get_line_col(input, start_pos);
        let (end_line, end_col) = get_line_col(input, end_pos);
        
        // Check if this has children (non-terminal)
        let has_children = extractor.has_children(&pair);
        
        // Format content display: apply 30...30 rule to ALL symbols and replace newlines
        let normalized_content = content.replace('\n', "\\n").replace('\r', "\\r");
        let display_content = if normalized_content.len() > 65 {
            // For long content, show first 30 ... last 30 chars
            let first_30: String = normalized_content.chars().take(30).collect();
            let last_30: String = normalized_content.chars().rev().take(30).collect::<String>().chars().rev().collect();
            format!("{}...{}", first_30, last_30)
        } else {
            normalized_content
        };
        
        // Create symbol info
        let symbol_info = SymbolInfo {
            symbol_number: current_symbol,
            level: indent_level,
            rule_name,
            positions: format!("{}-{}", start_pos, end_pos),
            line_col: format!("{}:{}-{}:{}", start_line, start_col, end_line, end_col),
            content: display_content,
        };
        
        symbols.push(symbol_info);
        
        // Recursively collect inner pairs
        if has_children {
            collect_symbol_info(extractor.get_children(&pair), input, symbol_counter, indent_level + 1, symbols);
        }
    }
}

/// Display parse tree as a formatted table using tabled for alignment (no headers/borders)
/// Returns the number of symbols parsed
fn display_parse_tree(pairs: Pairs<Rule>, input: &str) -> usize {
    let mut symbol_counter = 0;
    let mut symbols = Vec::new();
    
    collect_symbol_info(pairs, input, &mut symbol_counter, 0, &mut symbols);
    
    let mut table = Table::new(&symbols);
    table.with(Style::empty());  // Remove all borders but keep headers
    
    // Get the table as string to add underlines manually
    let table_output = table.to_string();
    let lines: Vec<&str> = table_output.lines().collect();
    
    if !lines.is_empty() {
        // Print first line (headers) - trim leading space and remove last 2 characters
        let header_line = lines[0].trim_start();
        let trimmed_header = if header_line.len() > 2 {
            &header_line[..header_line.len()-2]
        } else {
            header_line
        };
        println!("{}", trimmed_header);
        
        // Add underlines under headers and remove last 2 characters
        let mut underline = String::new();
        for c in trimmed_header.chars() {
            if c == ' ' {
                underline.push(' ');
            } else {
                underline.push('─');
            }
        }
        println!("{}", underline);
        
        // Find the content column position from the header
        let content_column_start = header_line.rfind("content").unwrap_or(header_line.len());
        
        // Print rest of the table (data rows) - trim leading space and apply selective coloring
        for line in &lines[1..] {
            let trimmed_line = line.trim_start();
            
            if trimmed_line.len() > content_column_start {
                // Split the line at the content column position
                let (prefix, content_part) = trimmed_line.split_at(content_column_start);
                
                // Remove first and last quotes properly, handling trailing spaces
                let unquoted_content = if content_part.len() > 1 {
                    // 1. Calculate original string length
                    let original_len = content_part.len();
                    
                    // 2. Right strip all spaces
                    let trimmed = content_part.trim_end();
                    
                    // 3. Calculate trimmed string length
                    let trimmed_len = trimmed.len();
                    
                    // 4. Calculate number of trailing spaces
                    let num_spaces = original_len - trimmed_len;
                    
                    // 5. Remove first and last characters (quotes) from trimmed string
                    let content_without_quotes = if trimmed_len > 1 {
                        &trimmed[1..trimmed_len-1]
                    } else {
                        ""
                    };
                    
                    // 6. Add back the trailing spaces
                    let spaces = " ".repeat(num_spaces);
                    format!("{}{}", content_without_quotes, spaces)
                } else {
                    content_part.to_string()
                };
                let colored_content = apply_content_coloring(&unquoted_content);
                
                println!("{}{}", prefix, colored_content);
            } else {
                println!("{}", trimmed_line);
            }
        }
    }
    
    symbol_counter
}

fn main() {
    let args = Args::parse();

    // Determine input source
    let (input_text, source_info) = match &args.input {
        None => {
            // Read from stdin
            let mut buffer = String::new();
            match io::stdin().read_to_string(&mut buffer) {
                Ok(_) => (buffer, "-".to_string()),
                Err(e) => {
                    eprintln!("Error reading from stdin: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(path) if path.to_string_lossy() == "-" => {
            // Explicitly requested stdin with "-"
            let mut buffer = String::new();
            match io::stdin().read_to_string(&mut buffer) {
                Ok(_) => (buffer, "-".to_string()),
                Err(e) => {
                    eprintln!("Error reading from stdin: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(path) => {
            // Read from file
            match fs::read_to_string(path) {
                Ok(content) => (content, path.display().to_string()),
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", path.display(), e);
                    std::process::exit(1);
                }
            }
        }
    };

    // Parse the input as a complete STAR file
    match StarParser::parse(Rule::star_file, &input_text) {
        Ok(pairs) => {
            println!("source: {}", source_info);
            println!();
            let symbol_count = display_parse_tree(pairs, &input_text);
            let line_count = input_text.lines().count();
            println!();
            println!("lines: {} symbols: {}", line_count, symbol_count);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}