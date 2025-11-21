use crate::config::EncodingMode;
use crate::error_core::ErrorData;
use crate::ErrorFormatMode;

/// USTAR parsing error types (simple version without miette dependencies)
#[derive(Debug, Clone)]
pub enum UstarError {
    ParseError(ErrorData),
}

impl std::fmt::Display for UstarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UstarError::ParseError(core) => {
                write!(f, "{}", core.pest_error_display)
            }
        }
    }
}

impl std::error::Error for UstarError {}

impl UstarError {
    pub fn from_pest_error<R: pest::RuleType>(
        error: pest::error::Error<R>,
        encoding: EncodingMode,
        input: &str,
    ) -> Self {
        let core = ErrorData::from_pest_error(error, encoding, input);
        UstarError::ParseError(core)
    }

    /// Format error according to specified mode
    pub fn format_error(&self, mode: ErrorFormatMode, context_lines: usize) -> String {
        match self {
            UstarError::ParseError(core) => {
                match mode {
                    ErrorFormatMode::Basic => core.format_basic(),
                    ErrorFormatMode::Ascii => core.format_ascii(context_lines),
                    ErrorFormatMode::Fancy => {
                        // Fallback to pest error display when extended-errors feature is disabled
                        core.pest_error_display.clone()
                    }
                }
            }
        }
    }
}
