use clap::Parser as ClapParser;

#[derive(Debug)]
struct ParseResult {
    last_position: usize,
    #[allow(dead_code)]
    total_bytes: usize,
    parsed_bytes: usize,
    remaining_bytes: usize,
    #[allow(dead_code)]
    total_lines: usize,
    parsed_lines: usize,
    remaining_lines: usize,
    truncated_content: String,
    unparsed_content: String,
}
use pest::Parser;
use std::fs;
use std::path::PathBuf;
use ustar::parsers::ascii::{AsciiParser, Rule};
use ustar::{default_config, parse, ConfigKey, ConfigValue, ErrorFormatMode};

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

    println!("uSTAR Parse Debugger\n");

    let mut config = default_config();
    // Configure for fancy error display with 10 lines of context
    config.insert(
        ConfigKey::ErrorFormat,
        ConfigValue::ErrorFormat(ErrorFormatMode::Fancy),
    );
    config.insert(ConfigKey::ContextLines, ConfigValue::Usize(10));

    let stored_result = match parse(&content, &config) {
        Ok(_) => {
            println!("✓ File parses successfully!");
            std::process::exit(0);
        }
        Err(e) => {
            let result = {
                // Now use direct pest parsing for detailed debugging
                match AsciiParser::parse(Rule::star_file, &content) {
                    Ok(_) => {
                        println!(
                            "Unexpected: direct pest parser succeeded where new system failed"
                        );
                        std::process::exit(0);
                    }
                    Err(pest_error) => {
                        // Extract error position for debugging analysis
                        let error_pos = match pest_error.location {
                            pest::error::InputLocation::Pos(pos) => pos,
                            pest::error::InputLocation::Span((start, _)) => start,
                        };

                        // Extract line and column
                        let (error_line, _error_col) = match pest_error.line_col {
                            pest::error::LineColLocation::Pos((line, col)) => (line, col),
                            pest::error::LineColLocation::Span((line, col), _) => (line, col),
                        };

                        println!("\n=== Attempting to find last parseable position ===\n");

                        // Now try to find the last good parse position by trimming back token by token
                        let parse_result = find_last_good_parse(&content, error_pos);

                        if let Some(result) = parse_result {
                            display_parse_debug_info(
                                &content,
                                error_line,
                                &result,
                                args.full_tree,
                                !args.whitespace,
                            );
                            Some(result)
                        } else {
                            println!("     Could not find a successful parse point.");
                            println!("     The file may have fundamental syntax errors near the beginning.");
                            None
                        }
                    }
                }
            };

            println!("\n=== Error Message ===\n");

            println!("{}\n", e.format_error(ErrorFormatMode::Fancy, 10));

            result
        }
    };
    println!("✗ Parse failed\n");
    if let Some(result) = stored_result {
        println!("Last parsed position: {} bytes", result.last_position);
        println!("Bytes parsed: {} bytes", result.parsed_bytes);
        println!("Bytes remaining: {} bytes", result.remaining_bytes);
        println!("Lines parsed: {} lines", result.parsed_lines);
        println!("Lines remaining: {} lines\n", result.remaining_lines);
    } else {
        println!("No successful parse position could be determined.\n");
    }
    println!("File: {}", args.input.display());
    println!("Size: {} bytes", content.len());
}

fn find_last_good_parse(content: &str, error_pos: usize) -> Option<ParseResult> {
    // Start from the error position and work backwards
    let mut current_pos = error_pos.min(content.len());

    // Try to find whitespace/newline boundaries working backwards
    let mut attempts = 0;
    let max_attempts = 10000; // Limit attempts to avoid infinite loops

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

        match AsciiParser::parse(Rule::star_file, truncated) {
            Ok(_) => {
                // Found a successful parse! Calculate and return parse statistics
                let lines_before: Vec<&str> = truncated.lines().collect();
                let parse_line = lines_before.len();
                let total_bytes = content.len();
                let parsed_bytes = current_pos;
                let remaining_bytes = total_bytes - parsed_bytes;
                let total_lines = content.lines().count();
                let parsed_lines = parse_line;
                let remaining_lines = total_lines - parsed_lines;
                let unparsed_content = content[current_pos..].to_string();

                return Some(ParseResult {
                    last_position: current_pos,
                    total_bytes,
                    parsed_bytes,
                    remaining_bytes,
                    total_lines,
                    parsed_lines,
                    remaining_lines,
                    truncated_content: truncated.to_string(),
                    unparsed_content,
                });
            }
            Err(_) => {
                // Still failing, continue trimming
                continue;
            }
        }
    }

    None
}

