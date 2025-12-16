use std::collections::HashMap;
use std::fs;
use std::path::Path;
use ustar::line_column_index::LineColumn;
use ustar::parse_default;
use ustar::sas_interface::{SASContentHandler, EMPTY_LOOP_DELIMITER};
use ustar::sas_walker::StarWalker;

mod snapshot_utils;

// Files that are known to fail parsing (or have special handling needs)
static KNOWN_PARSE_FAILURES: &[&str] = &[
    "loop3.str",   // loop with no header we should fail this
    "loop4.str",   // loop with no body, should we parse this
    "loop5.str",   // has triple quoted strings
    "warning.cif", // has triple quoted strings again and tests an error state...
    "warning.str", // uses triple quoted strings and tests an error state [runaway string]P
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

const GLOBAL_INPUT: &str = "
global_
    _global_setting 'test_value'
    _global_version 1.0

data_test
    _local_item value1
";

const GLOBAL_WITH_LOOP_INPUT: &str = "
global_
    _global_setting 'test_value'
    loop_
        _config_key
        _config_value
        'database_host' 'localhost'
        'database_port' '5432'
    stop_
    _global_version 1.0

data_experiment
    _experiment_name 'test_exp'
";

const GLOBAL_WITH_NESTED_INPUT: &str = "
global_
    _global_setting 'production'
    loop_
        _server_name
        _server_type
        loop_
            _port
            _protocol
        stop_
        'web-server' 'nginx'
            '80' 'http'
            '443' 'https'
        stop_
        'db-server' 'postgresql'
            '5432' 'tcp'
        stop_
    stop_

data_application
    _app_name 'web_app'
";

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum ElementToStopOn {
    StartStream(usize),
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
            ElementToStopOn::StartStream(n) => *n,
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
            ElementToStopOn::StartStream(_) => ElementType::StartStream,
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
    StartStream,
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
    fn start_stream(&mut self, _name: Option<&str>) -> bool {
        self.events.push("start_stream".to_string());
        self.increment_and_check(ElementType::StartStream)
    }

    fn end_stream(&mut self, _position: LineColumn) -> bool {
        self.events.push("end_stream".to_string());
        false
    }

    fn start_global(&mut self, _position: LineColumn) -> bool {
        self.events.push("start_global".to_string());
        false
    }

    fn end_global(&mut self, _position: LineColumn) -> bool {
        self.events.push("end_global".to_string());
        false
    }

    fn start_data(&mut self, _position: LineColumn, name: &str) -> bool {
        self.events.push(format!("start_data({})", name));
        self.increment_and_check(ElementType::StartData)
    }

    fn end_data(&mut self, _position: LineColumn, name: &str) -> bool {
        self.events.push(format!("end_data({})", name));
        self.increment_and_check(ElementType::EndData)
    }

    fn start_saveframe(&mut self, _position: LineColumn, name: &str) -> bool {
        self.events.push(format!("start_saveframe({})", name));
        self.increment_and_check(ElementType::StartSaveframe)
    }

    fn end_saveframe(&mut self, _position: LineColumn, name: &str) -> bool {
        self.events.push(format!("end_saveframe({})", name));
        self.increment_and_check(ElementType::EndSaveframe)
    }

    fn start_loop(&mut self, _position: LineColumn) -> bool {
        self.events.push("start_loop".to_string());
        self.increment_and_check(ElementType::StartLoop)
    }

    fn end_loop(&mut self, _position: LineColumn) -> bool {
        self.events.push("end_loop".to_string());
        self.increment_and_check(ElementType::EndLoop)
    }

    fn comment(&mut self, _position: LineColumn, text: &str) -> bool {
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
    fn start_stream(&mut self, _name: Option<&str>) -> bool {
        self.output.push("<start_stream>".to_string());
        false
    }

    fn end_stream(&mut self, _position: LineColumn) -> bool {
        self.output.push("<end_stream>".to_string());
        false
    }

    fn start_global(&mut self, position: LineColumn) -> bool {
        self.output
            .push(format!("<start global> [{}]", position.line));
        false
    }

    fn end_global(&mut self, position: LineColumn) -> bool {
        self.output
            .push(format!("<end global> [{}]", position.line));
        false
    }

    fn start_data(&mut self, position: LineColumn, name: &str) -> bool {
        self.output
            .push(format!("<start data> [{}] {}", position.line, name));
        false
    }

    fn end_data(&mut self, position: LineColumn, name: &str) -> bool {
        self.output
            .push(format!("<end data> [{}] {}", position.line, name));
        false
    }

    fn start_saveframe(&mut self, position: LineColumn, name: &str) -> bool {
        self.output
            .push(format!("<start saveframe> [{}] {}", position.line, name));
        false
    }

    fn end_saveframe(&mut self, position: LineColumn, name: &str) -> bool {
        self.output
            .push(format!("<end saveframe> [{}] {}", position.line, name));
        false
    }

    fn start_loop(&mut self, position: LineColumn) -> bool {
        self.output
            .push(format!("<start_loop> [{}]", position.line));
        false
    }

    fn end_loop(&mut self, position: LineColumn) -> bool {
        self.output.push(format!("<end_loop> [{}]", position.line));
        false
    }

    fn comment(&mut self, position: LineColumn, text: &str) -> bool {
        self.output.push(format!("# [{}] {}", position.line, text));
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
            EMPTY_LOOP_DELIMITER => {
                // Empty loop - no value position, special format
                self.output.push(format!(
                    "<data> [t:{}:{}] {} loop_level: {} [empty-loop]",
                    tag_position.line, tag_position.column, tag, loop_level
                ));
            }
            ";" => {
                // Semicolon-bounded string - multi-line
                self.output.push(format!(
                    "<data> [t:{}:{},v:{}:{}] {} delimiter: ; loop_level: {} [multi-line] value: {}",
                    tag_position.line,
                    tag_position.column,
                    value_position.line,
                    value_position.column,
                    tag,
                    loop_level,
                    value
                ));
            }
            "" => {
                // No delimiter (non-quoted string, frame code)
                self.output.push(format!(
                    "<data> [t:{}:{},v:{}:{}] {} delimiter: none loop_level: {} value: {}",
                    tag_position.line,
                    tag_position.column,
                    value_position.line,
                    value_position.column,
                    tag,
                    loop_level,
                    value
                ));
            }
            _ => {
                // Quote delimiters: "'", "\""
                self.output.push(format!(
                    "<data> [t:{}:{},v:{}:{}] {} delimiter: {} loop_level: {} value: {}",
                    tag_position.line,
                    tag_position.column,
                    value_position.line,
                    value_position.column,
                    tag,
                    delimiter,
                    loop_level,
                    value
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
    snapshot_utils::assert_snapshot_gz("sas_walker_tests__simple_data_walker_output", &output);
}

#[test]
fn test_loop_walker_output() {
    let input = "data_test\nloop_\n_tag1\n_tag2\nval1 val2\nval3 val4";

    let tree = parse_default(input).expect("Failed to parse test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    snapshot_utils::assert_snapshot_gz("sas_walker_tests__loop_walker_output", &output);
}

#[test]
fn test_multiline_and_frame_codes() {
    let input = "data_test\n_frame_ref $frame1\n_multiline\n;\nThis is multiline\ntext content\n;";

    let tree = parse_default(input).expect("Failed to parse test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    snapshot_utils::assert_snapshot_gz("sas_walker_tests__multiline_and_frame_codes", &output);
}

#[test]
fn test_saveframe_walker_output() {
    let input = "data_test\nsave_frame1\n_tag value\nsave_";

    let tree = parse_default(input).expect("Failed to parse test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    snapshot_utils::assert_snapshot_gz("sas_walker_tests__saveframe_walker_output", &output);
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
    snapshot_utils::assert_snapshot_gz(
        "sas_walker_tests__comprehensive_example_walker_output",
        &output,
    );
}

/// Test nested loop entry/exit events
///
/// # Input Format (STAR nested loop definition)
///
/// ```star
/// loop_                                  # definition & start of data
///     _atom_identity_node                # definition  
///     _atom_identity_symbol              # definition
///     loop_                              # definition
///         _atom_bond_node_1              # definition
///         _atom_bond_node_2              # definition
///         _atom_bond_order               # definition
/// A1 B1 1 2 single stop_                 # data
/// A2 B2 1 6 double 30 40 triple stop_    # data
/// A3 B3 1 7 single stop_                 # end of data
/// ```
///
/// # Expected Output (pseudo-code showing level transitions)
///
/// ```text
///                                             #0
/// start_loop                                  #1
/// data tag: atom_identity_node  value: A1     #1
/// data tag: atom_identity_symbol value: B1    #1
/// data tag: atom_bond_node_1 value: 1         #2
/// data tag: atom_bond_node_2 value: 2         #2       
/// data tag: atom_bond_order value: single     #2
/// data tag: atom_identity_node  value: A2     #1
/// data tag: atom_identity_symbol value: B2    #1
/// data tag: atom_bond_node_1 value: 1         #2
/// data tag: atom_bond_node_2 value: 6         #2
/// data tag: atom_bond_order value: double     #2
/// data tag: atom_bond_node_1 value: 30        #2
/// data tag: atom_bond_node_2 value: 40        #2
/// data tag: atom_bond_order value: triple     #2
/// data tag: atom_identity_node  value: A3     #1
/// data tag: atom_identity_symbol value: B3    #1
/// data tag: atom_bond_node_1 value: 1         #2
/// data tag: atom_bond_node_2 value: 7         #2
/// data tag: atom_bond_order value: single     #2
/// end_loop                                    #0
/// ```
///
/// Note: `start_loop` is emitted when transitioning to a nested loop level,
/// and `end_loop` is emitted when `stop_` exits a nested level OR when the
/// outer loop completes.
#[test]
fn test_nested_loop_walker_output() {
    let input = indoc::indoc!(
        r#"
        data_test
        loop_
            _atom_identity_node
            _atom_identity_symbol
            loop_
                _atom_bond_node_1
                _atom_bond_node_2
                _atom_bond_order
        A1 B1 1 2 single stop_
        A2 B2 1 6 double 30 40 triple stop_
        A3 B3 1 7 single stop_
        stop_
    "#
    );

    let tree = parse_default(input).expect("Failed to parse nested loop test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    snapshot_utils::assert_snapshot_gz("sas_walker_tests__nested_loop_walker_output", &output);
}

/// Test empty loop with explicit stop_ keyword
/// A loop with tags but no data values, terminated by stop_
/// This is valid syntax: the stop_ indicates the loop has zero rows of data
#[test]
fn test_empty_loop_with_stop() {
    let input = indoc::indoc!(
        r#"
        data_test
            loop_
                _tag1
                _tag2
                _tag3
            stop_
        _after_loop value
    "#
    );

    let tree = parse_default(input).expect("Failed to parse empty loop test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    snapshot_utils::assert_snapshot_gz("sas_walker_tests__empty_loop_with_stop", &output);
}

/// Test empty loop followed by save_ (which terminates the loop)
/// Note: Without stop_, STAR parsers treat subsequent values as loop data
/// So we need an explicit terminator like save_ or another data_
#[test]
fn test_empty_loop_before_saveframe() {
    let input = indoc::indoc!(
        r#"
        data_test
            loop_
                _tag1
                _tag2
            stop_
        save_frame1
            _inside_frame value
        save_
    "#
    );

    let tree = parse_default(input).expect("Failed to parse empty loop test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    snapshot_utils::assert_snapshot_gz("sas_walker_tests__empty_loop_before_saveframe", &output);
}

/// Test nested empty loop - outer loop has values but inner nested loop is empty
/// The inner loop has no values between the outer values and the stop_
#[test]
fn test_nested_empty_loop() {
    let input = indoc::indoc!(
        r#"
        data_test
            loop_
                _outer_tag1
                _outer_tag2
                loop_
                    _inner_tag1
                    _inner_tag2
                    A B stop_
                    C D stop_
            stop_
        "#
    );

    let tree = parse_default(input).expect("Failed to parse nested empty loop test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    snapshot_utils::assert_snapshot_gz("sas_walker_tests__nested_empty_loop", &output);
}

#[test]
fn test_early_termination_all_methods() {
    // 0. start_stream - should stop immediately (after 1st occurrence)
    test_early_termination(
        ElementToStopOn::StartStream(1),
        BASIC_INPUT,
        &["start_stream"],
    );

    // 1. start_data - should stop immediately (after 1st occurrence)
    test_early_termination(
        ElementToStopOn::StartData(1),
        BASIC_INPUT,
        &["start_stream", "start_data(test)"],
    );

    // 2. end_data - should process all data then stop (after 1st end_data)
    test_early_termination(
        ElementToStopOn::EndData(1),
        BASIC_INPUT,
        &[
            "start_stream",
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
        &[
            "start_stream",
            "start_data(test)",
            "start_saveframe(frame1)",
        ],
    );

    // 4. end_saveframe - should process saveframe then stop (after 1st end_saveframe)
    test_early_termination(
        ElementToStopOn::EndSaveframe(1),
        SAVEFRAME_INPUT,
        &[
            "start_stream",
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
        &["start_stream", "start_data(test)", "start_loop"],
    );

    // 6. end_loop - should process loop data then stop (after 1st end_loop)
    test_early_termination(
        ElementToStopOn::EndLoop(1),
        LOOP_INPUT,
        &[
            "start_stream",
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
            "start_stream",
            "start_data(test)",
            "data(_item1, value1)",
            "data(_item2, value2)",
        ],
    );

    // 8. Test stopping after 1st data item (demonstrating default of 1)
    test_early_termination(
        ElementToStopOn::Data(1),
        BASIC_INPUT,
        &["start_stream", "start_data(test)", "data(_item1, value1)"],
    );
}

// ============================================================================
// Snapshot tests for SAS test files
// ============================================================================

/// Generate a snapshot test for each file in the sas_test_files directory
/// that can be successfully parsed.
/// Uses check_snapshot_gz to collect all failures before panicking,
/// so all .snap.new files are generated in a single test run.
#[test]
fn test_sas_test_files_walker_output() {
    let dir = Path::new("tests/test_data/sas_test_files");
    assert!(
        dir.exists() && dir.is_dir(),
        "Directory {:?} does not exist",
        dir
    );

    let mut failures: Vec<snapshot_utils::SnapshotMismatch> = Vec::new();

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

                // Use check_snapshot_gz to collect failures without panicking
                let snapshot_name = format!(
                    "sas_walker_tests__sas_test_files_walker_output@{}",
                    filename.replace('.', "_")
                );
                if let Err(mismatch) = snapshot_utils::check_snapshot_gz(&snapshot_name, &output) {
                    failures.push(mismatch);
                }
            }
        }
    }

    // After processing all files, panic if there were any failures
    if !failures.is_empty() {
        let failure_summary: Vec<String> = failures.iter().map(|f| f.to_string()).collect();
        panic!(
            "{} snapshot(s) failed:\n\n{}\n\nRun ./scripts/insta-accept.sh to accept all new snapshots.",
            failures.len(),
            failure_summary.join("\n\n")
        );
    }
}

#[test]
fn test_global_block_walker_output() {
    let tree = parse_default(GLOBAL_INPUT).expect("Failed to parse global input");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, GLOBAL_INPUT);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    snapshot_utils::assert_snapshot_gz("sas_walker_tests__global_block_walker_output", &output);
}

#[test]
fn test_global_with_loop_walker_output() {
    let tree =
        parse_default(GLOBAL_WITH_LOOP_INPUT).expect("Failed to parse global with loop input");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, GLOBAL_WITH_LOOP_INPUT);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    snapshot_utils::assert_snapshot_gz("sas_walker_tests__global_with_loop_walker_output", &output);
}

#[test]
fn test_global_with_nested_walker_output() {
    let tree =
        parse_default(GLOBAL_WITH_NESTED_INPUT).expect("Failed to parse global with nested input");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, GLOBAL_WITH_NESTED_INPUT);

    walker.walk_star_tree_buffered(&tree);

    let output = handler.output.join("\n");
    snapshot_utils::assert_snapshot_gz(
        "sas_walker_tests__global_with_nested_walker_output",
        &output,
    );
}
