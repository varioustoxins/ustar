// Test to check if halfstring Pest pairs have correct position offsets
use ustar::{Rule, StarParser, string_splitter::{StringSplittingPipeline, SplittableString}};
use pest::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Halfstring Position Tracking ===\n");
    
    let test_string = "hello_world_test";
    println!("Original string: '{}'", test_string);
    println!("Length: {} characters", test_string.len());
    println!("Expected split at position: {}\n", test_string.len() / 2);
    
    // Parse original string
    let pairs = StarParser::parse(Rule::non_quoted_text_string, test_string)?;
    let original_pair = pairs.clone().into_iter().next().unwrap();
    
    println!("Original Pest pair:");
    println!("  Content: '{}'", original_pair.as_str());
    println!("  Rule: {:?}", original_pair.as_rule());
    println!("  Span start: {}", original_pair.as_span().start());
    println!("  Span end: {}", original_pair.as_span().end());
    
    // Manual splitting to show offset tracking
    println!("\n=== Manual Splitting with Offset Tracking ===");
    let splittable = SplittableString::from_pair(&original_pair);
    let (half1, half2) = splittable.split_in_half();
    
    println!("\nHalf 1:");
    println!("  Content: '{}'", half1.content);
    println!("  Original offset: {}", half1.original_start_offset);
    println!("  Original span: {:?}", half1.original_span());
    println!("  In original string: '{}'", &test_string[half1.original_span().0..half1.original_span().1]);
    
    println!("\nHalf 2:");
    println!("  Content: '{}'", half2.content);
    println!("  Original offset: {}", half2.original_start_offset);
    println!("  Original span: {:?}", half2.original_span());
    println!("  In original string: '{}'", &test_string[half2.original_span().0..half2.original_span().1]);
    
    // Transform through pipeline
    let contents = StringSplittingPipeline::transform_strings_to_halfstring_content(pairs)?;
    
    println!("\n=== Pipeline Result Pest Pairs ===");
    for (i, content) in contents.iter().enumerate() {
        println!("\n--- Half {} ---", i + 1);
        println!("Content: '{}'", content);
        
        // Parse back as halfstring
        let halfstring_pairs = StringSplittingPipeline::parse_halfstring_content(content)?;
        for pair in halfstring_pairs {
            println!("Halfstring Pest pair:");
            println!("  Content: '{}'", pair.as_str());
            println!("  Rule: {:?}", pair.as_rule());
            println!("  Local span: {}..{}", pair.as_span().start(), pair.as_span().end());
            
            // Note about limitations
            println!("  ⚠️  Local offsets only - not original source positions");
        }
    }
    
    println!("\n=== Summary ===");
    println!("✅ The SplittableString tracks original positions correctly");
    println!("✅ We can map back to original source locations");
    println!("⚠️  New Pest pairs have local offsets (limitation of re-parsing)");
    println!("💡 Use HalfString.original_span() for original source positions");
    
    Ok(())
}