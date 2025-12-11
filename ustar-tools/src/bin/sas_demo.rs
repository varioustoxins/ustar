use clap::Parser;
use std::fs;
use ustar_parser::line_column_index::LineColumn;
use ustar_parser::sas_interface::SASContentHandler;
use ustar_parser::sas_walker::StarWalker;
use ustar_parser::{default_config, get_context_lines, get_error_format, parse};

struct DemoHandler {
    depth: usize,
}

impl SASContentHandler for DemoHandler {
    fn start_stream(&mut self, name: Option<&str>) -> bool {
        match name {
            Some(n) => println!("<start_stream> {}", n),
            None => println!("<start_stream>"),
        }
        false
    }

    fn end_stream(&mut self, position: LineColumn) -> bool {
        println!("<end_stream> [{}:{}]", position.line, position.column);
        false
    }

    fn start_global(&mut self, position: LineColumn) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}<start global> [{}]", indent, position.line);
        self.depth += 1;
        false
    }

    fn end_global(&mut self, position: LineColumn) -> bool {
        if self.depth > 0 {
            self.depth -= 1;
        }
        let indent = "    ".repeat(self.depth);
        println!("{}<end global> [{}]", indent, position.line);
        false
    }

    fn start_data(&mut self, position: LineColumn, name: &str) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}<start data> [{}] {}", indent, position.line, name);
        self.depth += 1;
        false
    }
    fn end_data(&mut self, position: LineColumn, name: &str) -> bool {
        if self.depth > 0 {
            self.depth -= 1;
        }
        let indent = "    ".repeat(self.depth);
        println!("{}<end data> [{}] {}", indent, position.line, name);
        false
    }
    fn start_saveframe(&mut self, position: LineColumn, name: &str) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}<start saveframe> [{}] {}", indent, position.line, name);
        self.depth += 1;
        false
    }
    fn end_saveframe(&mut self, position: LineColumn, name: &str) -> bool {
        if self.depth > 0 {
            self.depth -= 1;
        }
        let indent = "    ".repeat(self.depth);
        println!("{}<end saveframe> [{}] {}", indent, position.line, name);
        false
    }
    fn start_loop(&mut self, position: LineColumn) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}<start_loop> [{}]", indent, position.line);
        self.depth += 1;
        false
    }
    fn end_loop(&mut self, position: LineColumn) -> bool {
        if self.depth > 0 {
            self.depth -= 1;
        }
        let indent = "    ".repeat(self.depth);
        println!("{}<end_loop> [{}]", indent, position.line);
        false
    }
    fn comment(&mut self, position: LineColumn, text: &str) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}# [{}] {}", indent, position.line, text);
        false
    }
    fn data(
        &mut self,
        tag: &str,
        tag_position: LineColumn,
        value: &str,
        value_position: LineColumn,
        delimiter: &str,
        loop_level: usize,
    ) -> bool {
        let indent = "    ".repeat(self.depth);
        let tag_prefix = format!("{}<data> ", indent);
        let value_indent = " ".repeat(tag_prefix.len());

        match delimiter {
            "\n" => {
                // Print line numbers right after <data>, then tag name
                println!(
                    "{}<data> [t:{}:{},v:{}:{}] {} delimiter: {:?} loop_level: {} value:",
                    indent,
                    tag_position.line,
                    tag_position.column,
                    value_position.line,
                    value_position.column,
                    tag,
                    delimiter,
                    loop_level
                );
                // Print each line of the value, indented to the tag_prefix
                for line in value.lines() {
                    println!("{}{}", value_indent, line);
                }
            }
            _ => {
                // Print line numbers right after <data>, then tag name
                println!(
                    "{}<data> [t:{}:{},v:{}:{}] {} delimiter: {} loop_level: {} value [multiline]: {}",
                    indent, tag_position.line, tag_position.column, value_position.line, value_position.column, tag, delimiter, loop_level, value
                );
            }
        }
        false
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Demonstrate SAS (SAX-like API for STAR) event streaming", long_about = None)]
struct Cli {
    /// STAR file to parse and demonstrate SAS events
    #[arg(value_name = "FILE", help = "Input STAR file to process")]
    file: Option<String>,

    /// Show verbose output with line numbers and positions
    #[arg(short, long, help = "Enable verbose output")]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    let filename = cli
        .file
        .unwrap_or_else(|| "examples/comprehensive_example.star".to_string());

    let input = fs::read_to_string(&filename).unwrap_or_else(|_| {
        eprintln!("Error: Failed to read file: {}", filename);
        eprintln!("Please check that the file exists and is readable.");
        std::process::exit(1);
    });

    if cli.verbose {
        println!("Processing STAR file: {}", filename);
        println!("File size: {} bytes", input.len());
        println!("Starting SAS event stream...\n");
    }

    let config = default_config();
    let tree = parse(&input, &config).unwrap_or_else(|e| {
        let error_format = get_error_format(&config);
        let context_lines = get_context_lines(&config);
        eprintln!("Parse error in {}:", filename);
        eprintln!("{}", e.format_error(error_format, context_lines));
        std::process::exit(1);
    });

    let mut handler = DemoHandler { depth: 0 };
    let mut walker = StarWalker::from_input(&mut handler, &input);
    walker.walk_star_tree_buffered(&tree);

    if cli.verbose {
        println!("\nSAS event streaming completed successfully.");
    }
}
