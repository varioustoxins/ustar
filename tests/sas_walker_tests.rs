use std::collections::HashMap;
use std::fs;
use ustar::parse_default;
use ustar::sas_interface::SASContentHandler;
use ustar::sas_walker::StarWalker;

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
        _tagline: usize,
        value: &str,
        _valline: usize,
        _delimiter: &str,
        _inloop: bool,
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
        tagline: usize,
        value: &str,
        valline: usize,
        delimiter: &str,
        inloop: bool,
    ) -> bool {
        match delimiter {
            "\n" => {
                self.output.push(format!(
                    "<data> [t:{},v:{}] {} delimiter: {:?} inloop: {} value:",
                    tagline, valline, tag, delimiter, inloop
                ));
                for line in value.lines() {
                    self.output.push(format!("       {}", line));
                }
            }
            _ => {
                self.output.push(format!(
                    "<data> [t:{},v:{}] {} delimiter: {} inloop: {} value [multiline]: {}",
                    tagline, valline, tag, delimiter, inloop, value
                ));
            }
        }
        false
    }
}

// Utility function to normalize whitespace for comparison
fn normalize_whitespace(s: &str) -> String {
    s.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_simple_data_walker_output() {
    let input = "data_test\n_tag_line_2 value1\n_tag_line_3 'quoted value'\n\n_tag_line_5 value3";

    let tree = parse_default(input).expect("Failed to parse test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let expected_output = vec![
        "<start data> [1] test",
        "<data> [t:2,v:2] _tag_line_2 delimiter:  inloop: false value [multiline]: value1",
        "<data> [t:3,v:3] _tag_line_3 delimiter: ' inloop: false value [multiline]: quoted value",
        "<data> [t:5,v:5] _tag_line_5 delimiter:  inloop: false value [multiline]: value3",
        "<end data> [5] test",
    ];

    assert_eq!(
        handler.output.len(),
        expected_output.len(),
        "Output length mismatch. Expected: {:?}, Got: {:?}",
        expected_output,
        handler.output
    );

    for (i, (expected, actual)) in expected_output
        .iter()
        .zip(handler.output.iter())
        .enumerate()
    {
        let normalized_expected = normalize_whitespace(expected);
        let normalized_actual = normalize_whitespace(actual);
        assert_eq!(
            normalized_expected, normalized_actual,
            "Line {} mismatch.\nExpected: '{}'\nActual: '{}'",
            i, expected, actual
        );
    }
}

#[test]
fn test_loop_walker_output() {
    let input = "data_test\nloop_\n_tag1\n_tag2\nval1 val2\nval3 val4";

    let tree = parse_default(input).expect("Failed to parse test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let expected_output = vec![
        "<start data> [1] test",
        "<start_loop> [2]",
        "<data> [t:3,v:5] _tag1 delimiter:  inloop: true value [multiline]: val1",
        "<data> [t:4,v:5] _tag2 delimiter:  inloop: true value [multiline]: val2",
        "<data> [t:3,v:6] _tag1 delimiter:  inloop: true value [multiline]: val3",
        "<data> [t:4,v:6] _tag2 delimiter:  inloop: true value [multiline]: val4",
        "<end_loop> [6]",
        "<end data> [6] test",
    ];

    assert_eq!(
        handler.output.len(),
        expected_output.len(),
        "Output length mismatch. Expected: {:?}, Got: {:?}",
        expected_output,
        handler.output
    );

    for (i, (expected, actual)) in expected_output
        .iter()
        .zip(handler.output.iter())
        .enumerate()
    {
        let normalized_expected = normalize_whitespace(expected);
        let normalized_actual = normalize_whitespace(actual);
        assert_eq!(
            normalized_expected, normalized_actual,
            "Line {} mismatch.\nExpected: '{}'\nActual: '{}'",
            i, expected, actual
        );
    }
}

#[test]
fn test_multiline_and_frame_codes() {
    let input = "data_test\n_frame_ref $frame1\n_multiline\n;\nThis is multiline\ntext content\n;";

    let tree = parse_default(input).expect("Failed to parse test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let expected_output = vec![
        "<start data> [1] test",
        "<data> [t:2,v:2] _frame_ref delimiter:  inloop: false value [multiline]: $frame1",
        "<data> [t:3,v:4] _multiline delimiter: \"\\n\" inloop: false value:",
        "       ;",
        "       This is multiline",
        "       text content",
        "<end data> [7] test",
    ];

    assert_eq!(
        handler.output.len(),
        expected_output.len(),
        "Output length mismatch. Expected: {:?}, Got: {:?}",
        expected_output,
        handler.output
    );

    for (i, (expected, actual)) in expected_output
        .iter()
        .zip(handler.output.iter())
        .enumerate()
    {
        let normalized_expected = normalize_whitespace(expected);
        let normalized_actual = normalize_whitespace(actual);
        assert_eq!(
            normalized_expected, normalized_actual,
            "Line {} mismatch.\nExpected: '{}'\nActual: '{}'",
            i, expected, actual
        );
    }
}

#[test]
fn test_saveframe_walker_output() {
    let input = "data_test\nsave_frame1\n_tag value\nsave_";

    let tree = parse_default(input).expect("Failed to parse test data");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, input);

    walker.walk_star_tree_buffered(&tree);

    let expected_output = vec![
        "<start data> [1] test",
        "<start saveframe> [2] frame1",
        "<data> [t:3,v:3] _tag delimiter:  inloop: false value [multiline]: value",
        "<end saveframe> [4] frame1",
        "<end data> [4] test",
    ];

    assert_eq!(
        handler.output.len(),
        expected_output.len(),
        "Output length mismatch. Expected: {:?}, Got: {:?}",
        expected_output,
        handler.output
    );

    for (i, (expected, actual)) in expected_output
        .iter()
        .zip(handler.output.iter())
        .enumerate()
    {
        let normalized_expected = normalize_whitespace(expected);
        let normalized_actual = normalize_whitespace(actual);
        assert_eq!(
            normalized_expected, normalized_actual,
            "Line {} mismatch.\nExpected: '{}'\nActual: '{}'",
            i, expected, actual
        );
    }
}
#[test]
fn test_comprehensive_example_walker_output() {
    // Read the input file from test_data
    let input = fs::read_to_string("tests/test_data/comprehensive_example.star")
        .expect("Failed to read comprehensive example file");

    // Read the expected output file
    let expected_output_text =
        fs::read_to_string("tests/test_data/comprehensive_example_walker_output.txt")
            .expect("Failed to read expected output file");

    // Parse and walk the input
    let tree = parse_default(&input).expect("Failed to parse comprehensive example");
    let mut handler = ComprehensiveTestHandler { output: Vec::new() };
    let mut walker = StarWalker::from_input(&mut handler, &input);

    walker.walk_star_tree_buffered(&tree);

    // Convert expected output text to lines
    let expected_lines: Vec<&str> = expected_output_text.lines().collect();

    // Compare lengths
    assert_eq!(
        handler.output.len(),
        expected_lines.len(),
        "Output length mismatch. Expected {} lines, got {} lines.\nFirst few actual lines: {:?}",
        expected_lines.len(),
        handler.output.len(),
        handler.output.iter().take(10).collect::<Vec<_>>()
    );

    // Compare each line with normalization
    // should really be in a utility function
    for (i, (expected, actual)) in expected_lines.iter().zip(handler.output.iter()).enumerate() {
        let normalized_expected = normalize_whitespace(expected);
        let normalized_actual = normalize_whitespace(actual);

        if normalized_expected != normalized_actual {
            println!("Mismatch at line {}", i);
            println!("Expected: '{}'", expected);
            println!("Actual:   '{}'", actual);
            println!("Expected normalized: '{}'", normalized_expected);
            println!("Actual normalized:   '{}'", normalized_actual);
        }

        assert_eq!(
            normalized_expected, normalized_actual,
            "Line {} mismatch.\nExpected: '{}'\nActual: '{}'",
            i, expected, actual
        );
    }
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
