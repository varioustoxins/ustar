use clap::Parser as ClapParser;
use std::fs;
use std::path::PathBuf;
use ustar::{StarParser, Rule, Parser};

#[derive(ClapParser, Debug)]
#[command(name = "ustar-parse-debugger")]
#[command(about = "Debug STAR file parsing by finding the last parseable position")]
struct Args {
    /// Input file to debug
    #[arg(value_name = "FILE")]
    input: PathBuf,
    
    /// Show full parse tree for the successful portion
    #[arg(short, long)]
    full_tree: bool,
    
    /// Show visible whitespace characters (spaces, tabs, CR, LF)
    #[arg(short, long)]
    whitespace: bool,
}

fn main() {
    let args = Args::parse();
    
    // Read the input file
    let content = match fs::read_to_string(&args.input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file {:?}: {}", args.input, e);
            std::process::exit(1);
        }
    };
    
    println!("=== STAR File Parse Debugger ===\n");
    println!("File: {:?}", args.input);
    println!("Size: {} bytes\n", content.len());
    
    // First, try to parse the entire file
    println!("Attempting to parse entire file...");
    match StarParser::parse(Rule::star_file, &content) {
        Ok(_) => {
            println!("✓ File parses successfully!");
            std::process::exit(0);
        }
        Err(e) => {
            println!("✗ Parse failed\n");
            
            // Extract error position
            let error_pos = match e.location {
                pest::error::InputLocation::Pos(pos) => pos,
                pest::error::InputLocation::Span((start, _)) => start,
            };
            
            // Extract line and column
            let (error_line, error_col) = match e.line_col {
                pest::error::LineColLocation::Pos((line, col)) => (line, col),
                pest::error::LineColLocation::Span((line, col), _) => (line, col),
            };
            
            println!("=== Parse Error Details ===");
            println!("Position: byte {}", error_pos);
            println!("Location: line {}, column {}", error_line, error_col);
            
            // Show what was expected
            match &e.variant {
                pest::error::ErrorVariant::ParsingError { positives, negatives } => {
                    if !positives.is_empty() {
                        println!("Expected: {:?}", positives);
                    }
                    if !negatives.is_empty() {
                        println!("Not expected: {:?}", negatives);
                    }
                }
                _ => {
                    println!("Error: {:?}", e.variant);
                }
            }
            
            // Show context around the error
            show_context(&content, error_line, error_col, &e, !args.whitespace);
            
            println!("\n=== Attempting to find last parseable position ===\n");
            
            // Now try to find the last good parse position by trimming back token by token
            find_last_good_parse(&content, error_pos, error_line, error_col, args.full_tree, &e, !args.whitespace);
        }
    }
}

