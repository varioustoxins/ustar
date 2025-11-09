use std::fs;
use std::collections::HashMap;
use ustar::parse;

fn main() {
    let input_file = "test_category.star";
    let content = fs::read_to_string(input_file)
        .expect("Failed to read input file");
    
    // Find the first semicolon string
    let lines: Vec<&str> = content.lines().collect();
    
    // Find start of semicolon string (line starting with ;)
    let mut semicolon_start_idx = None;
    let mut semicolon_end_idx = None;
    let mut found_first = false;
    
    for (idx, line) in lines.iter().enumerate() {
        if line.starts_with(';') {
            if !found_first {
                semicolon_start_idx = Some(idx);
                found_first = true;
            } else {
                semicolon_end_idx = Some(idx);
                break;
            }
        }
    }
    
    if semicolon_start_idx.is_none() || semicolon_end_idx.is_none() {
        eprintln!("Could not find semicolon string in file");
        return;
    }
    
    let start_idx = semicolon_start_idx.unwrap();
    let end_idx = semicolon_end_idx.unwrap();
    
    println!("Found semicolon string from line {} to line {}", start_idx + 1, end_idx + 1);
    
    // Extract the parts
    let header: Vec<&str> = lines[..start_idx].to_vec();
    let semicolon_content: Vec<&str> = lines[start_idx + 1..end_idx].to_vec();
    let footer: Vec<&str> = lines[end_idx + 1..].to_vec();
    
    println!("Semicolon content has {} lines", semicolon_content.len());
    println!("Starting search for failure point...\n");
    
    // Try progressively shorter versions of the semicolon content
    for num_lines in (0..=semicolon_content.len()).rev() {
        let mut test_content = String::new();
        
        // Add header
        for line in &header {
            test_content.push_str(line);
            test_content.push('\n');
        }
        
        // Add semicolon start
        test_content.push_str(";\n");
        
        // Add truncated semicolon content
        for i in 0..num_lines {
            test_content.push_str(semicolon_content[i]);
            test_content.push('\n');
        }
        
        // Add semicolon end
        test_content.push_str(";\n");
        
        // Add footer
        for line in &footer {
            test_content.push_str(line);
            test_content.push('\n');
        }
        
        // Try to parse
    let config = HashMap::new();
    match parse(&test_content, &config) {
            Ok(_) => {
                println!("✓ SUCCESS with {} lines in semicolon string", num_lines);
                println!("\nLast successful content:");
                println!("---");
                for i in 0..num_lines {
                    println!("{}", semicolon_content[i]);
                }
                println!("---\n");
                
                if num_lines < semicolon_content.len() {
                    println!("Next line that causes failure (line {}):", num_lines + 1);
                    println!("{}", semicolon_content[num_lines]);
                    println!("\nCharacter analysis:");
                    for (idx, ch) in semicolon_content[num_lines].chars().enumerate() {
                        println!("  [{}] '{}' (U+{:04X})", idx, ch.escape_default(), ch as u32);
                    }
                }
                break;
            }
            Err(_) => {
                if num_lines % 10 == 0 {
                    print!(".");
                }
            }
        }
    }
    println!();
}
