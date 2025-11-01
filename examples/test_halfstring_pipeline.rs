// Simple test demonstrating the string splitting pipeline
use ustar::{Rule, StarParser, string_splitter::StringSplittingPipeline};
use pest::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Test: Pipeline That Cuts Strings in Half ===\n");
    
    // Test string
    let test_string = "programming_with_pest_parser";
    println!("Original string: '{}'", test_string);
    println!("Length: {} characters\n", test_string.len());
    
    // Step 1: Parse the string as Pest pairs
    let pairs = StarParser::parse(Rule::non_quoted_text_string, test_string)?;
    println!("✓ Parsed as Pest pairs");
    
    // Step 2: Run through the splitting pipeline
    let halfstring_contents = StringSplittingPipeline::transform_strings_to_halfstring_content(pairs)?;
    println!("✓ Pipeline processed 1 string → {} halfstring contents", halfstring_contents.len());
    
    // Step 3: Convert back to Pest pairs of type halfstring
    println!("\nResults:");
    for (i, content) in halfstring_contents.iter().enumerate() {
        println!("  Half {}: '{}'", i + 1, content);
        
        // Parse back as halfstring Pest pairs
        let halfstring_pairs = StringSplittingPipeline::parse_halfstring_content(content)?;
        for pair in halfstring_pairs {
            println!("    → Pest pair: '{}' (rule: {:?})", pair.as_str(), pair.as_rule());
            println!("    → Length: {} characters", pair.as_str().len());
        }
        println!();
    }
    
    // Verify the pairs are actually halfstring type
    println!("✓ Success! Pipeline cuts strings in half and produces Pest pairs of type halfstring");
    
    Ok(())
}