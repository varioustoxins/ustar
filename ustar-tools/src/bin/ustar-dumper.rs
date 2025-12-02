use clap::Parser;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use tabled::{settings::Style, Table, Tabled};
use text_trees::{FormatCharacters, StringTreeNode, TreeFormatting};
use ustar_parser::mutable_pair::MutablePair;
use ustar_parser::{default_config, get_context_lines, get_error_format, parse};
use ustar_tools::dump_extractors::{DumpExtractor, MutablePairExtractor};

#[derive(Parser)]
#[command(name = "ustar-parser")]
#[command(about = "A STAR format parser with detailed parse tree visualization")]
#[command(version = "0.1.0")]
struct Args {
    /// Input file to parse (use '-' or omit for stdin)
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,
    /// Display rule names as a tree with ASCII connecting lines
    #[arg(long, action = clap::ArgAction::SetTrue)]
    tree: bool,
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
    #[tabled(rename = "content")]
    content: String,
}

fn display_symbol(symbol: &usize) -> String {
    format!("#{}", symbol)
}
fn display_level(level: &usize) -> String {
    format!("[{}]", level)
}
fn display_rule(rule: &String) -> String {
    rule.clone()
}

/// Generate tree visualization from MutablePair using text_trees
/// Returns a vector of strings representing the tree structure
fn generate_tree_lines(root: &MutablePair) -> Vec<String> {
    fn build_tree_node(pair: &MutablePair) -> StringTreeNode {
        let extractor = MutablePairExtractor::new();
        let rule_name = extractor.extract_rule_name(pair);

        if extractor.has_children(pair) {
            let children: Vec<StringTreeNode> = extractor
                .get_children(pair)
                .map(|child| build_tree_node(&child))
                .collect();

            StringTreeNode::with_child_nodes(rule_name, children.into_iter())
        } else {
            StringTreeNode::new(rule_name)
        }
    }

    let tree_node = build_tree_node(root);

    // Use box characters, top-down orientation, anchor below with your specified settings
    let formatting = TreeFormatting::dir_tree(FormatCharacters::box_chars());

    let tree_string = tree_node
        .to_string_with_format(&formatting)
        .unwrap_or_else(|_| tree_node.to_string());
    tree_string.lines().map(|s| s.to_string()).collect()
}
fn display_positions(pos: &String) -> String {
    pos.clone()
}
fn display_line_col(line_col: &String) -> String {
    // For now, return as-is. We'll need to modify the table generation to pre-compute widths
    line_col.clone()
}

