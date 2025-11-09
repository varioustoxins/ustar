use std::collections::HashMap;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=src/star.pest_template");
    
    let base_grammar = fs::read_to_string("src/star.pest_template")
        .expect("Failed to read base grammar file");
    
    // Generate ASCII grammar
    let ascii_patches = HashMap::from([
        ("BLANK___PLACEHOLDER", r#"{ " " | "\t" }"#),
        ("NON_BLANK_CHAR_NO_QUOTES___PLACEHOLDER", r#"{ "!" | '#'..'&' | '('..'~' }"#),
        ("NO_BLANK_CHAR___PLACEHOLDER", r#"{ '!'..'~' }"#),
        ("UTF8_BOM___PLACEHOLDER", r#"{ "\u{FEFF}" }"#),  // Support BOM detection in ASCII mode
    ]);
    generate_grammar(&base_grammar, "src/star_ascii.pest", ascii_patches);
    
    // Generate Extended ASCII grammar
    let extended_patches = HashMap::from([
        ("BLANK___PLACEHOLDER", r#"{ " " | "\t" | "\u{00A0}" }"#),  // Add non-breaking space
        ("NON_BLANK_CHAR_NO_QUOTES___PLACEHOLDER", r#"{ '\u{21}'..'\u{26}' | '\u{28}'..'\u{FF}' }"#),  // Exclude quotes at 0x27 and 0x22
        ("NO_BLANK_CHAR___PLACEHOLDER", r#"{ '\u{21}'..'\u{FF}' }"#),
        ("UTF8_BOM___PLACEHOLDER", r#"{ "\u{FEFF}" }"#),  // Support BOM detection in extended ASCII
    ]);
    generate_grammar(&base_grammar, "src/star_extended.pest", extended_patches);
    
    // Generate Unicode grammar
    let unicode_blank_rule = generate_unicode_whitespace_rule();
    let unicode_patches = HashMap::from([
        ("BLANK___PLACEHOLDER", unicode_blank_rule.as_str()),
        ("NON_BLANK_CHAR_NO_QUOTES___PLACEHOLDER", r#"{ '\u{0021}'..'\u{0021}' | '\u{0023}'..'\u{0026}' | '\u{0028}'..'\u{10FFFF}' }"#),  // All Unicode excluding quotes (0x22, 0x27)
        ("NO_BLANK_CHAR___PLACEHOLDER", r#"{ '\u{0021}'..'\u{10FFFF}' }"#),
        ("UTF8_BOM___PLACEHOLDER", r#"{ "\u{FEFF}" }"#),  // Support BOM in Unicode mode
    ]);
    generate_grammar(&base_grammar, "src/star_unicode.pest", unicode_patches);
    
    println!("Generated 3 grammar files:");
    println!("  - src/star_ascii.pest");
    println!("  - src/star_extended.pest");
    println!("  - src/star_unicode.pest");
}

fn generate_grammar(
    base: &str,
    output_path: &str,
    patches: HashMap<&str, &str>
) {
    let mut result = base.to_string();
    
    // Apply patches - replace placeholders with actual definitions
    for (placeholder, replacement) in patches {
        result = result.replace(placeholder, replacement);
    }
    
    fs::write(output_path, &result)
        .expect(&format!("Failed to write {}", output_path));
    
    println!("cargo:rerun-if-changed={}", output_path);
}

/// Generate the Unicode whitespace rule dynamically from known ranges
fn generate_unicode_whitespace_rule() -> String {
    let chars = find_unicode_whitespace_chars();
    let formatted: Vec<String> = chars
        .into_iter()
        .map(|code| format!(r#""\u{{{:04X}}}""#, code))
        .collect();
    format!("{{ {} }}", formatted.join(" | "))
}

/// Find all Unicode whitespace characters from known ranges
/// Based on benchmark_whitespace.rs
fn find_unicode_whitespace_chars() -> Vec<u32> {
    let ranges = [
        (0x0009, 0x000D),  // Tab, LF, VT, FF, CR
        (0x0020, 0x0020),  // Space
        (0x0085, 0x0085),  // Next Line (NEL)
        (0x00A0, 0x00A0),  // No-Break Space
        (0x1680, 0x1680),  // Ogham Space Mark
        (0x2000, 0x200A),  // En Quad through Hair Space
        (0x2028, 0x2029),  // Line Separator, Paragraph Separator
        (0x202F, 0x202F),  // Narrow No-Break Space
        (0x205F, 0x205F),  // Medium Mathematical Space
        (0x3000, 0x3000),  // Ideographic Space
    ];
    
    let mut chars = Vec::new();
    for (start, end) in ranges {
        for code_point in start..=end {
            if let Some(ch) = char::from_u32(code_point) {
                if ch.is_whitespace() {
                    chars.push(code_point);
                }
            }
        }
    }
    chars
}
