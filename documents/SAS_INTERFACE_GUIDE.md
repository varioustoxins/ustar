# SAS Interface Guide

The SAS[^SAS] (STAR-based API for Streaming) interface provides a SAX[^SAX] style streaming API for parsing STAR[^STAR] format files as implimented in the uStar[^uSTAR-URL] STAR parser written in Rust[^Rust]. This document covers the APIs core concepts and callback methods, along with special handling for nested and empty loops. 

[^SAS]: SAS (STAR-based API for Streaming) - A streaming interface for parsing STAR format files, adapted from the BMRB SAS API design. https://github.com/bmrb-io/SAS.

[^SAX]: Simple API for XML - An event-driven, sequential access parser API for XML documents. Originally developed by David Megginson and the XML-DEV mailing list in 1998. Documented at http://www.saxproject.org. Also see: Chiou, Y.W. (2002). APIs for XML: DOM, SAX, and JDOM. In: Harindranath, G., et al. New Perspectives on Information Systems Development. Springer, Boston, MA. https://doi.org/10.1007/978-1-4615-0595-2_24

[^STAR]: Self-defining Text Archive and Retrieval (STAR) file format. Hall, S.R. (1991). The STAR File: A New Format for Electronic Data Transfer and Archiving. DOI: 10.1021/ci00002a020

[^uSTAR-URL]: uStar STAR parser - A Rust implementation of a STAR format parser with multi-encoding support. Available at: https://github.com/variablenix/ustar

[^Rust]: Rust programming language - A systems programming language focused on safety and performance. For comprehensive analysis see: Bugden, W. & Alahmar, A. (2022). Rust: The Programming Language for Safety and Performance. DOI: 10.48550/arXiv.2206.05503

> This API is an adaption and extension of the SAS API[^BMRB-SAS] devised by Dimitri Maziuk and provided by the BMRB[^BMRB]. An appendix at the end of this manual discusses the difference between the Python API and the Rust API.
>
> The STAR file is a format used by a number of well known chemistry and biochemistry file formats specifially  NMR-Star, NEF, CIF and mmCIF. Most of these file formats use derivatives [typically subsets] of the complete STAR language. This interface currently provides support for all these formats and the complete STAR format excluding associative dictionaries which are currently not in production use.

[^BMRB-SAS]: BMRB SAS API - The original SAS (Simple API for STAR) implementation provided by the Biological Magnetic Resonance Bank for processing NMR-STAR files.

[^BMRB]: Biological Magnetic Resonance Data Bank - An open access repository of NMR spectroscopic data. Ulrich, E.L. et al. (2023). Biological Magnetic Resonance Data Bank. Nucleic Acids Research. DOI: 10.1093/nar/gkac1050
 
## Overview

The SAS interface uses the **Content Handler** pattern common in XML SAX parsers and is a **Streaming Interface**. Instead of building a complete parse tree in memory, your handler receives callbacks as each
element is encountered during parsing. This enables:

- **Low memory usage**: Process large files without loading everything into memory
- **Early termination**: Stop parsing when you've found what you need
- **Custom processing**: Build your own data structures or perform calculations on-the-fly

## Core Components

### SASContentHandler Trait

The cen tral component of the SAS parser is the SASContentHandler trait which is defined below.

```rust
pub trait SASContentHandler {
    // Stream callbacks - return true to stop parsing, false to continue
    fn start_stream(&mut self, name: Option<&str>) -> bool;
    fn end_stream(&mut self, position: LineColumn) -> bool;
    fn start_global(&mut self, position: LineColumn) -> bool;
    fn end_global(&mut self, position: LineColumn) -> bool;
    fn start_data(&mut self, position: LineColumn, name: &str) -> bool;
    fn end_data(&mut self, position: LineColumn, name: &str) -> bool;
    fn start_saveframe(&mut self, position: LineColumn, name: &str) -> bool;
    fn end_saveframe(&mut self, position: LineColumn, name: &str) -> bool;
    fn start_loop(&mut self, position: LineColumn) -> bool;
    fn end_loop(&mut self, position: LineColumn) -> bool;
    fn comment(&mut self, position: LineColumn, text: &str) -> bool;

    // Data item callback
    fn data(
        &mut self,
        tag: &str,                        // The tag name (e.g., "_atom_site_label")
        tag_position: LineColumn,
        value: &str,                      // The value
        value_position: LineColumn,
        delimiter: &str,                  // "" [none],  ', ", ;, or "EMPTY_LOOP"
        loop_level: usize,                // 0 = not in loop, 1 in a loop >1 in a nested loop
    ) -> bool;
}
```

This is based on the type 1 content handler `ContentHandler` from the Python implimentation with a small number of changes.

it should be noted that references aen't directly annotated  and should be identified by the user of the SASContentHandler

### StarWalker

The `StarWalker` traverses a parsed STAR tree and invokes your handler:

