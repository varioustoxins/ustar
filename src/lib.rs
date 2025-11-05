use pest_derive::Parser;
use std::collections::HashMap;
use std::any::Any;

#[derive(Parser)]
#[grammar = "star.pest"]
pub struct StarParser;

// Re-export commonly used types for external use
pub use pest::iterators::{Pair, Pairs};
pub use pest::Parser;
pub use pest::RuleType;

// Dumpable trait and implementations
pub mod dump_extractors;

// MutablePair - mutable alternative to Pair
pub mod mutable_pair;

// String decomposer - transforms MutablePairs to decompose strings (internal)
mod string_decomposer;

/// Configuration options for the USTAR parser
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum UstarConfiguration {
    /// Decompose string tokens into delimiter + content + delimiter; default is true
    DecomposedStrings,
}

/// Parse STAR format input with configuration options
///
/// # Arguments
/// * `input` - The input string to parse
/// * `configuration` - A map of configuration options to their values
///
/// # Returns
/// * `Result<mutable_pair::MutablePair, String>` - Parsed result as a MutablePair tree, or an error message
pub fn parse(
    input: &str,
    configuration: HashMap<UstarConfiguration, Box<dyn Any>>,
) -> Result<mutable_pair::MutablePair, String> {
    // Parse the input using the Pest parser
    let pairs = StarParser::parse(Rule::star_file, input)
        .map_err(|e| format!("Parse error: {}", e))?;
    
    // Convert to MutablePair tree
    let mut root_pairs: Vec<mutable_pair::MutablePair> = pairs
        .map(|p| mutable_pair::MutablePair::from_pest_pair(&p))
        .collect();
    
    // Apply transformations based on configuration
    // Default is to decompose strings (true), unless explicitly set to false
    let should_decompose = configuration
        .get(&UstarConfiguration::DecomposedStrings)
        .and_then(|v| v.downcast_ref::<bool>())
        .copied()
        .unwrap_or(true);
    
    if should_decompose {
        for pair in &mut root_pairs {
            string_decomposer::decompose_strings(pair);
        }
    }
    
    // For now, return the first root pair or create an empty one
    if root_pairs.is_empty() {
        Ok(mutable_pair::MutablePair::new(
            "star_file",
            String::new(),
            0,
            0,
        ))
    } else if root_pairs.len() == 1 {
        Ok(root_pairs.remove(0))
    } else {
        // Multiple root elements - wrap them in a container
        Ok(mutable_pair::MutablePair::with_children(
            "star_file",
            input.to_string(),
            0,
            input.len(),
            root_pairs,
        ))
    }
}
