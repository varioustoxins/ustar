use crate::config::EncodingMode;
use crate::error_core::ErrorData;
use crate::ErrorFormatMode;
use miette::{Diagnostic, SourceSpan};

/// USTAR parsing error types with rich diagnostics
#[derive(thiserror::Error, Debug, Clone, Diagnostic)]
#[allow(unused_assignments)] // Fields used by miette macros
pub enum UstarError {
    #[error("{core}")]
    ParseError {
        #[diagnostic(transparent)]
        core: ErrorData,
        #[source_code]
        src: String,
        #[label("Error occurred here")]
        error_span: SourceSpan,
    },
}

impl UstarError {
    pub fn from_pest_error<R: pest::RuleType>(
        error: pest::error::Error<R>,
        encoding: EncodingMode,
        input: &str,
    ) -> Self {
        let error_span = match &error.location {
            pest::error::InputLocation::Pos(pos) => (*pos, 0).into(),
            pest::error::InputLocation::Span((start, end)) => (*start, *end - *start).into(),
        };

        let core = ErrorData::from_pest_error(error, encoding, input);

        UstarError::ParseError {
            src: core.src.clone(),
            error_span,
            core,
        }
    }

    /// Format error according to specified mode
    pub fn format_error(&self, mode: ErrorFormatMode, context_lines: usize) -> String {
        match mode {
            ErrorFormatMode::Basic => match self {
                UstarError::ParseError { core, .. } => core.format_basic(),
            },
            ErrorFormatMode::Ascii => match self {
                UstarError::ParseError { core, .. } => core.format_ascii(context_lines),
            },
            ErrorFormatMode::Fancy => {
                // For fancy mode, we'll create a custom GraphicalReportHandler
                use miette::{GraphicalReportHandler, GraphicalTheme};
                let handler = GraphicalReportHandler::new()
                    .with_context_lines(context_lines)
                    .with_theme(GraphicalTheme::unicode());

                let mut output = String::new();
                handler
                    .render_report(&mut output, self)
                    .unwrap_or_else(|_| {
                        output.push_str(&format!("{:?}", miette::Report::new(self.clone())));
                    });
                output
            }
        }
    }
}
