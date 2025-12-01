//! Fast line/column lookup index for STAR file parsing
//!
//! This module provides efficient O(log n) line and column lookups from byte offsets
//! by precomputing line start positions once and using binary search with optimizations.

/// Sentinel value for undefined line number (since line numbers are 1-based)
const UNDEFINED_LINE: usize = 0;

/// Sentinel value for undefined column number (since column numbers are 1-based)
const UNDEFINED_COLUMN: usize = 0;

/// A fast index for converting byte offsets to line and column numbers
#[derive(Debug, Clone)]
pub struct LineColumnIndex {
    /// Byte offsets where each line starts (including 0 for line 1)
    line_starts: Vec<usize>,
    /// Total input length for bounds checking
    input_len: usize,
}

impl LineColumnIndex {
    /// Create a new LineColumnIndex by scanning the input once
    pub fn new(input: &str) -> Self {
        let mut line_starts = Vec::with_capacity(input.len() / 50); // Estimate ~50 chars per line
        line_starts.push(0);

        let input_bytes = input.as_bytes();
        let mut pos = 0;

        while let Some(newline_pos) = memchr::memchr(b'\n', &input_bytes[pos..]) {
            pos += newline_pos + 1;
            line_starts.push(pos);
        }

        Self {
            line_starts,
            input_len: input.len(),
        }
    }

    /// Convert a byte offset to (line, column) coordinates (1-based)
    pub fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        if offset > self.input_len {
            // Handle out-of-bounds gracefully
            return (self.line_starts.len(), 1);
        }

        // Binary search to find the line
        match self.line_starts.binary_search(&offset) {
            Ok(line_idx) => {
                // Exact match - this offset is at the start of a line
                (line_idx + 1, 1)
            }
            Err(line_idx) => {
                // offset falls between line_starts[line_idx-1] and line_starts[line_idx]
                if line_idx == 0 {
                    // Before first line (shouldn't happen with valid input)
                    (UNDEFINED_LINE, UNDEFINED_COLUMN)
                } else {
                    let line_number = line_idx; // 1-based line number
                    let line_start = self.line_starts[line_idx - 1];
                    let column = offset - line_start + 1; // 1-based column
                    (line_number, column)
                }
            }
        }
    }
}