fn find_last_good_parse(content: &str, error_pos: usize, error_line: usize, error_col: usize, show_full_tree: bool, error: &pest::error::Error<Rule>, no_visible_whitespace: bool) {
    // Start from the error position and work backwards
    let mut current_pos = error_pos.min(content.len());
    
    // Show context around the error
    println!("Context around error:");
    show_context(content, error_line, error_col, error, no_visible_whitespace);
    println!();
    
    // Try to find whitespace/newline boundaries working backwards
    let mut attempts = 0;
    let max_attempts = 1000; // Limit attempts to avoid infinite loops
    
    while current_pos > 0 && attempts < max_attempts {
        attempts += 1;
        
        // Find the previous whitespace boundary (space, tab, or newline)
        let mut found_boundary = false;
        while current_pos > 0 {
            current_pos -= 1;
            let ch = content.as_bytes()[current_pos] as char;
            if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                found_boundary = true;
                break;
            }
        }
        
        if !found_boundary {
            break;
        }
        
        // Skip any additional whitespace
        while current_pos > 0 {
            let ch = content.as_bytes()[current_pos.saturating_sub(1)] as char;
            if ch != ' ' && ch != '\t' && ch != '\n' && ch != '\r' {
                break;
            }
            current_pos -= 1;
        }
        
        if current_pos == 0 {
            break;
        }
        
        // Try to parse up to this position
        let truncated = &content[..current_pos];
        
        match StarParser::parse(Rule::star_file, truncated) {
            Ok(pairs) => {
                // Found a successful parse!
                println!("✓ Found successful parse at byte position: {}", current_pos);
                
                // Calculate line and column for this position
                let lines_before: Vec<&str> = truncated.lines().collect();
                let parse_line = lines_before.len();
                let parse_col = if let Some(last_line) = lines_before.last() {
                    last_line.len() + 1
                } else {
                    1
                };
                
                println!("  Parsed up to line: {}, column: {}", parse_line, parse_col);
                println!("  Successfully parsed: {} bytes", current_pos);
                println!("  Remaining unparsed: {} bytes\n", content.len() - current_pos);
                
                // Show the parse tree
                if show_full_tree {
                    println!("=== Full Parse Tree ===\n");
                    for pair in pairs {
                        println!("{:#?}", pair);
                    }
                } else {
                    println!("=== Parse Tree Summary ===\n");
                    for pair in pairs {
                        print_tree_summary(&pair, 0, content);
                    }
                }
                
                println!("\n=== Last 20 Lines of Successfully Parsed Content ===\n");
                let parsed_lines: Vec<&str> = truncated.lines().collect();
                let start_line = if parsed_lines.len() > 20 { parsed_lines.len() - 20 } else { 0 };
                
                for (i, line) in parsed_lines[start_line..].iter().enumerate() {
                    let line_num = start_line + i + 1;
                    println!("{:4}: {}", line_num, line);
                }
                
                println!("\n=== Unparsed Content ===\n");
                let remaining = &content[current_pos..];
                let remaining_lines: Vec<&str> = split_lines_with_endings(remaining);
                
                if remaining_lines.is_empty() {
                    println!("(No remaining content - file ends cleanly)");
                } else {
                    // Calculate how many lines from unparsed start to error
                    let lines_to_error = error_line - parse_line;
                    let lines_after_error = 20;
                    let total_lines_to_show = lines_to_error + lines_after_error;
                    
                    // Get the expected rules for display
                    let expected_msg = match &error.variant {
                        pest::error::ErrorVariant::ParsingError { positives, negatives } => {
                            let mut parts = Vec::new();
                            if !positives.is_empty() {
                                parts.push(format!("expected {:?}", positives));
                            }
                            if !negatives.is_empty() {
                                parts.push(format!("not expected {:?}", negatives));
                            }
                            if !parts.is_empty() {
                                format!(" ({})", parts.join(", "))
                            } else {
                                String::new()
                            }
                        }
                        _ => String::new(),
                    };
                    
                    println!("Showing lines from unparsed start to error (line {}) + 20 more lines:\n", error_line);
                    
                    for (i, line) in remaining_lines.iter().take(total_lines_to_show).enumerate() {
                        let line_num = parse_line + i;
                        
                        if line_num == error_line {
                            let visible_line = make_whitespace_visible(line, no_visible_whitespace);
                            println!(">>> {:4}: {}", line_num, visible_line);
                            // Show error column indicator
                            if error_col > 0 {
                                println!("    {}^--- ERROR HERE (column {}){}", 
                                    " ".repeat(error_col + 4), 
                                    error_col,
                                    expected_msg);
                            }
                        } else {
                            let visible_line = make_whitespace_visible(line, no_visible_whitespace);
                            println!("    {:4}: {}", line_num, visible_line);
                        }
                    }
                    
                    let shown = std::cmp::min(total_lines_to_show, remaining_lines.len());
                    if remaining_lines.len() > shown {
                        println!("\n... ({} more lines not shown)", remaining_lines.len() - shown);
                    }
                }
                
                println!("\n=== Analysis ===\n");
                println!("The parser successfully parsed up to byte position {}.", current_pos);
                println!("The error occurs in the content starting at byte position {}.", current_pos);
                println!("This suggests the parsing issue begins at line {}, column {}.", parse_line + 1, 1);
                
                return;
            }
            Err(_) => {
                // Still failing, continue trimming
                if attempts % 50 == 0 {
                    println!("  Tried {} positions... current position: {}", attempts, current_pos);
                }
            }
        }
    }
    
    println!("Could not find a successful parse point after {} attempts.", attempts);
    println!("The file may have fundamental syntax errors near the beginning.");
}

fn show_context(content: &str, error_line: usize, error_col: usize, error: &pest::error::Error<Rule>, no_visible_whitespace: bool) {
    // Split lines while preserving line endings
    let lines: Vec<&str> = split_lines_with_endings(content);
    let error_idx = error_line.saturating_sub(1);
    
    let start = error_idx.saturating_sub(3);
    let end = (error_idx + 3).min(lines.len());
    
    println!("\n=== Error Context ===\n");
    
    // Get the expected rules for display
    let expected_msg = match &error.variant {
        pest::error::ErrorVariant::ParsingError { positives, negatives } => {
            let mut parts = Vec::new();
            if !positives.is_empty() {
                parts.push(format!("expected {:?}", positives));
            }
            if !negatives.is_empty() {
                parts.push(format!("not expected {:?}", negatives));
            }
            if !parts.is_empty() {
                format!(" ({})", parts.join(", "))
            } else {
                String::new()
            }
        }
        _ => String::new(),
    };
    
    for i in start..end {
        let line_num = i + 1;
        let marker = if line_num == error_line { ">>>" } else { "   " };
        
        if i < lines.len() {
            let visible_line = make_whitespace_visible(lines[i], no_visible_whitespace);
            println!("{} {:4}: {}", marker, line_num, visible_line);
            
            if line_num == error_line {
                println!("    {}^--- ERROR HERE (column {}){}", 
                    " ".repeat(error_col + 4), 
                    error_col,
                    expected_msg);
            }
        }
    }
}

