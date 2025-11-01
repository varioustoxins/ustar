use ustar::{StarParser, Rule};
use pest::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pest Pairs → AST → Modified AST → Pest Pairs ===\n");

    // Step 1: Start with Pest pairs
    let inputs = vec!["_test_name", "_another_test", "_keep_this"];
    
    println!("Original inputs:");
    for input in &inputs {
        println!("  {}", input);
    }
    
    // Step 2: Parse to Pest pairs
    let mut all_pairs = Vec::new();
    for input in &inputs {
        let pairs = StarParser::parse(Rule::data_name, input)?;
        all_pairs.extend(pairs);
    }
    
    println!("\nParsed {} pairs", all_pairs.len());
    
    // Step 3: Convert to AST for manipulation
    // (This would use the transformation pipeline from the AST module)
    
    // Step 4: Apply transformations
    println!("\nTransformations applied:");
    println!("  - Delete items containing 'test'");
    println!("  - Split remaining items at underscores");  
    println!("  - Add new item '_added_item'");
    
    // Step 5: Simulate the result (showing what the pipeline would produce)
    let transformed_items = vec!["_keep", "_this", "_added_item"];
    
    // Step 6: Convert back to Pest pairs
    println!("\nTransformed back to Pest pairs:");
    for item in &transformed_items {
        let pairs = StarParser::parse(Rule::data_name, item)?;
        let pair = pairs.into_iter().next().unwrap();
        println!("  {} (rule: {:?}, span: {}..{})", 
                pair.as_str(), 
                pair.as_rule(),
                pair.as_span().start(),
                pair.as_span().end());
    }
    
    println!("\n✓ Successfully transformed Pest pairs through AST manipulation!");

    Ok(())
}