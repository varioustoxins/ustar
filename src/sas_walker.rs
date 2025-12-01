use crate::line_column_index::LineColumnIndex;
use crate::mutable_pair::MutablePair;
use crate::sas_interface::SASContentHandler;

/// Walks a MutablePair parse tree and calls the BufferedContentHandler methods.
pub struct StarWalker<'a, T: SASContentHandler> {
    line_index: LineColumnIndex, // Fast line/column index (always present)
    pub tag_table: Vec<Vec<String>>,
    pub tag_level: usize,
    pub tag_index: usize,
    pub tag_lines: Vec<Vec<usize>>,
    pub in_loop: bool,
    pub handler: &'a mut T,
}

impl<'a, T: SASContentHandler> StarWalker<'a, T> {
    /// Decrement tag_level if possible, and reset tag_index to zero
    pub fn decrement_tag_pointers(&mut self) {
        if self.tag_level > 0 {
            self.tag_level -= 1;
        }
        self.tag_index = 0;
    }
    /// Increment tag_index, and if needed, tag_level, according to tag_table structure
    pub fn increment_tag_pointers(&mut self) {
        self.tag_index += 1;
        let row_len = self.tag_table[self.tag_level].len();
        if self.tag_index >= row_len {
            if self.tag_level + 1 < self.tag_table.len() {
                self.tag_level += 1;
                self.tag_index = 0;
            } else {
                self.tag_index = 0;
            }
        }
    }
    pub fn from_input(handler: &'a mut T, input: &str) -> Self {
        let line_index = LineColumnIndex::new(input);
        StarWalker {
            line_index,
            tag_table: Vec::new(),
            tag_level: 0,
            tag_index: 0,
            tag_lines: Vec::new(),
            in_loop: false,
            handler,
        }
    }

    /// Get line number for a byte offset (private)
    fn get_line_number(&self, offset: usize) -> usize {
        self.line_index.offset_to_line_col(offset).0
    }

    pub fn walk_star_tree_buffered(&mut self, node: &MutablePair) -> bool {
        let mut should_stop = false;
        match node.rule_name.as_str() {
            "data" => {
                for child in &node.children {
                    should_stop = self.walk_star_tree_buffered(child);
                    if should_stop {
                        break;
                    }
                }
                if !self.in_loop {
                    self.tag_table.clear();
                    self.tag_lines.clear();
                }
            }
            "semi_colon_string" | "double_quote_string" | "single_quote_string" => {
                if node.children.len() < 2 {
                    return false;
                }
                let value_node = &node.children[1];
                let tag = self.tag_table[self.tag_level][self.tag_index].as_str();
                let tagline = self.tag_lines[self.tag_level][self.tag_index];
                let valline = self.get_line_number(value_node.start);
                let children = &value_node.children;
                if children.len() == 3 {
                    let delimiter = &children[0].content;
                    let value = &children[1].content;
                    should_stop =
                        self.handler
                            .data(tag, tagline, value, valline, delimiter, self.in_loop);
                } else {
                    let delimiter = &node.content[0..1];
                    let value = if delimiter == ";" {
                        &node.content[2..node.content.len() - 2]
                    } else {
                        &node.content[1..node.content.len() - 1]
                    };
                    should_stop =
                        self.handler
                            .data(tag, tagline, value, valline, delimiter, self.in_loop);
                }

                self.increment_tag_pointers();
            }
            // TODO: would it be better to make a non_quoted_string decompose to un_quoted_string->string for consistency
            "non_quoted_string" | "string" => {
                let tag = self.tag_table[self.tag_level][self.tag_index].as_str();
                let tagline = self.tag_lines[self.tag_level][self.tag_index];
                let valline = self.get_line_number(node.start);
                let value = node.content.as_str();
                should_stop = self
                    .handler
                    .data(tag, tagline, value, valline, "", self.in_loop);
                self.increment_tag_pointers();
            }
            "frame_code" => {
                let tag = self.tag_table[self.tag_level][self.tag_index].as_str();
                let tagline = self.tag_lines[self.tag_level][self.tag_index];
                let value = node.content.as_str();
                let valline = self.get_line_number(node.start);
                should_stop = self
                    .handler
                    .data(tag, tagline, value, valline, "", self.in_loop);
                self.increment_tag_pointers();
            }
            "stop_keyword" => {
                self.decrement_tag_pointers();
            }

            "loop_keyword" => {
                // Each time a loop keyword is seen, add an empty tag list for this loop level
                self.tag_table.push(Vec::new());
                self.tag_lines.push(Vec::new());
            }

            "data_loop" => {
                should_stop = self.handler.start_loop(self.get_line_number(node.start));

                if !should_stop {
                    self.in_loop = true;
                    for child in &node.children {
                        should_stop = self.walk_star_tree_buffered(child);
                        if should_stop {
                            break;
                        }
                    }
                    self.in_loop = false;

                    if !should_stop {
                        should_stop = self.handler.end_loop(self.get_line_number(node.end));
                    }
                }

                self.tag_table.clear();
                self.tag_lines.clear();
                self.tag_level = 0;
                self.tag_index = 0;
            }

            "data_name" => {
                let line_number = self.get_line_number(node.start);
                if self.in_loop {
                    let last = self.tag_table.len() - 1;
                    self.tag_table[last].push(node.content.to_string());
                    self.tag_lines[last].push(line_number);
                } else {
                    self.tag_table.push(vec![node.content.to_string()]);
                    self.tag_lines.push(vec![line_number]);
                }
            }
            "data_block" => {
                let data_heading = &node.children[0];
                let data_name = &data_heading.content[5..];
                should_stop = self
                    .handler
                    .start_data(self.get_line_number(node.start), data_name);

                if !should_stop {
                    for child in &node.children[1..] {
                        should_stop = self.walk_star_tree_buffered(child);
                        if should_stop {
                            break;
                        }
                    }
                }

                if !should_stop {
                    should_stop = self
                        .handler
                        .end_data(self.get_line_number(node.end), data_name);
                }
            }
            "save_frame" => {
                let save_heading = &node.children[0];
                let frame_name = &save_heading.content[5..];
                should_stop = self
                    .handler
                    .start_saveframe(self.get_line_number(node.start), frame_name);

                if !should_stop {
                    for child in &node.children[1..] {
                        should_stop = self.walk_star_tree_buffered(child);
                        if should_stop {
                            break;
                        }
                    }
                }

                if !should_stop {
                    should_stop = self
                        .handler
                        .end_saveframe(self.get_line_number(node.end), frame_name);
                }
            }
            "comment" => {
                should_stop = self
                    .handler
                    .comment(self.get_line_number(node.start), &node.content);
            }
            _ => {
                for child in &node.children {
                    should_stop = self.walk_star_tree_buffered(child);
                    if should_stop {
                        break;
                    }
                }
            }
        }
        should_stop
    }
}
