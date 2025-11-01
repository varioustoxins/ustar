use pest::iterators::{Pair, Pairs};
use pest::Parser;
use crate::{Rule, StarParser};

/// Simple string wrapper that can be split
#[derive(Clone, Debug, PartialEq)]
pub struct SplittableString {
    pub content: String,
    pub original_rule: Rule,
}

impl SplittableString {
    /// Create from a Pest pair
    pub fn from_pair(pair: &Pair<Rule>) -> Self {
        Self {
            content: pair.as_str().to_string(),
            original_rule: pair.as_rule(),
        }
    }
    
    /// Split string in half with position tracking
    pub fn split_in_half(&self) -> (HalfString, HalfString) {
        let content = self.get_inner_content();
        let mid = content.len() / 2;
        
        let first_half = &content[..mid];
        let second_half = &content[mid..];
        
        // Calculate offsets based on whether this is a quoted string
        let content_start_offset = match self.original_rule {
            Rule::double_quote_string | Rule::single_quote_string => 1, // Skip opening quote
            _ => 0
        };
        
        (
            HalfString::new(first_half.to_string(), content_start_offset),
            HalfString::new(second_half.to_string(), content_start_offset + mid)
        )
    }
    
    /// Get content without quotes for splitting
    fn get_inner_content(&self) -> &str {
        let content = &self.content;
        match self.original_rule {
            Rule::double_quote_string => {
                if content.starts_with('"') && content.ends_with('"') && content.len() >= 2 {
                    &content[1..content.len()-1]
                } else {
                    content
                }
            }
            Rule::single_quote_string => {
                if content.starts_with('\'') && content.ends_with('\'') && content.len() >= 2 {
                    &content[1..content.len()-1]
                } else {
                    content
                }
            }
            _ => content
        }
    }
}

/// Represents half of a split string that can be converted back to Pest pairs
#[derive(Clone, Debug, PartialEq)]
pub struct HalfString {
    pub content: String,
    pub original_start_offset: usize,  // Position in the original source
}

impl HalfString {
    /// Create a new HalfString with original offset information
    pub fn new(content: String, original_start_offset: usize) -> Self {
        Self {
            content,
            original_start_offset,
        }
    }
    
    /// Convert halfstring back to Pest pairs of rule 'halfstring'
    /// Note: These will have local offsets (0..content.len()), not original source offsets
    pub fn to_pairs(&self) -> Result<Pairs<Rule>, pest::error::Error<Rule>> {
        StarParser::parse(Rule::halfstring, &self.content)
    }
    
    /// Convert to a single Pest pair
    /// Note: This will have local offsets (0..content.len()), not original source offsets
    pub fn to_pair(&self) -> Result<Pair<Rule>, pest::error::Error<Rule>> {
        let mut pairs = self.to_pairs()?;
        pairs.next().ok_or_else(|| {
            pest::error::Error::new_from_pos(
                pest::error::ErrorVariant::CustomError {
                    message: "No pairs generated".to_string(),
                },
                pest::Position::from_start("")
            )
        })
    }
    
    /// Get the original source position information
    pub fn original_span(&self) -> (usize, usize) {
        (self.original_start_offset, self.original_start_offset + self.content.len())
    }
}

/// Pipeline that transforms string pairs into halfstring pairs
pub struct StringSplittingPipeline;

impl StringSplittingPipeline {
    /// Main transformation: Pairs → Split → HalfString content
    /// Returns the content strings that can be parsed as halfstring rules
    pub fn transform_strings_to_halfstring_content(pairs: Pairs<Rule>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut result_strings = Vec::new();
        
        for pair in pairs {
            match pair.as_rule() {
                Rule::double_quote_string | Rule::single_quote_string | Rule::non_quoted_text_string => {
                    // Convert to splittable string
                    let splittable = SplittableString::from_pair(&pair);
                    
                    // Split in half
                    let (half1, half2) = splittable.split_in_half();
                    
                    // Add the content strings (these can be parsed as halfstring later)
                    result_strings.push(half1.content);
                    result_strings.push(half2.content);
                }
                _ => {
                    // For non-string pairs, keep original content
                    result_strings.push(pair.as_str().to_string());
                }
            }
        }
        
        Ok(result_strings)
    }
    