```rust
use ustar::parse_default;
use ustar::sas_walker::StarWalker;
use ustar::sas_interface::SASContentHandler;

let tree = parse_default(star_content)?;
let mut handler = MyHandler::new();
let mut walker = StarWalker::from_input(&mut handler, star_content);
walker.walk_star_tree_buffered(&tree);
```

## Understanding Delimiters

The `delimiter` parameter in the `data()` callback indicates how the value was quoted:

| Delimiter      | Meaning              | Example              |
|----------------|----------------------|----------------------|
| `` (empty)     | Unquoted value       | `_tag value`         |
| `'`            | Single-quoted string | `_tag 'hello world'` |
| `"`            | Double-quoted string | `_tag "hello world"` |
| `;`            | Semicolon multi-line | `_tag \n; ... \n;`   |
| `"EMPTY_LOOP"` | Tag in empty loop    | See below            |

## Loop Levels

The `loop_level` parameter indicates nesting depth:

- `0` = Not in a loop (regular data item)
- `1` = Outermost loop
- `2` = First nested loop
- `3+` = Deeper nesting

## Examples

### Example 1: Simple Data Items

**STAR Input:**
```star
data_example
    _simple_tag    value1
    _quoted_tag    'value with spaces'
    _multiline
;
Line 1
Line 2
;
```

**Handler Callbacks:**
```
start_data(position: 1:1, name: "example")
  data(tag: "_simple_tag", value: "value1", delimiter: "", loop_level: 0)
  data(tag: "_quoted_tag", value: "value with spaces", delimiter: "'", loop_level: 0)
  data(tag: "_multiline", value: "Line 1\nLine 2\n", delimiter: ";", loop_level: 0)
end_data(position: 8:1, name: "example")
```

### Example 2: Simple Loop

**STAR Input:**
```star
data_atoms
loop_
    _atom_label
    _atom_symbol
    _atom_x
    C1  C  1.234
    N1  N  2.345
stop_
```

**Handler Callbacks:**
```
start_data(position: 1:1, name: "atoms")
  start_loop(position: 2:1)
    data(tag: "_atom_label", value: "C1", delimiter: "", loop_level: 1)
    data(tag: "_atom_symbol", value: "C", delimiter: "", loop_level: 1)
    data(tag: "_atom_x", value: "1.234", delimiter: "", loop_level: 1)
    data(tag: "_atom_label", value: "N1", delimiter: "", loop_level: 1)
    data(tag: "_atom_symbol", value: "N", delimiter: "", loop_level: 1)
    data(tag: "_atom_x", value: "2.345", delimiter: "", loop_level: 1)
  end_loop(position: 8:1)
end_data(position: 8:1, name: "atoms")
```

### Example 3: Nested Loop

Nested loops allow variable-length inner data for each outer row.

**STAR Input:**
```star
data_bonds
loop_
    _mol_id
    _mol_name
    loop_
        _bond_atom1
        _bond_atom2
        _bond_order
    stop_
    MOL1 'Molecule One'
        C1 C2 single
        C2 C3 double
    stop_
    MOL2 'Molecule Two'  
        N1 N2 single
    stop_
stop_
```

**Handler Callbacks:**
```
start_data(position: 1:1, name: "bonds")
  start_loop(position: 2:1)
    # First outer row
    data(tag: "_mol_id", value: "MOL1", delimiter: "", loop_level: 1)
    data(tag: "_mol_name", value: "Molecule One", delimiter: "'", loop_level: 1)
    # Inner loop values for first row
    data(tag: "_bond_atom1", value: "C1", delimiter: "", loop_level: 2)
    data(tag: "_bond_atom2", value: "C2", delimiter: "", loop_level: 2)
    data(tag: "_bond_order", value: "single", delimiter: "", loop_level: 2)
    data(tag: "_bond_atom1", value: "C2", delimiter: "", loop_level: 2)
    data(tag: "_bond_atom2", value: "C3", delimiter: "", loop_level: 2)
    data(tag: "_bond_order", value: "double", delimiter: "", loop_level: 2)
    # Second outer row
    data(tag: "_mol_id", value: "MOL2", delimiter: "", loop_level: 1)
    data(tag: "_mol_name", value: "Molecule Two", delimiter: "'", loop_level: 1)
    # Inner loop values for second row
    data(tag: "_bond_atom1", value: "N1", delimiter: "", loop_level: 2)
    data(tag: "_bond_atom2", value: "N2", delimiter: "", loop_level: 2)
    data(tag: "_bond_order", value: "single", delimiter: "", loop_level: 2)
  end_loop(position: 18:1)
end_data(position: 18:1, name: "bonds")
```

### Example 4: Empty Loop

An empty loop has tags defined but no data values.

**STAR Input:**
```star
data_placeholder
loop_
    _planned_tag1
    _planned_tag2
stop_
```

**Handler Callbacks:**
```
start_data(position: 1:1, name: "placeholder")
  start_loop(position: 2:1)
    data(tag: "_planned_tag1", value: "", delimiter: "EMPTY_LOOP", loop_level: 1)
    data(tag: "_planned_tag2", value: "", delimiter: "EMPTY_LOOP", loop_level: 1)
  end_loop(position: 5:1)
end_data(position: 5:1, name: "placeholder")
```

