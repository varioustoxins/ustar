use ustar::sas_buffered::{BufferedContentHandler};
use ustar::sas_buffered_walker::StarWalker;
use ustar::parse_default;
use std::fs;

struct DemoHandler {
    depth: usize,
}

impl BufferedContentHandler for DemoHandler {
    fn start_data(&mut self, _line: usize, name: &str) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}<start data> {}", indent, name);
        self.depth += 1;
        false
    }
    fn end_data(&mut self, _line: usize, name: &str) -> bool {
        if self.depth > 0 { self.depth -= 1; }
        let indent = "    ".repeat(self.depth);
        println!("{}<end data> {}", indent, name);
        false
    }
    fn start_saveframe(&mut self, _line: usize, name: &str) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}<start saveframe> -{}-", indent, name);
        self.depth += 1;
        false
    }
    fn end_saveframe(&mut self, _line: usize, name: &str) -> bool {
        if self.depth > 0 { self.depth -= 1; }
        let indent = "    ".repeat(self.depth);
        println!("{}<end saveframe> -{}-", indent, name);
        false
    }
    fn start_loop(&mut self, _line: usize) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}<start_loop>", indent);
        self.depth += 1;
        false
    }
    fn end_loop(&mut self, _line: usize) -> bool {
        if self.depth > 0 { self.depth -= 1; }
        let indent = "    ".repeat(self.depth);
        println!("{}<end_loop>", indent);
        false
    }
    fn comment(&mut self, _line: usize, text: &str) -> bool {
        let indent = "    ".repeat(self.depth);
        println!("{}# {}", indent, text);
        false
    }
    fn data(&mut self, tag: &str, _tagline: usize, value: &str, _valline: usize, delimiter: &str, inloop: bool) -> bool {
        let indent = "    ".repeat(self.depth);
        let tag_prefix = format!("{}<data> tag: {} ", indent, tag);
        let value_indent = " ".repeat(tag_prefix.len());

        match delimiter {
            ";" => {
                // Print tag line
                println!("{}<data> tag: {} delimiter: {:?} inloop: {}", indent, tag, delimiter, inloop);
                // Print each line of the value, indented to the tag_prefix
                for line in value.lines() {
                    println!("{}{}", value_indent, line);
                }
            }
            _ => {
                // Print tag and value on the same line
                println!("{}<data> tag: {} delimiter: {} inloop: {} value: {}", indent, tag, delimiter, inloop, value);
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
    let mut walker = StarWalker::new(&mut handler);
    walker.walk_star_tree_buffered(&tree);
}
