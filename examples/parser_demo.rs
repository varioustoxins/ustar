use std::collections::HashMap;
use ustar::dump_extractors::dump_mutable_pair;

fn main() {
    let input = r#"
data_test
    _double "hello world"
    _single 'test string'
    _non_quoted simple_value
    _semicolon
;
Multi-line content
with embedded text
;
"#;

    println!("=== USTAR Parser Demo ===\n");
    println!("Input:");
    println!("{}", input);
    println!();

    println!("\n=== Dump of Parsed Input ===\n");
    let config_decompose = HashMap::new(); // Empty config defaults to true

    match ustar::parse(input, &config_decompose) {
        Ok(result) => {
            dump_mutable_pair(&result, 0);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
        }
    }
}
