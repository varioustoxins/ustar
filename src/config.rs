use std::collections::HashMap;

/// Character encoding mode for the USTAR parser
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub enum EncodingMode {
    /// ASCII-only mode: Characters 0x21-0x7E ('!' to '~')
    /// Whitespace: space (0x20) and tab (0x09)
    #[default]
    Ascii,

    /// Extended ASCII mode: Characters 0x00-0xFF
    /// Whitespace: ASCII whitespace plus non-breaking space (0xA0)
    ExtendedAscii,

    /// Full Unicode mode: All Unicode characters
    /// Whitespace: All 25 Unicode whitespace characters
    /// Supports UTF-8 BOM detection
    Unicode,
}


/// Error formatting mode for runtime display
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ErrorFormatMode {
    /// Simple [line:col] format
    Basic,
    /// ASCII pest error report
    Ascii,
    /// Full miette formatting (requires extended-errors feature)
    Fancy,
}

impl Default for ErrorFormatMode {
    fn default() -> Self {
        #[cfg(feature = "extended-errors")]
        return ErrorFormatMode::Fancy;
        #[cfg(not(feature = "extended-errors"))]
        return ErrorFormatMode::Ascii;
    }
}

/// Configuration keys for the USTAR parser
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ConfigKey {
    /// Whether to decompose string tokens into delimiter + content + delimiter (value: bool)
    DecomposedStrings,

    /// Character encoding mode (value: EncodingMode)
    Encoding,

    /// Whether to auto-detect BOM and override encoding (value: bool)
    AutoDetectBom,

    /// Error format mode for runtime error display (value: ErrorFormatMode)
    ErrorFormat,

    /// Number of context lines to display around errors (value: usize, ignored by Basic mode)
    ContextLines,
}

/// Parser configuration as a HashMap
pub type ParserConfig = HashMap<ConfigKey, ConfigValue>;

/// Configuration value enum to hold different types
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Bool(bool),
    Encoding(EncodingMode),
    ErrorFormat(ErrorFormatMode),
    Usize(usize),
}

impl ConfigValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_encoding(&self) -> Option<EncodingMode> {
        match self {
            ConfigValue::Encoding(e) => Some(*e),
            _ => None,
        }
    }

    pub fn as_error_format(&self) -> Option<ErrorFormatMode> {
        match self {
            ConfigValue::ErrorFormat(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        match self {
            ConfigValue::Usize(n) => Some(*n),
            _ => None,
        }
    }
}

/// Create default parser configuration
pub fn default_config() -> ParserConfig {
    let mut config = HashMap::new();
    config.insert(ConfigKey::DecomposedStrings, ConfigValue::Bool(true));
    config.insert(
        ConfigKey::Encoding,
        ConfigValue::Encoding(EncodingMode::Ascii),
    );
    config.insert(ConfigKey::AutoDetectBom, ConfigValue::Bool(false));
    config.insert(
        ConfigKey::ErrorFormat,
        ConfigValue::ErrorFormat(ErrorFormatMode::default()),
    );
    config.insert(ConfigKey::ContextLines, ConfigValue::Usize(3)); // Default to 3 lines of context
    config
}

/// Get auto_detect_bom setting from configuration
pub fn get_auto_detect_bom(config: &ParserConfig) -> bool {
    config
        .get(&ConfigKey::AutoDetectBom)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Get decomposed_strings setting from configuration
pub fn get_decomposed_strings(config: &ParserConfig) -> bool {
    config
        .get(&ConfigKey::DecomposedStrings)
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

/// Get encoding mode from configuration
pub fn get_encoding(config: &ParserConfig) -> EncodingMode {
    config
        .get(&ConfigKey::Encoding)
        .and_then(|v| v.as_encoding())
        .unwrap_or_default()
}

/// Get error format mode from configuration
pub fn get_error_format(config: &ParserConfig) -> ErrorFormatMode {
    config
        .get(&ConfigKey::ErrorFormat)
        .and_then(|v| v.as_error_format())
        .unwrap_or_default()
}

/// Get context lines setting from configuration
pub fn get_context_lines(config: &ParserConfig) -> usize {
    config
        .get(&ConfigKey::ContextLines)
        .and_then(|v| v.as_usize())
        .unwrap_or(3) // Default to 3 lines
}
