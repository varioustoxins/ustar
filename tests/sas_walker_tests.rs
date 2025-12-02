use insta::assert_snapshot;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use ustar::line_column_index::LineColumn;
use ustar::parse_default;
use ustar::sas_interface::SASContentHandler;
use ustar::sas_walker::StarWalker;

// Files that are known to fail parsing (or have special handling needs)
static KNOWN_PARSE_FAILURES: &[&str] = &[
    "loop3.str",
    "loop4.str",
    "loop5.str",
    "warning.cif",
    "warning.str",
];

// Test input constants for early termination tests
const BASIC_INPUT: &str = "
data_test
    _item1 value1
    _item2 value2
";

const SAVEFRAME_INPUT: &str = "
data_test
    save_frame1
        _item1 value1
    save_
    _after value
";

const LOOP_INPUT: &str = "
data_test
    loop_
        _tag1
        _tag2
        value1 value2
    stop_
    _after value
";

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum ElementToStopOn {
    StartData(usize),
    EndData(usize),
    StartSaveframe(usize),
    EndSaveframe(usize),
    StartLoop(usize),
    EndLoop(usize),
    Data(usize),
}

impl ElementToStopOn {
    fn get_count(&self) -> usize {
        match self {
            ElementToStopOn::StartData(n) => *n,
            ElementToStopOn::EndData(n) => *n,
            ElementToStopOn::StartSaveframe(n) => *n,
            ElementToStopOn::EndSaveframe(n) => *n,
            ElementToStopOn::StartLoop(n) => *n,
            ElementToStopOn::EndLoop(n) => *n,
            ElementToStopOn::Data(n) => *n,
        }
    }

