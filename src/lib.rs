use pest::Parser as PestParser;

mod config;
mod error_core;
pub mod parsers;

#[cfg(feature = "extended-errors")]
mod extended_errors;
#[cfg(not(feature = "extended-errors"))]
mod simple_errors;

// Re-export the appropriate error type based on features
#[cfg(feature = "extended-errors")]
pub use extended_errors::UstarError;
#[cfg(not(feature = "extended-errors"))]
pub use simple_errors::UstarError;

pub use config::{
    default_config, get_context_lines, get_decomposed_strings, get_encoding, get_error_format,
    ConfigKey, ConfigValue, EncodingMode, ErrorFormatMode, ParserConfig,
};
pub use parsers::Rule;

// Re-export commonly used types for external use
pub use pest::iterators::{Pair, Pairs};
pub use pest::RuleType;

// Dumpable trait and implementations
pub mod dump_extractors;

// MutablePair - mutable alternative to Pair
pub mod mutable_pair;

// Buffered handler traits and walker
pub mod sas_interface;
pub mod sas_walker;

// String decomposer - transforms MutablePair strings to decomposed strings
pub mod string_decomposer;

/// Configuration options for the USTAR parser
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum UstarConfiguration {
    /// Decompose string tokens into delimiter + content + delimiter; default is true
    DecomposedStrings,
}

/// Format a pest error according to the configuration error format
pub fn format_parse_error<R: pest::RuleType>(
    pest_error: &pest::error::Error<R>,
    input: &str,
    encoding: EncodingMode,
    format_mode: ErrorFormatMode,
    context_lines: usize,
) -> String {
    let ustar_error = UstarError::from_pest_error(pest_error.clone(), encoding, input);
    ustar_error.format_error(format_mode, context_lines)
}

// Helper function to process pairs from any parser
fn process_pairs<'a, R>(
    pairs: impl Iterator<Item = pest::iterators::Pair<'a, R>>,
) -> Vec<mutable_pair::MutablePair>
where
    R: pest::RuleType,
{
    pairs
        .map(|p| mutable_pair::MutablePair::from_pest_pair(&p))
        .collect()
}

fn split_pairs_if_requested(pairs: &mut [mutable_pair::MutablePair], config: &ParserConfig) {
    if get_decomposed_strings(config) {
        for pair in pairs.iter_mut() {
            string_decomposer::decompose_strings(pair);
        }
    }
}

/// Parse STAR format input with configuration options
///
/// # Arguments
/// * `input` - The input string to parse
/// * `config` - A map of configuration options to their values
///
/// # Returns
/// * `Result<mutable_pair::MutablePair, UstarError>` - Parsed result as a MutablePair tree, or an error with diagnostics
pub fn parse(
    input: &str,
    config: &ParserConfig,
) -> Result<mutable_pair::MutablePair, Box<UstarError>> {
    // BOM auto-detection is controlled by config
    let auto_detect_bom = config::get_auto_detect_bom(config);
    let (encoding, input_clean) = if auto_detect_bom && input.starts_with('\u{FEFF}') {
        (EncodingMode::Unicode, &input[3..])
    } else {
        (get_encoding(config), input)
    };

    // Choose the appropriate parser based on encoding mode
    let mut result = match encoding {
        EncodingMode::Ascii => {
            let pairs =
                parsers::ascii::AsciiParser::parse(parsers::ascii::Rule::star_file, input_clean)
                    .map_err(|e| Box::new(UstarError::from_pest_error(e, encoding, input)))?;
            process_pairs(pairs)
        }
        EncodingMode::ExtendedAscii => {
            let pairs = parsers::extended::ExtendedParser::parse(
                parsers::extended::Rule::star_file,
                input_clean,
            )
            .map_err(|e| Box::new(UstarError::from_pest_error(e, encoding, input)))?;
            process_pairs(pairs)
        }
        EncodingMode::Unicode => {
            let pairs = parsers::unicode::UnicodeParser::parse(
                parsers::unicode::Rule::star_file,
                input_clean,
            )
            .map_err(|e| Box::new(UstarError::from_pest_error(e, encoding, input)))?;
            process_pairs(pairs)
        }
    };

    split_pairs_if_requested(&mut result, config);

    // For now, return the first root pair or create an empty one
    if result.is_empty() {
        Ok(mutable_pair::MutablePair::new(
            "star_file",
            String::new(),
            0,
            0,
        ))
    } else if result.len() == 1 {
        Ok(result.into_iter().next().unwrap())
    } else {
        // Multiple root elements - wrap them in a container
        Ok(mutable_pair::MutablePair::with_children(
            "star_file",
            input,
            0,
            input.len(),
            result,
        ))
    }
}

/// Parse with default configuration (ASCII mode, decomposed strings, fancy error formatting)
pub fn parse_default(input: &str) -> Result<mutable_pair::MutablePair, Box<UstarError>> {
    parse(input, &default_config())
}
