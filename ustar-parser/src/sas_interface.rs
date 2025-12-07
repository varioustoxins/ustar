use crate::line_column_index::LineColumn;

/// Delimiter used to indicate an empty loop (no data values)
/// When a loop has tags defined but no data values, each tag is emitted
/// with this delimiter, an empty value "", and position for the values (-1, -1)
pub const EMPTY_LOOP_DELIMITER: &str = "EMPTY_LOOP";

/// SAS-style ContentHandler trait for STAR file parsing
/// Returns true to stop parsing, false to continue (SAS convention)
pub trait SASContentHandler {
    // Stream callbacks
    fn start_stream(&mut self, name: Option<&str>) -> bool;
    fn end_stream(&mut self, position: LineColumn) -> bool;

    // Structure callbacks
    fn start_global(&mut self, position: LineColumn) -> bool;
    fn end_global(&mut self, position: LineColumn) -> bool;
    fn start_data(&mut self, position: LineColumn, name: &str) -> bool;
    fn end_data(&mut self, position: LineColumn, name: &str) -> bool;
    fn start_saveframe(&mut self, position: LineColumn, name: &str) -> bool;
    fn end_saveframe(&mut self, position: LineColumn, name: &str) -> bool;
    fn start_loop(&mut self, position: LineColumn) -> bool;
    fn end_loop(&mut self, position: LineColumn) -> bool;
    fn comment(&mut self, position: LineColumn, text: &str) -> bool;

    // Data item callback (buffered)
    fn data(
        &mut self,
        tag: &str,
        tag_position: LineColumn,
        value: &str,
        value_position: LineColumn,
        delimiter: &str,
        loop_level: usize,
    ) -> bool;
}

// Example skeleton for a parse-tree walker function
// (Assumes you have a MutablePair or similar parse tree node)
//
// pub fn walk_star_tree_buffered<T: BufferedContentHandler>(
//     node: &MutablePair,
//     handler: &mut T,
// ) {
//     // Traverse the tree and call handler methods as appropriate
// }
