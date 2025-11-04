use pest_derive::Parser;

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
