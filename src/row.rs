use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

use crate::{Position, SearchDirection};

#[derive(Default)]
pub struct Row {
    string: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(content: &str) -> Self {
        Self {
            string: String::from(content),
            len: content.graphemes(true).count(),
        }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        #[allow(clippy::integer_arithmetic)]
        for grapheme in self.string[..]
                .graphemes(true)
                .skip(start)
                .take(end-start) {
            if grapheme == "\t" {
                result.push_str(" ");
            } else {
                result.push_str(grapheme);
            }
        }
        
        result
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
            return;
        } 

        let mut result = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }

        self.len = length;
        self.string = result;
    }

    #[allow(clippy::integer_arithmetic)]
    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return; // do nothing
        }

        let mut result = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index != at {
                length += 1;
                result.push_str(grapheme);
            }
        }

        self.len = length;
        self.string = result;
    }

    pub fn append(&mut self, string_to_add: Row) {
        self.string = format!("{}{}", self.string, string_to_add.string);
        self.len = self.string.len();
    }

    pub fn split(&mut self, at: usize) -> Self {
        let mut row = String::new();
        let mut splitted_str = String::new();
        let mut length = 0;
        let mut split_len = 0;

        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if (index < at) {
                length += 1;
                row.push_str(grapheme);
            } else {
                split_len += 1;
                splitted_str.push_str(grapheme);
            }
        }

        self.string = row;
        self.len = length;

        Self {
            string: splitted_str,
            len: split_len,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        return self.string.as_bytes()
    }

    pub fn find(&self, query: &str, at: usize, search_direction: SearchDirection) -> Option<usize> {
        if at > self.len {
            return None;
        }
        
        let start = if search_direction == SearchDirection::Forward {
            at
        } else {
            0
        };

        let end = if search_direction == SearchDirection::Forward {
            self.len
        } else {
            at 
        };
        #[allow(clippy::integer_arithmetic)]
        let substring:String = self.string[..].graphemes(true).skip(start).take(end-start).collect();
        let str_index = if search_direction == SearchDirection::Forward {
            substring.find(query)
        } else {
            substring.rfind(query)
        };

        // str_index does not necessarily give us the query idx, since many characters
        // take more than one len, we need to check graphemes as well.
        if let Some(str_index) = str_index {
            for (grapheme_idx, (byte_idx, _)) in substring[..].grapheme_indices(true).enumerate() {
                if str_index == byte_idx {
                    #[allow(clippy::integer_arithmetic)]
                    return Some(start + grapheme_idx);
                }
            }
        }
        None
    }
}