fn display_parse_debug_info(
    content: &str,
    error_line: usize,
    result: &ParseResult,
    show_full_tree: bool,
    no_visible_whitespace: bool,
) {
    println!(
        "   ✓ Found successful parse at byte position: {}\n",
        result.last_position
    );

    // Show the parse tree by re-parsing the truncated content
    match AsciiParser::parse(Rule::star_file, &result.truncated_content) {
        Ok(pairs) => {
            if show_full_tree {
                println!("=== Full Parse Tree ===\n");
                for pair in pairs {
                    println!("{:#?}", pair);
                }
            } else {
                println!("=== Successful Parse Tree ===\n");
                for pair in pairs {
                    print_tree_summary(&pair, 0, content);
                }
            }
        }
        Err(_) => {
            println!("Error: Could not re-parse truncated content for tree display");
        }
    }

    println!();
    println!("=== Unparsed Content ===\n");

    let remaining_lines: Vec<&str> = split_lines_with_endings(&result.unparsed_content);

    if remaining_lines.is_empty() {
        println!("    (No remaining content - file ends cleanly)");
    } else {
        // Calculate how many lines from unparsed start to error
        let lines_to_error = error_line - result.parsed_lines;
        let total_lines_to_show = lines_to_error;

        println!(
            "    Showing unparsed content ({} lines):\n",
            total_lines_to_show
        );

        for (i, line) in remaining_lines.iter().take(total_lines_to_show).enumerate() {
            let line_num = result.parsed_lines + i + 1;
            let visible_line = make_whitespace_visible(line, no_visible_whitespace);
            println!("  {:4}: {}", line_num, visible_line);
        }
        println!();
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
            ' ' => result.push_str("\x1b[90m·\x1b[0m"), // Grey middle dot for space
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
        .count()
        + 1
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
    print_tree_summary_with_indent(pair, depth, content, 4);
}

fn print_tree_summary_with_indent(
    pair: &pest::iterators::Pair<Rule>,
    depth: usize,
    content: &str,
    indent_spaces: usize,
) {
    // First pass: calculate max depth
    let max_depth = calculate_max_depth(pair, depth);

    // Second pass: calculate max line number
    let max_line = find_max_line(pair, content);

    // Third pass: build tree structure strings and calculate max width
    let mut tree_lines = Vec::new();
    let mut text_lines = Vec::new();
    build_tree_lines(
        pair,
        depth,
        max_depth,
        content,
        max_line,
        &mut tree_lines,
        &mut text_lines,
    );

    // Find the maximum width of tree structure
    let max_tree_width = tree_lines.iter().map(|s| s.len()).max().unwrap_or(0);

    // Fourth pass: print with aligned text and custom indentation
    let indent = " ".repeat(indent_spaces);
    for (tree_line, text_line) in tree_lines.iter().zip(text_lines.iter()) {
        println!(
            "{}{:<width$}  \"{}\"",
            indent,
            tree_line,
            text_line,
            width = max_tree_width
        );
    }
}

fn build_tree_lines(
    pair: &pest::iterators::Pair<Rule>,
    depth: usize,
    max_depth: usize,
    content: &str,
    max_line: usize,
    tree_lines: &mut Vec<String>,
    text_lines: &mut Vec<String>,
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
    let tree_line = format!(
        "{:>line_width$} | {:>depth_width$} | {}{:?} ({}-{}) lines {}-{}",
        start_line,
        depth,
        indent,
        rule,
        span.start(),
        span.end(),
        start_line,
        end_line,
        line_width = line_width,
        depth_width = depth_width
    );

    // Get the token text and format it
    let token_text = span.as_str();
    let formatted_text = if token_text.len() > 60 {
        let first_30: String = token_text.chars().take(30).collect();
        let last_30: String = token_text
            .chars()
            .rev()
            .take(30)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
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
        build_tree_lines(
            &inner,
            depth + 1,
            max_depth,
            content,
            max_line,
            tree_lines,
            text_lines,
        );
    }
}