    /// Convenience method to convert content back to halfstring pairs
    pub fn parse_halfstring_content(content: &str) -> Result<Pairs<Rule>, pest::error::Error<Rule>> {
        StarParser::parse(Rule::halfstring, content)
    }
    
    /// Enhanced transformation that returns both content and original position info
    pub fn transform_strings_to_halfstring_with_positions(pairs: Pairs<Rule>) -> Result<Vec<(String, usize, usize)>, Box<dyn std::error::Error>> {
        let mut result_with_positions = Vec::new();
        
        for pair in pairs {
            match pair.as_rule() {
                Rule::double_quote_string | Rule::single_quote_string | Rule::non_quoted_text_string => {
                    // Convert to splittable string
                    let splittable = SplittableString::from_pair(&pair);
                    
                    // Split in half
                    let (half1, half2) = splittable.split_in_half();
                    
                    // Add content with original position information
                    let (start1, end1) = half1.original_span();
                    let (start2, end2) = half2.original_span();
                    
                    result_with_positions.push((half1.content, start1, end1));
                    result_with_positions.push((half2.content, start2, end2));
                }
                _ => {
                    // For non-string pairs, keep original position
                    let content = pair.as_str().to_string();
                    let start = pair.as_span().start();
                    let end = pair.as_span().end();
                    result_with_positions.push((content, start, end));
                }
            }
        }
        
        Ok(result_with_positions)
    }
    
    /// Demo function showing the pipeline in action
    pub fn demo() -> Result<(), Box<dyn std::error::Error>> {
        println!("=== String Splitting Pipeline Demo ===\n");
        
        // Test inputs - using strings that will work with your grammar
        let test_cases = vec![
            (Rule::non_quoted_text_string, "hello_world"),
            (Rule::non_quoted_text_string, "test_string_longer"),
            (Rule::non_quoted_text_string, "short"),
        ];
        
        for (rule, input) in test_cases {
            println!("Input: '{}' (rule: {:?})", input, rule);
            
            // Parse to original pairs
            let original_pairs = StarParser::parse(rule, input)?;
            
            // Transform through pipeline to get content strings
            let halfstring_contents = Self::transform_strings_to_halfstring_content(original_pairs)?;
            
            println!("Output halfstring contents:");
            for (i, content) in halfstring_contents.iter().enumerate() {
                println!("  Half {}: '{}'", i+1, content);
                
                // Show how to parse it back as halfstring
                match Self::parse_halfstring_content(content) {
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
            println!();
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_splitting_pipeline() {
        let input = "test_string";
        let pairs = StarParser::parse(Rule::non_quoted_text_string, input).unwrap();
        
        let result = StringSplittingPipeline::transform_strings_to_halfstring_content(pairs).unwrap();
        
        assert_eq!(result.len(), 2); // Should produce 2 halfstring contents
        assert_eq!(result[0], "test_");
        assert_eq!(result[1], "string");
        
        // Test parsing back as halfstring
        let first_pairs = StringSplittingPipeline::parse_halfstring_content(&result[0]).unwrap();
        let first_pair = first_pairs.into_iter().next().unwrap();
        assert_eq!(first_pair.as_str(), "test_");
        assert_eq!(first_pair.as_rule(), Rule::halfstring);
        
        let second_pairs = StringSplittingPipeline::parse_halfstring_content(&result[1]).unwrap();
        let second_pair = second_pairs.into_iter().next().unwrap();
        assert_eq!(second_pair.as_str(), "string");
        assert_eq!(second_pair.as_rule(), Rule::halfstring);
    }
    
    #[test]
    fn test_splittable_string() {
        // Test creating from pair
        let input = "hello_world";
        let pairs = StarParser::parse(Rule::non_quoted_text_string, input).unwrap();
        let pair = pairs.into_iter().next().unwrap();
        
        let splittable = SplittableString::from_pair(&pair);
        assert_eq!(splittable.content, "hello_world");
        assert_eq!(splittable.original_rule, Rule::non_quoted_text_string);
        
        // Test splitting
        let (half1, half2) = splittable.split_in_half();
        assert_eq!(half1.content, "hello");
        assert_eq!(half2.content, "_world");
        
        // Test offset tracking
        assert_eq!(half1.original_start_offset, 0);
        assert_eq!(half2.original_start_offset, 5);
        
        // Test original span calculation
        assert_eq!(half1.original_span(), (0, 5));
        assert_eq!(half2.original_span(), (5, 11));
    }
}