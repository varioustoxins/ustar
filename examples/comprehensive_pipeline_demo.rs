// Comprehensive demonstration of the string splitting pipeline
use ustar::{Rule, StarParser, string_splitter::{StringSplittingPipeline, SplittableString, HalfString}};
use pest::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== String Splitting Pipeline - Complete Demo ===\n");
    
    // 1. Basic Usage
    demonstrate_basic_pipeline()?;
    
    // 2. Different String Types
    demonstrate_string_types()?;
    
    // 3. Manual Step-by-Step Process
    demonstrate_manual_process()?;
    
    Ok(())
}

fn demonstrate_basic_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Basic Pipeline Usage:");
    println!("   Input: 'programming_language' → Split → Parse as halfstring pairs\n");
    
    let input = "programming_language";
    let pairs = StarParser::parse(Rule::non_quoted_text_string, input)?;
    
    let contents = StringSplittingPipeline::transform_strings_to_halfstring_content(pairs)?;
    
    for (i, content) in contents.iter().enumerate() {
        let halfstring_pairs = StringSplittingPipeline::parse_halfstring_content(content)?;
        for pair in halfstring_pairs {
            println!("   Half {}: '{}' (rule: {:?})", i+1, pair.as_str(), pair.as_rule());
        }
    }
    println!();
    
    Ok(())
}

fn demonstrate_string_types() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Different String Types:");
    
    let test_cases = vec![
        ("Non-quoted", Rule::non_quoted_text_string, "hello_world_test"),
        // Add quoted strings when they work with the grammar
    ];
    
    for (name, rule, input) in test_cases {
        println!("   {} String: '{}'", name, input);
        
        if let Ok(pairs) = StarParser::parse(rule, input) {
            let contents = StringSplittingPipeline::transform_strings_to_halfstring_content(pairs)?;
            
            for (i, content) in contents.iter().enumerate() {
                println!("     Half {}: '{}'", i+1, content);
            }
        } else {
            println!("     Parse failed for rule: {:?}", rule);
        }
        println!();
    }
    
    Ok(())
}

fn demonstrate_manual_process() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Manual Step-by-Step Process:");
    println!("   Shows the internal transformation steps\n");
    
    let input = "manual_processing_example";
    println!("   Original: '{}'", input);
    
    // Step 1: Parse to Pest pairs
    let pairs = StarParser::parse(Rule::non_quoted_text_string, input)?;
    let pair = pairs.into_iter().next().unwrap();
    println!("   Step 1 - Parsed as: {:?} = '{}'", pair.as_rule(), pair.as_str());
    
    // Step 2: Convert to SplittableString  
    let splittable = SplittableString::from_pair(&pair);
    println!("   Step 2 - SplittableString: {:?}", splittable);
    
    // Step 3: Split in half
    let (half1, half2) = splittable.split_in_half();
    println!("   Step 3 - Split: '{}' | '{}'", half1.content, half2.content);
    
    // Step 4: Convert back to Pest pairs
    let pairs1 = half1.to_pairs()?;
    let pairs2 = half2.to_pairs()?;
    
    for pair in pairs1 {
        println!("   Step 4a - First half as pest pair: '{}' (rule: {:?})", pair.as_str(), pair.as_rule());
    }
    
    for pair in pairs2 {
        println!("   Step 4b - Second half as pest pair: '{}' (rule: {:?})", pair.as_str(), pair.as_rule());
    }
    
    println!();
    Ok(())
}