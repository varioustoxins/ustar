use std::collections::HashMap;

/// Character encoding mode for the USTAR parser
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum EncodingMode {
    /// ASCII-only mode: Characters 0x21-0x7E ('!' to '~')
    /// Whitespace: space (0x20) and tab (0x09)
    Ascii,
    
    /// Extended ASCII mode: Characters 0x00-0xFF
    /// Whitespace: ASCII whitespace plus non-breaking space (0xA0)
    ExtendedAscii,
    
    /// Full Unicode mode: All Unicode characters
    /// Whitespace: All 25 Unicode whitespace characters
    /// Supports UTF-8 BOM detection
    Unicode,
}

impl Default for EncodingMode {
    fn default() -> Self {
        EncodingMode::Ascii
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
}

/// Parser configuration as a HashMap
pub type ParserConfig = HashMap<ConfigKey, ConfigValue>;

/// Configuration value enum to hold different types
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Bool(bool),
    Encoding(EncodingMode),
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
}

/// Create default parser configuration
pub fn default_config() -> ParserConfig {
    let mut config = HashMap::new();
    config.insert(ConfigKey::DecomposedStrings, ConfigValue::Bool(true));
    config.insert(ConfigKey::Encoding, ConfigValue::Encoding(EncodingMode::Ascii));
    config.insert(ConfigKey::AutoDetectBom, ConfigValue::Bool(false));
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
