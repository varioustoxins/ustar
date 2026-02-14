//! Stable formatting for pest `Pair` trees.
//!
//! Pest's `Debug` implementation for `Span` has changed between versions
//! (e.g., `start: N, end: M` in 2.8.4 vs `range: N..M` in 2.8.6).
//! This module provides a stable formatter that won't break snapshot tests
//! when pest releases semver-compatible updates.

use pest::RuleType;
use std::fmt::Write;

/// Format a pest `Pair` as a stable, pretty-printed string.
///
/// Produces output structurally similar to pest's `{:#?}` Debug format
/// but with a stable span representation using `range: start..end`.
///
/// # Example output
///
/// ```text
/// Pair {
///     rule: star_file,
///     span: Span {
///         str: "data_example\n...",
///         range: 0..4566,
///     },
///     inner: [
///         Pair {
///             rule: data_heading,
///             span: Span {
///                 str: "data_example",
///                 range: 0..12,
///             },
///             inner: [],
///         },
///     ],
/// }
/// ```
pub fn format_pest_pair<R: RuleType>(pair: &pest::iterators::Pair<'_, R>) -> String {
    let mut output = String::new();
    write_pair(pair, 0, &mut output);
    // Remove the trailing newline to match Debug convention
    if output.ends_with('\n') {
        output.pop();
    }
    output
}

fn write_pair<R: RuleType>(
    pair: &pest::iterators::Pair<'_, R>,
    indent: usize,
    output: &mut String,
) {
    let pad = " ".repeat(indent);
    let span = pair.as_span();

    writeln!(output, "{pad}Pair {{").unwrap();
    writeln!(output, "{pad}    rule: {:?},", pair.as_rule()).unwrap();
    writeln!(output, "{pad}    span: Span {{").unwrap();
    writeln!(output, "{pad}        str: {:?},", span.as_str()).unwrap();
    writeln!(
        output,
        "{pad}        range: {}..{},",
        span.start(),
        span.end()
    )
    .unwrap();
    writeln!(output, "{pad}    }},").unwrap();

    let inner: Vec<_> = pair.clone().into_inner().collect();
    if inner.is_empty() {
        writeln!(output, "{pad}    inner: [],").unwrap();
    } else {
        writeln!(output, "{pad}    inner: [").unwrap();
        for child in &inner {
            write_pair(child, indent + 8, output);
        }
        writeln!(output, "{pad}    ],").unwrap();
    }

    writeln!(output, "{pad}}}").unwrap();
}

#[cfg(test)]
mod tests {
    // Integration testing of format_pest_pair() happens in ustar-parser's
    // snapshot tests where real grammar Pairs are available. We can't easily
    // create pest Pairs without a grammar, so we just verify compilation here.
}
