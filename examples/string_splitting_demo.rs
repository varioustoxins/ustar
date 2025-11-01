// String splitting pipeline demonstration
use ustar::{Rule, StarParser, string_splitter::StringSplittingPipeline};
use pest::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pest Pairs String Splitting Pipeline ===\n");
    
    // Run the built-in demo
    StringSplittingPipeline::demo()?;
    
    // Additional custom test
    println!("=== Custom Test: Parsing and Splitting ===\n");
    
    let test_string = "example_text_to_split";
    println!("Original string: '{}'", test_string);
    
    // Parse to Pest pairs
    let pairs = StarParser::parse(Rule::non_quoted_text_string, test_string)?;
    println!("Parsed as: {:?}", Rule::non_quoted_text_string);
    
    // Transform through pipeline  
    let halfstring_contents = StringSplittingPipeline::transform_strings_to_halfstring_content(pairs)?;
    
    println!("Pipeline produced {} halfstring contents:", halfstring_contents.len());
    for (i, content) in halfstring_contents.iter().enumerate() {
        println!("  Halfstring {}: '{}'", i+1, content);
        
        // Parse back as halfstring pairs
        match StringSplittingPipeline::parse_halfstring_content(content) {
            Ok(pairs) => {
                for pair in pairs {
                    println!("    → Parsed as: '{}' (rule: {:?})", pair.as_str(), pair.as_rule());
                }
            }
            Err(e) => {
                println!("    → Parse error: {}", e);
            }
        }
    }
    
    Ok(())
}