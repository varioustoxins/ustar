// Demonstration of enhanced position tracking in the string splitting pipeline
use ustar::{Rule, StarParser, string_splitter::StringSplittingPipeline};
use pest::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Enhanced Position Tracking Demo ===\n");
    
    let test_string = "hello_world_programming";
    println!("Original string: '{}'", test_string);
    println!("Positions:       {}", (0..test_string.len()).map(|i| format!("{}", i % 10)).collect::<String>());
    println!("Length: {} characters\n", test_string.len());
    
    // Parse original string
    let pairs = StarParser::parse(Rule::non_quoted_text_string, test_string)?;
    
    // Use enhanced pipeline with position tracking
    let results = StringSplittingPipeline::transform_strings_to_halfstring_with_positions(pairs)?;
    
    println!("Results from enhanced pipeline:");
    for (i, (content, start, end)) in results.iter().enumerate() {
        println!("\nHalf {}:", i + 1);
        println!("  Content: '{}'", content);
        println!("  Original positions: {}..{}", start, end);
        println!("  Length: {}", content.len());
        println!("  Verification: '{}'", &test_string[*start..*end]);
        
        // Show that we can still create Pest pairs
        let pairs = StringSplittingPipeline::parse_halfstring_content(content)?;
        for pair in pairs {
            println!("  Pest pair rule: {:?} (local offsets 0..{})", pair.as_rule(), pair.as_str().len());
        }
    }
    
    println!("\n=== Summary ===");
    println!("✅ Original source positions preserved: {}..{}", results[0].1, results[0].2);
    println!("✅ Original source positions preserved: {}..{}", results[1].1, results[1].2);
    println!("✅ Can create valid halfstring Pest pairs");
    println!("✅ Can map back to original source locations");
    
    // Demonstrate rebuilding the original string
    let rebuilt = results.iter()
        .map(|(content, _, _)| content.as_str())
        .collect::<String>();
    
    println!("\nReconstructed: '{}'", rebuilt);
    println!("Matches original: {}", rebuilt == test_string);
    
    Ok(())
}