fn split_lines_with_endings(content: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let bytes = content.as_bytes();
    
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            // Include the \n in the line
            lines.push(&content[start..=i]);
            start = i + 1;
        } else if bytes[i] == b'\r' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                // Include both \r\n in the line
                lines.push(&content[start..=i + 1]);
                i += 1; // Skip the \n
                start = i + 1;
            } else {
                // Just \r
                lines.push(&content[start..=i]);
                start = i + 1;
            }
        }
        i += 1;
    }
    
    // Add any remaining content as the last line
    if start < content.len() {
        lines.push(&content[start..]);
    }
    
    lines
}

fn make_whitespace_visible(line: &str, no_visible_whitespace: bool) -> String {
    if no_visible_whitespace {
        // Strip line endings and return the line
        return line.trim_end_matches(&['\r', '\n'][..]).to_string();
    }
    
    let mut result = String::new();
    for ch in line.chars() {
        match ch {
            ' ' => result.push_str("\x1b[90m·\x1b[0m"),  // Grey middle dot for space
            '\t' => result.push_str("\x1b[90m→\x1b[0m"), // Grey arrow for tab
            '\r' => result.push_str("\x1b[90m␍\x1b[0m"), // Grey CR symbol
            '\n' => result.push_str("\x1b[90m␊\x1b[0m"), // Grey LF symbol
            _ => result.push(ch),
        }
    }
    result
}

fn calculate_max_depth(pair: &pest::iterators::Pair<Rule>, current_depth: usize) -> usize {
    let mut max = current_depth;
    for inner in pair.clone().into_inner() {
        let child_max = calculate_max_depth(&inner, current_depth + 1);
        max = max.max(child_max);
    }
    max
}

fn byte_to_line(content: &str, byte_pos: usize) -> usize {
    content[..byte_pos.min(content.len())]
        .chars()
        .filter(|&c| c == '\n')
        .count() + 1
}

fn find_max_line(pair: &pest::iterators::Pair<Rule>, content: &str) -> usize {
    let span = pair.as_span();
    let mut max_line = byte_to_line(content, span.end());
    
    for inner in pair.clone().into_inner() {
        let child_max = find_max_line(&inner, content);
        max_line = max_line.max(child_max);
    }
    
    max_line
}

fn print_tree_summary(pair: &pest::iterators::Pair<Rule>, depth: usize, content: &str) {
    // First pass: calculate max depth
    let max_depth = calculate_max_depth(pair, depth);
    
    // Second pass: calculate max line number
    let max_line = find_max_line(pair, content);
    
    // Third pass: build tree structure strings and calculate max width
    let mut tree_lines = Vec::new();
    let mut text_lines = Vec::new();
    build_tree_lines(pair, depth, max_depth, content, max_line, &mut tree_lines, &mut text_lines);
    
    // Find the maximum width of tree structure
    let max_tree_width = tree_lines.iter().map(|s| s.len()).max().unwrap_or(0);
    
    // Fourth pass: print with aligned text
    for (tree_line, text_line) in tree_lines.iter().zip(text_lines.iter()) {
        println!("{:<width$}  \"{}\"", tree_line, text_line, width = max_tree_width);
    }
}

fn build_tree_lines(
    pair: &pest::iterators::Pair<Rule>,
    depth: usize,
    max_depth: usize,
    content: &str,
    max_line: usize,
    tree_lines: &mut Vec<String>,
    text_lines: &mut Vec<String>
) {
    let rule = pair.as_rule();
    let span = pair.as_span();
    
    // Calculate line numbers for start and end positions
    let start_line = byte_to_line(content, span.start());
    let end_line = byte_to_line(content, span.end());
    
    // Calculate the width needed for line numbers and depth
    let line_width = max_line.to_string().len();
    let depth_width = max_depth.to_string().len();
    
    // Build the tree structure line
    let indent = "  ".repeat(depth);
    let tree_line = format!("{:>line_width$} | {:>depth_width$} | {}{:?} ({}-{}) lines {}-{}", 
        start_line, depth, indent, rule, span.start(), span.end(), start_line, end_line,
        line_width = line_width,
        depth_width = depth_width);
    
    // Get the token text and format it
    let token_text = span.as_str();
    let formatted_text = if token_text.len() > 60 {
        let first_30: String = token_text.chars().take(30).collect();
        let last_30: String = token_text.chars().rev().take(30).collect::<Vec<_>>().into_iter().rev().collect();
        format!("{}...{}", first_30, last_30)
    } else {
        token_text.to_string()
    };
    
    // Escape special characters for display
    let display_text = formatted_text
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    
    tree_lines.push(tree_line);
    text_lines.push(display_text);
    
    for inner in pair.clone().into_inner() {
        build_tree_lines(&inner, depth + 1, max_depth, content, max_line, tree_lines, text_lines);
    }
}
