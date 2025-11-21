use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file1> [file2] [file3] ...", args[0]);
        std::process::exit(1);
    }

    for file_path in &args[1..] {
        println!("\n{:=>80}", "=");
        println!("Analyzing: {}", file_path);
        println!("{:=>80}", "=");

        if !Path::new(file_path).exists() {
            eprintln!("✗ File not found: {}", file_path);
            continue;
        }

        let bytes = match fs::read(file_path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("✗ Error reading file: {}", e);
                continue;
            }
        };

        // Check for UTF-8 BOM
        let has_bom = bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF;
        println!(
            "UTF-8 BOM: {}",
            if has_bom {
                "✓ Present"
            } else {
                "✗ Not found"
            }
        );

        // Convert to string (handling potential UTF-8 errors)
        let content = match String::from_utf8(bytes.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("✗ Invalid UTF-8: {}", e);
                continue;
            }
        };

        // Find all non-ASCII characters
        let mut unicode_chars: Vec<(usize, usize, usize, char)> = Vec::new();
        let mut line_num = 1;
        let mut col_num = 1;
        let mut byte_pos = if has_bom { 3 } else { 0 };

        for ch in content.chars() {
            if ch == '\n' {
                line_num += 1;
                col_num = 1;
            } else {
                col_num += 1;
            }

            if !ch.is_ascii() {
                unicode_chars.push((byte_pos, line_num, col_num, ch));
            }

            byte_pos += ch.len_utf8();
        }

        println!("\nFile size: {} bytes", bytes.len());
        println!("Non-ASCII characters found: {}", unicode_chars.len());

        if unicode_chars.is_empty() {
            println!("✓ File contains only ASCII characters");
        } else {
            println!("\nNon-ASCII Unicode characters:");
            println!("{:-<80}", "");
            println!(
                "{:<8} {:<8} {:<8} {:<8} {:<20} Description",
                "Byte", "Line", "Column", "Char", "Unicode"
            );
            println!("{:-<80}", "");

            for (byte_pos, line, col, ch) in unicode_chars.iter().take(50) {
                let unicode_name = match *ch {
                    '≤' => "LESS-THAN OR EQUAL TO",
                    '≥' => "GREATER-THAN OR EQUAL TO",
                    '≠' => "NOT EQUAL TO",
                    '×' => "MULTIPLICATION SIGN",
                    '÷' => "DIVISION SIGN",
                    '°' => "DEGREE SIGN",
                    'α'..='ω' => "GREEK LETTER",
                    'Α'..='Ω' => "GREEK CAPITAL LETTER",
                    '–' => "EN DASH",
                    '—' => "EM DASH",
                    '\u{2018}' | '\u{2019}' => "SINGLE QUOTATION MARK",
                    '\u{201C}' | '\u{201D}' => "DOUBLE QUOTATION MARK",
                    _ => "OTHER",
                };

                println!(
                    "{:<8} {:<8} {:<8} {:<8} U+{:04X} ({:<6}) {}",
                    byte_pos,
                    line,
                    col,
                    ch,
                    *ch as u32,
                    format!("'{}'", ch.escape_unicode()),
                    unicode_name
                );
            }

            if unicode_chars.len() > 50 {
                println!("\n... and {} more", unicode_chars.len() - 50);
            }

            // Show context for first few occurrences
            println!("\n{:-<80}", "");
            println!("Context for first occurrences:");
            println!("{:-<80}", "");

            let lines: Vec<&str> = content.lines().collect();
            for (_, line_num, col, ch) in unicode_chars.iter().take(5) {
                if *line_num > 0 && *line_num <= lines.len() {
                    let line_idx = *line_num - 1;
                    println!("\nLine {}, Column {}:", line_num, col);

                    // Show surrounding lines for context
                    if line_idx > 0 {
                        println!("  {} | {}", line_idx, lines[line_idx - 1]);
                    }

                    let current_line = lines[line_idx];
                    println!("▶ {} | {}", line_num, current_line);

                    // Point to the character
                    let marker_pos = *col - 1;
                    let padding = format!("{} | ", line_num).len() + marker_pos;
                    println!(
                        "  {: <padding$}^--- '{}' (U+{:04X})",
                        "",
                        ch,
                        *ch as u32,
                        padding = padding
                    );

                    if line_idx + 1 < lines.len() {
                        println!("  {} | {}", line_idx + 2, lines[line_idx + 1]);
                    }
                }
            }
        }
    }

    println!("\n{:=>80}", "=");
}
