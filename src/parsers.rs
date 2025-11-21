// Each parser needs to be in its own module to avoid Rule enum conflicts
pub mod ascii {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "star_ascii.pest"]
    pub struct AsciiParser;
}

pub mod extended {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "star_extended.pest"]
    pub struct ExtendedParser;
}

pub mod unicode {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "star_unicode.pest"]
    pub struct UnicodeParser;
}

// Re-export the parsers at the top level for convenience
pub use ascii::AsciiParser;
pub use extended::ExtendedParser;
pub use unicode::UnicodeParser;

// All three parsers generate the same Rule enum structure
// Export Rule from the ascii module (they're all compatible)
pub use ascii::Rule;
