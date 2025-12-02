//! MutablePair - a mutable alternative to pest's Pair that uses plain strings.
//!
//! This structure contains the minimum information needed to replicate a Pair's
//! functionality, but with strings instead of pest grammar tokens. This makes it
//! suitable for representing transformed/patched parse trees.

use pest::RuleType;

/// A mutable pair-like structure that mimics pest's Pair but with plain strings.
/// Unlike `Pair<Rule>`, this can be constructed and modified freely.
/// Uses String for rule names to allow synthetic rules not in the grammar.
#[derive(Debug, Clone, PartialEq)]
pub struct MutablePair {
    /// The rule name as a string (allows synthetic rules)
    pub rule_name: String,

    /// The string content (token) for this pair
    pub content: String,

    /// Starting position in the original input
    pub start: usize,

    /// Ending position in the original input
    pub end: usize,

    /// Child pairs
    pub children: Vec<MutablePair>,
}

impl MutablePair {
    /// Create a new MutablePair without children
    pub fn new(
        rule_name: impl Into<String>,
        content: impl Into<String>,
        start: usize,
        end: usize,
    ) -> Self {
        MutablePair {
            rule_name: rule_name.into(),
            content: content.into(),
            start,
            end,
            children: Vec::new(),
        }
    }

    /// Create a new MutablePair with children
    pub fn with_children(
        rule_name: impl Into<String>,
        content: impl Into<String>,
        start: usize,
        end: usize,
        children: Vec<MutablePair>,
    ) -> Self {
        MutablePair {
            rule_name: rule_name.into(),
            content: content.into(),
            start,
            end,
            children,
        }
    }

    /// Builder-style method to add children
    pub fn add_child(mut self, child: MutablePair) -> Self {
        self.children.push(child);
        self
    }

    /// Builder-style method to set children
    pub fn set_children(mut self, children: Vec<MutablePair>) -> Self {
        self.children = children;
        self
    }

    /// Get the rule name
    pub fn rule_name(&self) -> &str {
        &self.rule_name
    }

    /// Get the content as a string slice
    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// Get the start position
    pub fn start_pos(&self) -> usize {
        self.start
    }

    /// Get the end position
    pub fn end_pos(&self) -> usize {
        self.end
    }

    /// Check if this pair has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get a reference to the children
    pub fn children(&self) -> &[MutablePair] {
        &self.children
    }

    /// Get a mutable reference to the children
    pub fn children_mut(&mut self) -> &mut Vec<MutablePair> {
        &mut self.children
    }

    /// Consume self and return children
    pub fn into_children(self) -> Vec<MutablePair> {
        self.children
    }

    /// Create a MutablePair from a pest Pair
    pub fn from_pest_pair<R: RuleType>(pair: &pest::iterators::Pair<R>) -> Self {
        let children: Vec<MutablePair> = pair
            .clone()
            .into_inner()
            .map(|child| MutablePair::from_pest_pair(&child))
            .collect();

        MutablePair {
            rule_name: format!("{:?}", pair.as_rule()),
            content: pair.as_str().to_string(),
            start: pair.as_span().start(),
            end: pair.as_span().end(),
            children,
        }
    }
}

impl std::fmt::Display for MutablePair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}..{}, {:?})",
            self.rule_name, self.start, self.end, self.content
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutable_pair_creation() {
        let pair = MutablePair::new("non_quoted_string", "hello".to_string(), 0, 5);

        assert_eq!(pair.rule_name(), "non_quoted_string");
        assert_eq!(pair.as_str(), "hello");
        assert_eq!(pair.start_pos(), 0);
        assert_eq!(pair.end_pos(), 5);
        assert!(!pair.has_children());
    }

    #[test]
    fn test_mutable_pair_with_children() {
        let child1 = MutablePair::new("non_quoted_string", "a".to_string(), 0, 1);
        let child2 = MutablePair::new("non_quoted_string", "b".to_string(), 2, 3);

        let parent =
            MutablePair::with_children("data", "a b".to_string(), 0, 3, vec![child1, child2]);

        assert!(parent.has_children());
        assert_eq!(parent.children().len(), 2);
    }

    #[test]
    fn test_builder_pattern() {
        let pair = MutablePair::new("data", "parent".to_string(), 0, 10)
            .add_child(MutablePair::new(
                "non_quoted_string",
                "child1".to_string(),
                0,
                6,
            ))
            .add_child(MutablePair::new(
                "non_quoted_string",
                "child2".to_string(),
                7,
                13,
            ));

        assert_eq!(pair.children().len(), 2);
    }
}
