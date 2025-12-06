use crate::line_column_index::{LineColumn, LineColumnIndex};
use crate::mutable_pair::MutablePair;
use crate::sas_interface::SASContentHandler;

/// Walks a MutablePair parse tree and calls the BufferedContentHandler methods.
pub struct StarWalker<'a, T: SASContentHandler> {
    line_index: LineColumnIndex, // Fast line/column index (always present)
    pub tag_table: Vec<Vec<String>>,
    pub tag_level: usize,
    pub tag_index: usize,
    pub tag_positions: Vec<Vec<LineColumn>>,
    pub loop_level: usize, // 0 = not in loop, 1+ = loop nesting level
    pub handler: &'a mut T,
}

impl<'a, T: SASContentHandler> StarWalker<'a, T> {
    /// Decrement tag_level if possible, and reset tag_index to zero.
    pub fn decrement_tag_pointers(&mut self) {
        if self.tag_level > 0 {
            self.tag_level -= 1;
        }
        self.tag_index = 0;
    }
    /// Increment tag_index, and if needed, tag_level, according to tag_table structure.
    pub fn increment_tag_pointers(&mut self) {
        self.tag_index += 1;
        let row_len = self.tag_table[self.tag_level].len();
        if self.tag_index >= row_len {
            if self.tag_level + 1 < self.tag_table.len() {
                self.tag_level += 1;
            }
            self.tag_index = 0;
        }
    }

    /// Get the current loop level for data callbacks.
    /// Returns tag_level + 1 when inside a loop (so level 1 = outermost loop tags,
    /// level 2 = first nested loop tags, etc.), or 0 when not in a loop.
    fn current_loop_level(&self) -> usize {
        if self.loop_level > 0 {
            self.tag_level + 1
        } else {
            0
        }
    }

    pub fn from_input(handler: &'a mut T, input: &str) -> Self {
        let line_index = LineColumnIndex::new(input);
        StarWalker {
            line_index,
            tag_table: Vec::new(),
            tag_level: 0,
            tag_index: 0,
            tag_positions: Vec::new(),
            loop_level: 0,
            handler,
        }
    }

    /// Get line number for a byte offset (private)
    fn get_line_number(&self, offset: usize) -> usize {
        self.line_index.offset_to_line_col(offset).line
    }

    /// Get line and column for a byte offset (private)
    fn get_line_column(&self, offset: usize) -> LineColumn {
        self.line_index.offset_to_line_col(offset)
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
                if self.loop_level == 0 {
                    self.tag_table.clear();
                    self.tag_positions.clear();
                }
            }
            "semi_colon_string" | "double_quote_string" | "single_quote_string" => {
                if node.children.len() < 2 {
                    return false;
                }
                let value_node = &node.children[1];
                let tag = self.tag_table[self.tag_level][self.tag_index].as_str();
                let tag_position = self.tag_positions[self.tag_level][self.tag_index];
                let value_position = self.get_line_column(value_node.start);
                let children = &value_node.children;
                if children.len() == 3 {
                    let delimiter = &children[0].content;
                    let value = &children[1].content;
                    should_stop = self.handler.data(
                        tag,
                        tag_position,
                        value,
                        value_position,
                        delimiter,
                        self.current_loop_level(),
                    );
                } else {
                    let delimiter = &node.content[0..1];
                    let value = if delimiter == ";" {
                        &node.content[2..node.content.len() - 2]
                    } else {
                        &node.content[1..node.content.len() - 1]
                    };
                    should_stop = self.handler.data(
                        tag,
                        tag_position,
                        value,
                        value_position,
                        delimiter,
                        self.current_loop_level(),
                    );
                }

                self.increment_tag_pointers();
            }
            // TODO: would it be better to make a non_quoted_string decompose to un_quoted_string->string for consistency
            "non_quoted_string" | "string" => {
                let tag = self.tag_table[self.tag_level][self.tag_index].as_str();
                let tag_position = self.tag_positions[self.tag_level][self.tag_index];
                let value_position = self.get_line_column(node.start);
                let value = node.content.as_str();
                should_stop = self.handler.data(
                    tag,
                    tag_position,
                    value,
                    value_position,
                    "",
                    self.current_loop_level(),
                );
                self.increment_tag_pointers();
            }
            "frame_code" => {
                let tag = self.tag_table[self.tag_level][self.tag_index].as_str();
                let tag_position = self.tag_positions[self.tag_level][self.tag_index];
                let value = node.content.as_str();
                let value_position = self.get_line_column(node.start);
                should_stop = self.handler.data(
                    tag,
                    tag_position,
                    value,
                    value_position,
                    "",
                    self.current_loop_level(),
                );
                self.increment_tag_pointers();
            }
            "stop_keyword" => {
                self.decrement_tag_pointers();
            }

            "loop_keyword" => {
                // Each time a loop keyword is seen, add an empty tag list for this loop level
                self.tag_table.push(Vec::new());
                self.tag_positions.push(Vec::new());
            }

            "data_loop" => {
                should_stop = self.handler.start_loop(self.get_line_number(node.start));

                if !should_stop {
                    self.loop_level = 1; // Enter first loop level
                    for child in &node.children {
                        should_stop = self.walk_star_tree_buffered(child);
                        if should_stop {
                            break;
                        }
                    }
                    self.loop_level = 0; // Exit loop

                    if !should_stop {
                        should_stop = self.handler.end_loop(self.get_line_number(node.end));
                    }
                }

                self.tag_table.clear();
                self.tag_positions.clear();
                self.tag_level = 0;
                self.tag_index = 0;
            }

            "data_name" => {
                let tag_position = self.get_line_column(node.start);
                if self.loop_level > 0 {
                    let last = self.tag_table.len() - 1;
                    self.tag_table[last].push(node.content.to_string());
                    self.tag_positions[last].push(tag_position);
                } else {
                    self.tag_table.push(vec![node.content.to_string()]);
                    self.tag_positions.push(vec![tag_position]);
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
