/// SAS-style ContentHandler trait for STAR file parsing
/// Returns true to stop parsing, false to continue (SAS convention)
pub trait SASContentHandler {
    // Structure callbacks
    fn start_data(&mut self, line: usize, name: &str) -> bool;
    fn end_data(&mut self, line: usize, name: &str) -> bool;
    fn start_saveframe(&mut self, line: usize, name: &str) -> bool;
    fn end_saveframe(&mut self, line: usize, name: &str) -> bool;
    fn start_loop(&mut self, line: usize) -> bool;
    fn end_loop(&mut self, line: usize) -> bool;
    fn comment(&mut self, line: usize, ext: &str) -> bool;

    // Data item callback (buffered)
    fn data(
        &mut self,
        tag: &str,
        tagline: usize,
        value: &str,
        valline: usize,
        delimiter: &str,
        inloop: bool,
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
