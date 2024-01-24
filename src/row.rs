use std::cmp;
use unicode_segmentation::UnicodeSegmentation;
use termion::color;
use crate::{Position, SearchDirection};
use crate::highlighting;

#[derive(Default)]
pub struct Row {
    string: String,
    highlighting: Vec<highlighting::Type>,
    len: usize,
}

impl From<&str> for Row {
    fn from(content: &str) -> Self {
        Self {
            string: String::from(content),
            highlighting: Vec::new(),
            len: content.graphemes(true).count(),
        }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut current_highlighting = &highlighting::Type::None;
        let mut result = String::new();
        #[allow(clippy::integer_arithmetic)]
        for (index, grapheme) in self.string[..]
                .graphemes(true)
                .enumerate()
                .skip(start)
                .take(end-start) {
                        if let Some(c) = grapheme.chars().next() {
                        let highlighting_type = self.highlighting.get(index).unwrap_or( &highlighting::Type::None);

                        if current_highlighting != highlighting_type {
                            current_highlighting = highlighting_type;
                            let highlight_start = format!("{}", color::Fg(highlighting_type.to_color()));
                            result.push_str(&highlight_start[..]);
                        }
                    
                        if c == '\t' {
                            result.push_str(" ");
                        } else {
                            result.push(c);
                        }
                    }
                }
        let highlight_end = format!("{}", color::Fg(color::Reset));
        result.push_str(&highlight_end[..]);
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
            highlighting: Vec::new(),
            len: split_len,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        return self.string.as_bytes()
    }

    pub fn find(&self, query: &str, at: usize, search_direction: SearchDirection) -> Option<usize> {
        if at > self.len || query.is_empty(){
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

    pub fn highlight(&mut self, word: Option<&str>) {
        let mut highlighting = Vec::new();
        let chars: Vec<char> = self.string.chars().collect();
        let mut matches = Vec::new();
        let mut search_index = 0;

        if let Some(word) = word {
            while let Some(match_index) = self.find(word, search_index, SearchDirection::Forward) {
                matches.push(match_index);
                if let Some(next_index) = match_index.checked_add(word[..].graphemes(true).count()) {
                    search_index = next_index;
                } else {
                    break; // eol
                }
            }
        }

        let mut index = 0;
        let mut is_prev_separator = true;
        while let Some(c) = chars.get(index) {
            if let Some(word) = word {
                if matches.contains(&index) {
                    for _ in word[..].graphemes(true) {
                        index += 1;
                        highlighting.push(highlighting::Type::Match);
                    }
                    continue;
                }
            }

            let prev_sep_highlight = if index > 0 {
                #[allow(clippy::integer_arithmetic)]
                highlighting.get(index - 1).unwrap_or(&highlighting::Type::None)
            } else {
                &highlighting::Type::None
            };

            if (c.is_ascii_digit() 
            && (prev_sep_highlight == &highlighting::Type::Number || is_prev_separator))
            || (c == &'.' && prev_sep_highlight == &highlighting::Type::Number) {
                highlighting.push(highlighting::Type::Number);
            } else {
                highlighting.push(highlighting::Type::None);
            }

            is_prev_separator = c.is_ascii_whitespace() || c.is_ascii_punctuation();
            index += 1;
        }
        
        self.highlighting = highlighting;
    }
}