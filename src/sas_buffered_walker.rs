

use crate::mutable_pair::MutablePair;
use crate::sas_buffered::BufferedContentHandler;

/// Walks a MutablePair parse tree and calls the BufferedContentHandler methods.
pub struct StarWalker<'a, T: BufferedContentHandler> {
    pub tag_table: Vec<Vec<String>>,
    pub tag_level: usize,
    pub tag_index: usize,
    pub in_loop: bool,
    pub handler: &'a mut T,
}

impl<'a, T: BufferedContentHandler> StarWalker<'a, T> {
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
    pub fn new(handler: &'a mut T) -> Self {
        StarWalker {
            tag_table: Vec::new(),
            tag_level: 0,
            tag_index: 0,
            in_loop: false,
            handler,
        }
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
                }
            }
            "semi_colon_string" | "double_quote_string" | "single_quote_string" => {
                let tag_node = &node.children[0];
                let value_node = &node.children[1];
                let tag =  self.tag_table[self.tag_level][self.tag_index].as_str();
                let tagline = tag_node.start;
                let valline = value_node.start;
                let children = &value_node.children;
                if children.len() == 3 {
                    let delimiter = &children[0].content;
                    let value = &children[1].content;
                    should_stop =  self.handler.data(tag, tagline, value, valline, delimiter, self.in_loop);
                } else {
                    let delimiter = &node.content[0..1];
                    let value = if delimiter == ";" {
                        &node.content[2..node.content.len() - 2]
                    } else {
                        &node.content[1..node.content.len() - 1]
                    };
                    should_stop = self.handler.data(tag, tagline, value , valline, delimiter, self.in_loop);
                }

                self.increment_tag_pointers();
                
            }
            // TODO: would it be better to make a non_quoted_string decompose to un_quoted_string->string for consistency 
            "non_quoted_string" | "string" => {
                let tag =  self.tag_table[self.tag_level][self.tag_index].as_str();
                let tagline = node.start;
                let valline = node.start;
                let value = node.content.as_str();
                should_stop = self.handler.data(tag, tagline, value, valline, "", self.in_loop);
                self.increment_tag_pointers();
            }
            "frame_code" => {
                let value = node.content.as_str();
                let valline = node.start;
                should_stop = self.handler.data("", 0, value, valline, "", self.in_loop);
                self.increment_tag_pointers();
            }
            "stop_keyword" => {
                self.decrement_tag_pointers();
            }

            "loop_keyword" => {
                // Each time a loop keyword is seen, add an empty tag list for this loop level
                self.tag_table.push(Vec::new());
            }

            "data_loop" => {
                self.handler.start_loop(node.start);
                self.in_loop = true;
                for child in &node.children {
                    should_stop = self.walk_star_tree_buffered(child);
                    if should_stop {
                        break;
                    }
                }
                self.in_loop = false;
                self.handler.end_loop(node.end);
                self.tag_table.clear();
                self.tag_level = 0;
                self.tag_index = 0;
                
            }

            "data_name" => {
                if self.in_loop {
                    let last = self.tag_table.len() - 1;
                    self.tag_table[last].push(node.content[1..].to_string());
                } else {
                    self.tag_table.push(vec![node.content[1..].to_string()]);
                }
            }
            "data_block" => {
                let data_heading = &node.children[0];
                let data_name = &data_heading.content[5..];
                should_stop = self.handler.start_data(node.start, data_name);
                for child in &node.children[1..] {
                    should_stop = self.walk_star_tree_buffered(child);
                    if should_stop {
                        break;
                    }
                }
                if !should_stop {
                    should_stop = self.handler.end_data(node.end, data_name);
                }
            }
            "save_frame" => {
                let save_heading = &node.children[0];
                let frame_name = &save_heading.content[5..];
                should_stop = self.handler.start_saveframe(node.start, frame_name);
                for child in &node.children[1..] {
                    should_stop = self.walk_star_tree_buffered(child);
                    if should_stop {
                        break;
                    }
                }
                if !should_stop{
                    should_stop = self.handler.end_saveframe(node.end, frame_name);
                }
            }
            "comment" => {
                should_stop = self.handler.comment(node.start, &node.content);
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