/// Apply ANSI coloring to whitespace and special characters
fn apply_content_coloring(content_part: &str) -> String {
    let mut result = String::new();
    let mut chars = content_part.chars().peekable();

    while let Some(ch) = chars.next() {
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
            }
            _ => {
                // Handle escape sequences
                if ch == '\\' && chars.peek().is_some() {
                    let next_ch = chars.next().unwrap();
                    match next_ch {
                        'n' => result.push_str("\x1b[38;5;250m␊\x1b[0m"), // Very light grey newline symbol
                        'r' => result.push_str("\x1b[38;5;250m␍\x1b[0m"), // Very light grey carriage return symbol
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

/// Collect symbol information from MutablePair into a vector for table display
fn collect_symbol_info_from_mutable(
    pair: &MutablePair,
    input: &str,
    symbol_counter: &mut usize,
    indent_level: usize,
    symbols: &mut Vec<SymbolInfo>,
    tree_lines: Option<&[String]>,
) {
    let extractor = MutablePairExtractor::new();

    *symbol_counter += 1;
    let current_symbol = *symbol_counter;

    // Get tree line if available, otherwise use basic rule name
    let rule_name = if let Some(lines) = tree_lines {
        // Use the tree line for this symbol index
        if let Some(line) = lines.get(current_symbol - 1) {
            line.clone()
        } else {
            extractor.extract_rule_name(pair)
        }
    } else {
        extractor.extract_rule_name(pair)
    };

    let start_pos = extractor.extract_start(pair);
    let end_pos = extractor.extract_end(pair);
    let content = extractor.extract_str(pair);

    // Calculate line and column positions
    let (start_line, start_col) = get_line_col(input, start_pos);
    let (end_line, end_col) = get_line_col(input, end_pos);

    // Check if this has children (non-terminal)
    let has_children = extractor.has_children(pair);

    // Format content display: apply 30...30 rule to ALL symbols and replace newlines
    let normalized_content = content.replace('\n', "\\n").replace('\r', "\\r");
    let display_content = if normalized_content.len() > 65 {
        // For long content, show first 30 ... last 30 chars
        let first_30: String = normalized_content.chars().take(30).collect();
        let last_30: String = normalized_content
            .chars()
            .rev()
            .take(30)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
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

    // Recursively collect child pairs
    if has_children {
        for child in extractor.get_children(pair) {
            collect_symbol_info_from_mutable(
                &child,
                input,
                symbol_counter,
                indent_level + 1,
                symbols,
                tree_lines,
            );
        }
    }
}

/// Display parse tree as a formatted table using tabled for alignment (no headers/borders)
/// Returns the number of symbols parsed
fn display_parse_tree(mutable_pair: &MutablePair, input: &str, use_tree: bool) -> usize {
    let mut symbol_counter = 0;
    let mut symbols = Vec::new();

    let tree_lines = if use_tree {
        Some(generate_tree_lines(mutable_pair))
    } else {
        None
    };

    collect_symbol_info_from_mutable(
        mutable_pair,
        input,
        &mut symbol_counter,
        0,
        &mut symbols,
        tree_lines.as_deref(),
    );

    // Pre-compute the maximum width of the first part of line:col for alignment
    let max_first_width = symbols
        .iter()
        .map(|sym| {
            if let Some(dash_pos) = sym.line_col.find('-') {
                dash_pos
            } else {
                sym.line_col.len()
            }
        })
        .max()
        .unwrap_or(0);

    // Update line_col formatting with proper alignment
    for symbol in &mut symbols {
        if let Some(dash_pos) = symbol.line_col.find('-') {
            let first_part = &symbol.line_col[..dash_pos];
            let second_part = &symbol.line_col[dash_pos + 1..];
            symbol.line_col = format!(
                "{:<width$} - {}",
                first_part,
                second_part,
                width = max_first_width
            );
        }
    }

    let mut table = Table::new(&symbols);
    table.with(Style::empty()); // Remove all borders but keep headers

    // Get the table as string to add underlines manually
    let table_output = table.to_string();
    let lines: Vec<&str> = table_output.lines().collect();

    if !lines.is_empty() {
        // Print first line (headers) - trim leading space and remove last 2 characters
        let header_line = lines[0].trim_start();
        let trimmed_header = if header_line.len() > 2 {
            &header_line[..header_line.len() - 2]
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

        // Print rest of the table (data rows) - trim leading space and apply selective coloring
        for line in &lines[1..] {
            let trimmed_line = line.trim_start();

            // Find the content column position dynamically by looking for line:col pattern
            // The line:col column comes right before content and has format like "1:18-1:23"
            let content_column_start = {
                // Look for the last occurrence of a pattern like ":digit" which indicates line:col
                if let Some(colon_pos) = trimmed_line.rfind(':') {
                    // Find the end of this line:col field by looking for the next space after digits
                    let after_colon = &trimmed_line[colon_pos + 1..];
                    let mut end_offset = 0;

                    for ch in after_colon.chars() {
                        if ch.is_ascii_digit() || ch == '-' || ch == ':' {
                            end_offset += ch.len_utf8();
                        } else {
                            break;
                        }
                    }

                    let after_line_col = colon_pos + 1 + end_offset;

                    // Skip spaces to find where content starts
                    let remaining = &trimmed_line[after_line_col..];
                    let content_offset = remaining.chars().take_while(|c| *c == ' ').count();

                    after_line_col + content_offset
                } else {
                    // Fallback: couldn't find line:col pattern
                    trimmed_line.len()
                }
            };

            if trimmed_line.len() > content_column_start {
                // Split the line at the actual content column position
                let (prefix, content_part) = trimmed_line.split_at(content_column_start);

                // Strip trailing spaces since content is the last column
                let content_trimmed = content_part.trim_end();

                // Apply light grey coloring to special characters
                let colored_content = apply_content_coloring(content_trimmed);

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
                    eprintln!("Error reading file {}: {}", path.display(), e);
                    std::process::exit(1);
                }
            }
        }
    };

    // Parse the input using the new error formatting system
    let config = default_config();
    match parse(&input_text, &config) {
        Ok(mutable_result) => {
            println!("source: {}", source_info);
            println!();
            let symbol_count = display_parse_tree(&mutable_result, &input_text, args.tree);
            let line_count = input_text.lines().count();
            println!();
            println!("lines: {} symbols: {}", line_count, symbol_count);
        }
        Err(e) => {
            eprintln!("Syntax error in {}", source_info);

            eprintln!();

            // Then show the detailed error formatting
            let error_format = get_error_format(&config);
            let context_lines = get_context_lines(&config);
            eprintln!("{}", e.format_error(error_format, context_lines));
            std::process::exit(1);
        }
    }
}
