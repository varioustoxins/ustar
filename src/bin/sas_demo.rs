use ustar::sas_interface::{SASContentHandler};
use ustar::sas_walker::StarWalker;
use ustar::parse_default;
use std::fs;

struct DemoHandler {
    depth: usize,
}

impl SASContentHandler for DemoHandler {
    fn start_data(&mut self, line: usize, name: &str) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}<start data> [{}] {}", indent, line, name);
        self.depth += 1;
        false
    }
    fn end_data(&mut self, line: usize, name: &str) -> bool {
        if self.depth > 0 { self.depth -= 1; }
        let indent = "    ".repeat(self.depth);
        println!("{}<end data> [{}] {}", indent, line, name);
        false
    }
    fn start_saveframe(&mut self, line: usize, name: &str) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}<start saveframe> [{}] {}", indent, line, name);
        self.depth += 1;
        false
    }
    fn end_saveframe(&mut self, line: usize, name: &str) -> bool {
        if self.depth > 0 { self.depth -= 1; }
        let indent = "    ".repeat(self.depth);
        println!("{}<end saveframe> [{}] {}", indent, line, name);
        false
    }
    fn start_loop(&mut self, line: usize) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}<start_loop> [{}]", indent, line);
        self.depth += 1;
        false
    }
    fn end_loop(&mut self, line: usize) -> bool {
        if self.depth > 0 { self.depth -= 1; }
        let indent = "    ".repeat(self.depth);
        println!("{}<end_loop> [{}]", indent, line);
        false
    }
    fn comment(&mut self, line: usize, text: &str) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}# [{}] {}", indent, line, text);
        false
    }
    fn data(&mut self, tag: &str, tagline: usize, value: &str, valline: usize, delimiter: &str, inloop: bool) -> bool {
        let indent = "    ".repeat(self.depth);
        let tag_prefix = format!("{}<data> ", indent);
        let value_indent = " ".repeat(tag_prefix.len());

        match delimiter {
            "\n" => {
                // Print line numbers right after <data>, then tag name
                println!("{}<data> [t:{},v:{}] {} delimiter: {:?} inloop: {} value:", indent, tagline, valline, tag, delimiter, inloop);
                // Print each line of the value, indented to the tag_prefix
                for line in value.lines() {
                    println!("{}{}", value_indent, line);
                }
            }
            _ => {
                // Print line numbers right after <data>, then tag name
                println!("{}<data> [t:{},v:{}] {} delimiter: {} inloop: {} value [multiline]: {}", indent, tagline, valline, tag, delimiter, inloop, value);
            }
        }
        false
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "examples/comprehensive_example.star"
    };
    let input = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Failed to read example file: {}", filename));
    let tree = parse_default(&input).expect("Failed to parse");
    let mut handler = DemoHandler { depth: 0 };
    let mut walker = StarWalker::from_input(&mut handler, &input);
    walker.walk_star_tree_buffered(&tree);
}
