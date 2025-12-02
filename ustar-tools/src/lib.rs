// CLI utilities
pub mod dump_extractors;

// Re-export common types from the core parser for convenience
pub use ustar_parser::{mutable_pair, parse, parse_default, ParserConfig, UstarError};