    fn element_type(&self) -> ElementType {
        match self {
            ElementToStopOn::StartData(_) => ElementType::StartData,
            ElementToStopOn::EndData(_) => ElementType::EndData,
            ElementToStopOn::StartSaveframe(_) => ElementType::StartSaveframe,
            ElementToStopOn::EndSaveframe(_) => ElementType::EndSaveframe,
            ElementToStopOn::StartLoop(_) => ElementType::StartLoop,
            ElementToStopOn::EndLoop(_) => ElementType::EndLoop,
            ElementToStopOn::Data(_) => ElementType::Data,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
enum ElementType {
    StartData,
    EndData,
    StartSaveframe,
    EndSaveframe,
    StartLoop,
    EndLoop,
    Data,
}

struct ParameterizedHandler {
    stop_on: ElementToStopOn,
    events: Vec<String>,
    element_counts: HashMap<ElementType, usize>,
}

impl ParameterizedHandler {
    fn new(stop_on: ElementToStopOn) -> Self {
        Self {
            stop_on,
            events: Vec::new(),
            element_counts: HashMap::new(),
        }
    }

    fn increment_and_check(&mut self, element_type: ElementType) -> bool {
        let count = self.element_counts.entry(element_type).or_insert(0);
        *count += 1;

        if self.stop_on.element_type() == element_type {
            *count >= self.stop_on.get_count()
        } else {
            false
        }
    }
}

impl SASContentHandler for ParameterizedHandler {
    fn start_data(&mut self, _line: usize, name: &str) -> bool {
        self.events.push(format!("start_data({})", name));
        self.increment_and_check(ElementType::StartData)
    }

    fn end_data(&mut self, _line: usize, name: &str) -> bool {
        self.events.push(format!("end_data({})", name));
        self.increment_and_check(ElementType::EndData)
    }

    fn start_saveframe(&mut self, _line: usize, name: &str) -> bool {
        self.events.push(format!("start_saveframe({})", name));
        self.increment_and_check(ElementType::StartSaveframe)
    }

    fn end_saveframe(&mut self, _line: usize, name: &str) -> bool {
        self.events.push(format!("end_saveframe({})", name));
        self.increment_and_check(ElementType::EndSaveframe)
    }

    fn start_loop(&mut self, _line: usize) -> bool {
        self.events.push("start_loop".to_string());
        self.increment_and_check(ElementType::StartLoop)
    }

    fn end_loop(&mut self, _line: usize) -> bool {
        self.events.push("end_loop".to_string());
        self.increment_and_check(ElementType::EndLoop)
    }

    fn comment(&mut self, _line: usize, text: &str) -> bool {
        self.events.push(format!("comment({})", text));
        false
    }

    fn data(
        &mut self,
        tag: &str,
        _tag_position: LineColumn,
        value: &str,
        _value_position: LineColumn,
        _delimiter: &str,
        _loop_level: usize,
    ) -> bool {
        self.events.push(format!("data({}, {})", tag, value));
        self.increment_and_check(ElementType::Data)
    }
}

fn test_early_termination(stop_on: ElementToStopOn, input: &str, expected: &[&str]) {
    let tree = parse_default(input).expect("Failed to parse");
    let mut handler = ParameterizedHandler::new(stop_on.clone());
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let expected_vec: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        handler.events, expected_vec,
        "Early termination test failed for {:?}\nExpected: {:?}\nGot: {:?}",
        stop_on, expected, handler.events
    );
}

struct ComprehensiveTestHandler {
    output: Vec<String>,
}

impl SASContentHandler for ComprehensiveTestHandler {
    fn start_data(&mut self, line: usize, name: &str) -> bool {
        self.output
            .push(format!("<start data> [{}] {}", line, name));
        false
    }

    fn end_data(&mut self, line: usize, name: &str) -> bool {
        self.output.push(format!("<end data> [{}] {}", line, name));
        false
    }

    fn start_saveframe(&mut self, line: usize, name: &str) -> bool {
        self.output
            .push(format!("<start saveframe> [{}] {}", line, name));
        false
    }

    fn end_saveframe(&mut self, line: usize, name: &str) -> bool {
        self.output
            .push(format!("<end saveframe> [{}] {}", line, name));
        false
    }

    fn start_loop(&mut self, line: usize) -> bool {
        self.output.push(format!("<start_loop> [{}]", line));
        false
    }

    fn end_loop(&mut self, line: usize) -> bool {
        self.output.push(format!("<end_loop> [{}]", line));
        false
    }

    fn comment(&mut self, line: usize, text: &str) -> bool {
        self.output.push(format!("# [{}] {}", line, text));
        false
    }

    fn data(
        &mut self,
        tag: &str,
        tag_position: LineColumn,
        value: &str,
        value_position: LineColumn,
        delimiter: &str,
        loop_level: usize,
    ) -> bool {
        match delimiter {
            "\n" => {
                self.output.push(format!(
                    "<data> [t:{}:{},v:{}:{}] {} delimiter: {:?} loop_level: {} value:",
                    tag_position.line,
                    tag_position.column,
                    value_position.line,
                    value_position.column,
                    tag,
                    delimiter,
                    loop_level
                ));
                for line in value.lines() {
                    self.output.push(format!("       {}", line));
                }
            }
            _ => {
                self.output.push(format!(
                    "<data> [t:{}:{},v:{}:{}] {} delimiter: {} loop_level: {} value [multiline]: {}",
                    tag_position.line, tag_position.column, value_position.line, value_position.column, tag, delimiter, loop_level, value
                ));
            }
        }
        false
    }
}

#[test]
fn test_simple_data_walker_output() {
    let input = "data_test\n_tag_line_2 value1\n_tag_line_3 'quoted value'\n\n_tag_line_5 value3";

    let tree = parse_default(input).expect("Failed to parse test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    assert_snapshot!(output);
}

#[test]
fn test_loop_walker_output() {
    let input = "data_test\nloop_\n_tag1\n_tag2\nval1 val2\nval3 val4";

    let tree = parse_default(input).expect("Failed to parse test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    assert_snapshot!(output);
}

#[test]
fn test_multiline_and_frame_codes() {
    let input = "data_test\n_frame_ref $frame1\n_multiline\n;\nThis is multiline\ntext content\n;";

    let tree = parse_default(input).expect("Failed to parse test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    assert_snapshot!(output);
}

#[test]
fn test_saveframe_walker_output() {
    let input = "data_test\nsave_frame1\n_tag value\nsave_";

    let tree = parse_default(input).expect("Failed to parse test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    assert_snapshot!(output);
}
#[test]
fn test_comprehensive_example_walker_output() {
    // Read the input file from test_data
    let input = fs::read_to_string("tests/test_data/comprehensive_example.star")
        .expect("Failed to read comprehensive example file");

    // Parse and walk the input
    let tree = parse_default(&input).expect("Failed to parse comprehensive example");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, &input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    assert_snapshot!(output);
}

#[test]
fn test_early_termination_all_methods() {
    // 1. start_data - should stop immediately (after 1st occurrence)
    test_early_termination(
        ElementToStopOn::StartData(1),
        BASIC_INPUT,
        &["start_data(test)"],
    );

    // 2. end_data - should process all data then stop (after 1st end_data)
    test_early_termination(
        ElementToStopOn::EndData(1),
        BASIC_INPUT,
        &[
            "start_data(test)",
            "data(_item1, value1)",
            "data(_item2, value2)",
            "end_data(test)",
        ],
    );

    // 3. start_saveframe - should stop at saveframe start (after 1st occurrence)
    test_early_termination(
        ElementToStopOn::StartSaveframe(1),
        SAVEFRAME_INPUT,
        &["start_data(test)", "start_saveframe(frame1)"],
    );

    // 4. end_saveframe - should process saveframe then stop (after 1st end_saveframe)
    test_early_termination(
        ElementToStopOn::EndSaveframe(1),
        SAVEFRAME_INPUT,
        &[
            "start_data(test)",
            "start_saveframe(frame1)",
            "data(_item1, value1)",
            "end_saveframe(frame1)",
        ],
    );

    // 5. start_loop - should stop at loop start (after 1st occurrence)
    test_early_termination(
        ElementToStopOn::StartLoop(1),
        LOOP_INPUT,
        &["start_data(test)", "start_loop"],
    );

    // 6. end_loop - should process loop data then stop (after 1st end_loop)
    test_early_termination(
        ElementToStopOn::EndLoop(1),
        LOOP_INPUT,
        &[
            "start_data(test)",
            "start_loop",
            "data(_tag1, value1)",
            "data(_tag2, value2)",
            "end_loop",
        ],
    );

    // 7. data after N - should stop after N data items
    test_early_termination(
        ElementToStopOn::Data(2),
        BASIC_INPUT,
        &[
            "start_data(test)",
            "data(_item1, value1)",
            "data(_item2, value2)",
        ],
    );

    // 8. Test stopping after 1st data item (demonstrating default of 1)
    test_early_termination(
        ElementToStopOn::Data(1),
        BASIC_INPUT,
        &["start_data(test)", "data(_item1, value1)"],
    );
}

// ============================================================================
// Snapshot tests for SAS test files
// ============================================================================

/// Generate a snapshot test for each file in the sas_test_files directory
/// that can be successfully parsed
#[test]
fn test_sas_test_files_walker_output() {
    let dir = Path::new("tests/test_data/sas_test_files");
    assert!(
        dir.exists() && dir.is_dir(),
        "Directory {:?} does not exist",
        dir
    );

    for entry in fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("entry failed");
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "str" || ext == "cif" || ext == "dic" {
                let filename = path.file_name().unwrap().to_string_lossy().to_string();

                // Skip known parse failures
                if KNOWN_PARSE_FAILURES.contains(&filename.as_str()) {
                    continue;
                }

                let data =
                    fs::read(&path).unwrap_or_else(|_| panic!("Failed to read file {:?}", path));
                let content = String::from_utf8_lossy(&data).to_string();

                // Parse and walk the file
                let tree = parse_default(&content)
                    .unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", path, e));
                let mut handler = ComprehensiveTestHandler { output: Vec::new() };
                let mut walker = StarWalker::from_input(&mut handler, &content);

                walker.walk_star_tree_buffered(&tree);

                let output = handler.output.join("\n");

                // Use insta's dynamic snapshot naming
                insta::with_settings!({
                    snapshot_suffix => filename.replace('.', "_"),
                }, {
                    assert_snapshot!(output);
                });
            }
        }
    }
}
