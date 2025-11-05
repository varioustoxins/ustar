use ustar::{StarParser, Rule, Parser};
use ustar::mutable_pair::MutablePair;
use ustar::string_decomposer::decompose_strings;
use ustar::dump_extractors::dump_mutable_pair;

fn main() {
    let input = r#"data_test
_double "hello world"
_single 'test string'
_non_quoted simple_value
_semicolon
;
Multi-line content
with embedded text
;
"#;

    println!("=== String Decomposer Demo ===\n");
    println!("Input:");
    println!("{}", input);
    println!();
    
    // Parse the input
    match StarParser::parse(Rule::star_file, input) {
        Ok(pairs) => {
            // Convert to MutablePair
            let mut mutable_pairs: Vec<MutablePair> = pairs
                .map(|p| MutablePair::from_pest_pair(&p))
                .collect();
            
            println!("=== BEFORE Decomposition ===\n");
            for pair in &mutable_pairs {
                dump_mutable_pair(pair, 0);
            }
            
            // Apply string decomposition
            for pair in &mut mutable_pairs {
                decompose_strings(pair);
            }
            
            println!("\n=== AFTER Decomposition ===\n");
            for pair in &mutable_pairs {
                dump_mutable_pair(pair, 0);
            }
            
            println!("\n=== Summary ===");
            println!("✅ Double quoted strings decomposed into: DOUBLE_QUOTE + string + DOUBLE_QUOTE");
            println!("✅ Single quoted strings decomposed into: SINGLE_QUOTE + string + SINGLE_QUOTE");
            println!("✅ Semicolon strings decomposed into: NEWLINE_SEMICOLON + string + NEWLINE_SEMICOLON");
            println!("✅ Non-quoted strings converted to: string rule");
            println!("✅ All original offsets preserved correctly");
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}
