use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

use crate::Position;

#[derive(Default)]
pub struct Row {
    string: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(content: &str) -> Self {
        let mut row = Self {
            string: String::from(content),
            len: 0,
        };

        row.update_length();
        row
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
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

    pub fn update_length(&mut self)  {
        self.len = self.string[..].graphemes(true).count()
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at > self.len() {
            self.string.push(c);
        } else {
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            let remaining_str: String = self.string[..].graphemes(true).skip(at).collect();
            result.push(c);
            result.push_str(&remaining_str);
            self.string = result;    
        }

        self.update_length();
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return; // do nothing
        } else {
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            let remaining_str: String = self.string[..].graphemes(true).skip(at + 1).collect();
            result.push_str(&remaining_str);
            self.string = result;
        }

        self.update_length();
    }

    pub fn append(&mut self, string_to_add: Row) {
        self.string = format!("{}{}", self.string, string_to_add.string);
        self.update_length();
    }

    pub fn split(&mut self, at: usize) -> Self {
        let first_split: String = self.string[..].graphemes(true).take(at).collect();
        let second_split: String = self.string[..].graphemes(true).skip(at).collect();
        self.string = first_split;
        self.update_length();
        Self::from(&second_split[..])
    }

    pub fn as_bytes(&self) -> &[u8] {
        return self.string.as_bytes()
    }
}