The `EMPTY_LOOP` delimiter signals that the value is empty because the loop itself
was empty, not because the value was literally an empty string.

### Example 5: Nested Loop with Empty Inner

When only outer loop has data, inner tags get `EMPTY_LOOP`.

**STAR Input:**
```star
data_partial
loop_
    _outer_tag
    loop_
        _inner_tag
    stop_
    outer_value1 stop_
    outer_value2 stop_
stop_
```

**Handler Callbacks:**
```
start_data(position: 1:1, name: "partial")
  start_loop(position: 2:1)
    data(tag: "_outer_tag", value: "outer_value1", delimiter: "", loop_level: 1)
    data(tag: "_outer_tag", value: "outer_value2", delimiter: "", loop_level: 1)
    # Inner loop was never filled - emit EMPTY_LOOP for its tags
    data(tag: "_inner_tag", value: "", delimiter: "EMPTY_LOOP", loop_level: 2)
  end_loop(position: 9:1)
end_data(position: 9:1, name: "partial")
```

### Example 6: Save Frames

Save frames are named containers within a data block.

**STAR Input:**
```star
data_experiment
save_sample_1
    _sample_name  'Test Sample'
    _sample_ph    7.4
save_

save_sample_2
    _sample_name  'Control'
    _sample_ph    7.0
save_
```

**Handler Callbacks:**
```
start_data(position: 1:1, name: "experiment")
  start_saveframe(position: 2:1, name: "sample_1")
    data(tag: "_sample_name", value: "Test Sample", delimiter: "'", loop_level: 0)
    data(tag: "_sample_ph", value: "7.4", delimiter: "", loop_level: 0)
  end_saveframe(position: 5:1, name: "sample_1")
  start_saveframe(position: 7:1, name: "sample_2")
    data(tag: "_sample_name", value: "Control", delimiter: "'", loop_level: 0)
    data(tag: "_sample_ph", value: "7.0", delimiter: "", loop_level: 0)
  end_saveframe(position: 10:1, name: "sample_2")
end_data(position: 10:1, name: "experiment")
```

## Implementing a Handler

Here's a minimal handler that collects all data items:

```rust
use ustar::line_column_index::LineColumn;
use ustar::sas_interface::{SASContentHandler, EMPTY_LOOP_DELIMITER};

struct DataCollector {
    items: Vec<(String, String)>,
}

impl SASContentHandler for DataCollector {
    fn start_stream(&mut self, _name: Option<&str>) -> bool { false }
    fn end_stream(&mut self, _position: LineColumn) -> bool { false }
    fn start_data(&mut self, _position: LineColumn, _name: &str) -> bool { false }
    fn end_data(&mut self, _position: LineColumn, _name: &str) -> bool { false }
    fn start_saveframe(&mut self, _position: LineColumn, _name: &str) -> bool { false }
    fn end_saveframe(&mut self, _position: LineColumn, _name: &str) -> bool { false }
    fn start_loop(&mut self, _position: LineColumn) -> bool { false }
    fn end_loop(&mut self, _position: LineColumn) -> bool { false }
    fn comment(&mut self, _position: LineColumn, text: &str) -> bool { false }

    fn data(
        &mut self,
        tag: &str,
        _tag_position: LineColumn,
        value: &str,
        _value_position: LineColumn,
        delimiter: &str,
        _loop_level: usize,
    ) -> bool {
        // Skip empty loop markers
        if delimiter != EMPTY_LOOP_DELIMITER {
            self.items.push((tag.to_string(), value.to_string()));
        }
        false // continue parsing
    }
}
```

## Early Termination

Return `true` from any callback to stop parsing immediately:

```rust
fn data(
    &mut self,
    tag: &str,
    _tag_position: LineColumn,
    value: &str,
    _value_position: LineColumn,
    _delimiter: &str,
    _loop_level: usize,
) -> bool {
    if tag == "_target_tag" {
        self.found_value = Some(value.to_string());
        return true; // Stop parsing - we found what we need
    }
    false
}
```

## Best Practices

1. **Handle EMPTY_LOOP**: Always check for `EMPTY_LOOP_DELIMITER` when processing
   loop data, especially if you're counting rows or validating data.

2. **Use loop_level**: Track which loop level you're in to properly associate
   nested data with its parent row.

3. **Track context**: Maintain state in your handler (current data block, saveframe,
   loop) to correctly interpret each callback.

4. **Position information**: Use `LineColumn` positions for error reporting or
   building source maps.

5. **Early termination**: For search operations, return `true` as soon as you
   find what you need to avoid parsing the entire file.

## Constants

```rust
use ustar::sas_interface::EMPTY_LOOP_DELIMITER;

// Value: "EMPTY_LOOP"
// Used as delimiter for tags in loops with no data values
```

## Appendix 1 The Python SAS interface


## References

