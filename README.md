# USTAR

A Rust parser for STAR (Self-defining Text Archive and Retrieval) format files, including NEF, CIF, mmCIF, and NMR-STAR.

## Features

- **Multi-encoding support**: ASCII, Extended ASCII, Unicode
- **Multiple STAR formats**: CIF, mmCIF, NMR-STAR, NEF
- **Error handling**: Rich error diagnostics with miette integration
- **Grammar generation**: Dynamic parser generation for different character sets
- **Real-world tested**: Validated against databases (PDB, COD, BMRB, NEF, multiple mmcif dictionaries)

## Usage

```toml
[dependencies]
ustar = "0.1"
```

```rust
use ustar::parsers::ascii::{AsciiParser, Rule};
use pest::Parser;

let content = std::fs::read_to_string("example.cif")?;
let pairs = AsciiParser::parse(Rule::star_file, &content)?;
```

## License

LGPL3
