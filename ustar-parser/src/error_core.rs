use crate::config::EncodingMode;

/// Core error data shared between extended and simple error implementations
#[cfg(feature = "extended-errors")]
use miette::{Diagnostic, SourceSpan};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "extended-errors", derive(thiserror::Error, Diagnostic))]
#[cfg_attr(feature = "extended-errors", error("{message}"))]
#[allow(dead_code)] // Fields used by miette macros
pub struct ErrorData {
    pub encoding: EncodingMode,
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub line_content: String,
    pub pest_error_display: String,
    #[cfg_attr(feature = "extended-errors", source_code)]
    pub src: String,
    #[cfg_attr(feature = "extended-errors", label("Error occurred here"))]
    #[cfg(feature = "extended-errors")]
    pub error_span: SourceSpan,
}

impl ErrorData {
    /// Create ErrorData from a pest error
    pub fn from_pest_error<R: pest::RuleType>(
        error: pest::error::Error<R>,
        encoding: EncodingMode,
        input: &str,
    ) -> Self {
        let (line, col) = match error.line_col {
            pest::error::LineColLocation::Pos((line, col)) => (line, col),
            pest::error::LineColLocation::Span((line, col), _) => (line, col),
        };
        let line_content = Self::get_line_content_from_pest(input, &error);
        let pest_error_display = format!("{}", error);

        // Extract simple error message for later formatting
        let simple_message = match &error.variant {
            pest::error::ErrorVariant::ParsingError {
                positives,
                negatives: _,
            } => {
                if positives.is_empty() {
                    "Unexpected input".to_string()
                } else {
                    let tokens = positives
                        .iter()
                        .map(|r| format!("{:?}", r))
                        .collect::<Vec<_>>();
                    let joined = tokens.join(", ");
                    let final_message = if joined.contains(", ") {
                        // Replace the last ", " with " or "
                        let mut parts: Vec<&str> = joined.rsplitn(2, ", ").collect();
                        parts.reverse();
                        parts.join(" or ")
                    } else {
                        joined
                    };
                    format!("Expected {}", final_message)
                }
            }
            pest::error::ErrorVariant::CustomError { message } => message.clone(),
        };

        #[cfg(feature = "extended-errors")]
        let error_span = match &error.location {
            pest::error::InputLocation::Pos(pos) => (*pos, 0).into(),
            pest::error::InputLocation::Span((start, end)) => (*start, *end - *start).into(),
        };

        ErrorData {
            encoding,
            message: simple_message,
            line,
            col,
            line_content,
            pest_error_display,
            src: input.to_string(),
            #[cfg(feature = "extended-errors")]
            error_span,
        }
    }

    /// Format error in basic format (similar to ASCII but minimal)
    pub fn format_basic(&self) -> String {
        // Use pest-style basic format with minimal context
        let result = format!(
            "Parse error at l{}:c{} because {}\n",
            self.line,
            self.col,
            self.message.to_lowercase()
        );

        result
    }

    /// Format error using Pest-style display with controlled context
    pub fn format_ascii(&self, context_lines: usize) -> String {
        let lines: Vec<&str> = self.src.lines().collect();

        // Calculate context range - show context_lines before and after
        let start_line = self.line.saturating_sub(context_lines + 1);
        let end_line = (self.line + context_lines).min(lines.len());

        // Calculate the width needed for line numbers
        let max_line_num = end_line;
        let line_num_width = max_line_num.to_string().len();

        let mut result = format!(
            "x {}\n --> {}:{}\n{} |\n",
            self.message,
            self.line,
            self.col,
            " ".repeat(line_num_width)
        );

        // Show context lines with Pest-style formatting
        for line_num in start_line..end_line {
            let line_content = lines.get(line_num).unwrap_or(&"");
            let display_line_num = line_num + 1;

            result.push_str(&format!(
                "{:width$} | {}\n",
                display_line_num,
                line_content,
                width = line_num_width
            ));

            // Add pointer under the error line
            if display_line_num == self.line {
                result.push_str(&format!(
                    "{} | {}^---\n",
                    " ".repeat(line_num_width),
                    " ".repeat(self.col.saturating_sub(1))
                ));
            }
        }

        result
    }

    /// Get line content from pest error
    fn get_line_content_from_pest(
        input: &str,
        pest_error: &pest::error::Error<impl pest::RuleType>,
    ) -> String {
        let offset = match &pest_error.location {
            pest::error::InputLocation::Pos(pos) => *pos,
            pest::error::InputLocation::Span((start, _)) => *start,
        };

        let lines: Vec<&str> = input.lines().collect();
        let mut char_count = 0;

        for line in lines.iter() {
            let line_end = char_count + line.len() + 1; // +1 for newline
            if offset < line_end {
                return line.to_string();
            }
            char_count = line_end;
        }

        "".to_string()
    }
}
