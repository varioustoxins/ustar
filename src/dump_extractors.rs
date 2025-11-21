//! Trait-based information extraction for dumping parse trees.
//!
//! This provides a uniform interface for extracting dump information
//! from both real Pair<Rule> objects and mutable structures.

use crate::mutable_pair::MutablePair;
use crate::Rule;
use pest::iterators::Pair;

/// A trait that defines how to extract information needed for dumping.
/// The extractor is stateless - methods take references to the object being extracted.
pub trait DumpExtractor<T: ?Sized> {
    /// The type of child item returned by get_children
    type Child;
    /// The type of iterator returned by get_children
    type ChildIter: Iterator<Item = Self::Child>;

    /// Extract the rule name from a node as a string
    fn extract_rule_name(&self, node: &T) -> String;

    /// Extract the string content from a node
    fn extract_str<'a>(&self, node: &'a T) -> &'a str;

    /// Extract the starting position
    fn extract_start(&self, node: &T) -> usize;

    /// Extract the ending position
    fn extract_end(&self, node: &T) -> usize;

    /// Check if node has children (efficient check without iteration)
    fn has_children(&self, node: &T) -> bool;

    /// Get an iterator over children
    fn get_children(&self, node: &T) -> Self::ChildIter;
}

/// Extractor for Pair<Rule> objects - stateless!
#[derive(Default)]
pub struct PairExtractor;

impl PairExtractor {
    pub fn new() -> Self {
        PairExtractor
    }
}

impl<'i> DumpExtractor<Pair<'i, Rule>> for PairExtractor {
    type Child = Pair<'i, Rule>;
    type ChildIter = pest::iterators::Pairs<'i, Rule>;

    fn extract_rule_name(&self, node: &Pair<'i, Rule>) -> String {
        format!("{:?}", node.as_rule())
    }

    fn extract_str<'a>(&self, node: &'a Pair<'i, Rule>) -> &'a str {
        node.as_str()
    }

    fn extract_start(&self, node: &Pair<'i, Rule>) -> usize {
        node.as_span().start()
    }

    fn extract_end(&self, node: &Pair<'i, Rule>) -> usize {
        node.as_span().end()
    }

    fn has_children(&self, node: &Pair<'i, Rule>) -> bool {
        node.clone().into_inner().peek().is_some()
    }

    fn get_children(&self, node: &Pair<'i, Rule>) -> Self::ChildIter {
        node.clone().into_inner()
    }
}

/// Extractor for MutablePair objects - stateless!
#[derive(Default)]
pub struct MutablePairExtractor;

impl MutablePairExtractor {
    pub fn new() -> Self {
        MutablePairExtractor
    }
}

impl DumpExtractor<MutablePair> for MutablePairExtractor {
    type Child = MutablePair; // Return owned values for consistency with Pair
    type ChildIter = std::vec::IntoIter<MutablePair>;

    fn extract_rule_name(&self, node: &MutablePair) -> String {
        node.rule_name().to_owned()
    }

    fn extract_str<'a>(&self, node: &'a MutablePair) -> &'a str {
        node.as_str()
    }

    fn extract_start(&self, node: &MutablePair) -> usize {
        node.start_pos()
    }

    fn extract_end(&self, node: &MutablePair) -> usize {
        node.end_pos()
    }

    fn has_children(&self, node: &MutablePair) -> bool {
        node.has_children()
    }

    fn get_children(&self, node: &MutablePair) -> Self::ChildIter {
        node.children().to_vec().into_iter()
    }
}

/// Dump a Pair<Rule> recursively
pub fn dump_pair(pair: &Pair<Rule>, level: usize) {
    let extractor = PairExtractor::new();
    let indent = "  ".repeat(level);
    let symbol = if extractor.has_children(pair) {
        ">"
    } else {
        "-"
    };

    println!(
        "{}{} {} {}..{} {:?}",
        indent,
        symbol,
        extractor.extract_rule_name(pair),
        extractor.extract_start(pair),
        extractor.extract_end(pair),
        extractor.extract_str(pair)
    );

    for child in extractor.get_children(pair) {
        dump_pair(&child, level + 1);
    }
}

/// Dump a MutablePair recursively
pub fn dump_mutable_pair(pair: &MutablePair, level: usize) {
    let extractor = MutablePairExtractor::new();
    let indent = "  ".repeat(level);
    let symbol = if extractor.has_children(pair) {
        ">"
    } else {
        "-"
    };

    println!(
        "{}{} {} {}..{} {:?}",
        indent,
        symbol,
        extractor.extract_rule_name(pair),
        extractor.extract_start(pair),
        extractor.extract_end(pair),
        extractor.extract_str(pair)
    );

    for child in extractor.get_children(pair) {
        dump_mutable_pair(&child, level + 1);
    }
}
