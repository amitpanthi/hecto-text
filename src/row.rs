use std::cmp;
use unicode_segmentation::UnicodeSegmentation;
use termion::color;
use crate::filetype::HighlightingOptions;
use crate::{Position, SearchDirection};
use crate::highlighting;

#[derive(Default)]
pub struct Row {
    string: String,
    highlighting: Vec<highlighting::Type>,
    pub is_highlighted: bool,
    len: usize,
}

impl From<&str> for Row {
    fn from(content: &str) -> Self {
        Self {
            string: String::from(content),
            highlighting: Vec::new(),
            is_highlighted: false,
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
            if index < at {
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
            is_highlighted: false,
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

    fn highlight_match(&mut self, word: &Option<String>) {
        if let Some(word) = word {
            if word.is_empty() {
                return;
            }
        }

        let mut search_index = 0;
        if let Some(word) = word {
            while let Some(match_index) = self.find(word, search_index, SearchDirection::Forward) {
                if let Some(next_index) = match_index.checked_add(word[..].graphemes(true).count()) {
                    #[allow(clippy::indexing_slicing)]
                    for i in search_index..next_index {
                        self.highlighting[i] = highlighting::Type::Match;
                    }
                    search_index = next_index;
                } else {
                    break; // eol
                }
            }
        }
    }

    fn highlight_char(&mut self, index: &mut usize, hl_opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if hl_opts.chars() && c == '\'' {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                let closing_index = if *next_char == '\\' {
                    index.saturating_add(3) // '\a'
                } else {
                    index.saturating_add(2) // 'a'
                };

                if let Some(closing_char) = chars.get(closing_index) {
                    if *closing_char == '\'' {
                        for _ in 0..=closing_index.saturating_sub(*index) {
                            self.highlighting.push(highlighting::Type::Character);
                            *index += 1;
                        }
                        return true;
                    }
                }
            };
        }
        false
    }

    fn highlight_comment(&mut self, index: &mut usize, hl_opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if hl_opts.comments() && c == '/' && *index < chars.len() {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                if *next_char == '/' {
                    for _ in *index..chars.len() {
                        self.highlighting.push(highlighting::Type::Comment);
                        *index += 1;
                    }
                    return true;
                }
            }
        }
        false
    }

    fn highlight_strings(&mut self, index: &mut usize, hl_opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if hl_opts.strings() && c == '"' {
            loop {
                self.highlighting.push(highlighting::Type::String);
                *index += 1;
    
                if let Some (next_char) = chars.get(*index) {
                    if *next_char == '"' {
                        break;
                    }
                } else {
                    break;
                }
            }
            self.highlighting.push(highlighting::Type::String);
            *index += 1;
            return true;
        }
        false
    }

    fn highlight_number(&mut self, index: &mut usize, hl_opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if hl_opts.number() && c.is_ascii_digit() {
            if *index > 0 {
                let prev_char = chars[*index - 1]; 
                if !prev_char.is_ascii_whitespace() && !prev_char.is_ascii_punctuation() {
                    return false;
                }
            }

            loop {
                self.highlighting.push(highlighting::Type::Number);
                *index += 1;
                if let Some(next_char) = chars.get(*index) {
                    if *next_char != '.' && !next_char.is_ascii_digit() {
                        break;
                    } 
                } else {
                    break;
                }
            }
            return true;
        }
        false
    }

    fn highlight_str(&mut self, index: &mut usize, substring: &str, chars: &[char], hl_type: highlighting::Type) -> bool {
        if substring.is_empty() {
            return false;
        }

        for (substring_idx, c) in substring.chars().enumerate() {
            if let Some(next_char) = chars.get(index.saturating_add(substring_idx)) {
                if *next_char != c {
                    return false;
                }
            } else {
                return false;
            }
        }

        for _ in 0..substring.len() {
            *index += 1;
            self.highlighting.push(hl_type);
        }

        true
    }

    fn highlight_keywords(&mut self, index: &mut usize, chars: &[char], keywords: &[String], hl_type: highlighting::Type) -> bool {
        if *index > 0 {
            #[allow(clippy::indexing_slicing, clippy::integer_arithmetic)]
            let prev_char = chars[*index - 1];
            if !is_separator(prev_char) {
                return false;
            }
        }
        for word in keywords {
            if *index < chars.len().saturating_sub(word.len()) {
                #[allow(clippy::indexing_slicing, clippy::integer_arithmetic)]
                let next_char = chars[*index + word.len()];
                if !is_separator(next_char) {
                    continue; // potential word is not followed by a separator, so just skip this idx
                }
            }
            if self.highlight_str(index, &word, chars, hl_type) {
                return true;
            }
        }
        false
    }

    fn highlight_primary_keywords(&mut self, index: &mut usize, hl_opts: &HighlightingOptions, chars: &[char]) -> bool {
        self.highlight_keywords(index, chars, hl_opts.primary_keywords(), highlighting::Type::PrimaryKeywords)
    }

    fn highlight_secondary_keywords(&mut self, index: &mut usize, hl_opts: &HighlightingOptions, chars: &[char]) -> bool {
        self.highlight_keywords(index, chars, hl_opts.secondary_keywords(), highlighting::Type::SecondaryKeywords)
    }

    fn highlight_multiline_comments(&mut self, index: &mut usize, hl_opts: &HighlightingOptions, c: char, chars: &[char]) -> bool {
        if hl_opts.comments() && c == '/' && *index < chars.len() {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                if *next_char == '*' {
                    let closing_index = 
                        if let Some(closing_index) = self.string[*index + 2..].find("*/") {
                            *index + closing_index + 4
                        } else {
                            chars.len()
                        };

                        for _ in *index..closing_index {
                            self.highlighting.push(highlighting::Type::MultiLineComment);
                            *index += 1;
                        }
                        return true;
                }
            } 
        }
        false
    }
 
    pub fn highlight(&mut self, hl_opts: &HighlightingOptions, word: &Option<String>, start_with_comment: bool) -> bool {
        let chars: Vec<char> = self.string.chars().collect();
        if self.is_highlighted && word.is_none() {
            if let Some(hl_type) = self.highlighting.last() {
                if *hl_type == highlighting::Type::MultiLineComment
                && self.string.len() > 1
                && self.string[self.string.len() - 2..] == *"*/" {
                    return true;
                }
            }

            return false;
        }
        self.highlighting = Vec::new();
        let mut index = 0;
        let mut in_ml_comment = start_with_comment;
        if in_ml_comment {
            let closing_index = if let Some(closing_index) = self.string.find("*/") {
                closing_index + 2
            } else {
                chars.len()
            };
            for _ in 0..closing_index {
                self.highlighting.push(highlighting::Type::MultiLineComment);
            }
            index = closing_index;
        }

        while let Some(c) = chars.get(index) {
            if self.highlight_multiline_comments(&mut index, &hl_opts, *c, &chars) {
                in_ml_comment = true;
                continue;
            }
            in_ml_comment = false;
            if self.highlight_char(&mut index, hl_opts, *c, &chars)
            || self.highlight_comment(&mut index, hl_opts, *c, &chars)
            || self.highlight_primary_keywords(&mut index, &hl_opts, &chars)
            || self.highlight_secondary_keywords(&mut index, &hl_opts, &chars)
            || self.highlight_number(&mut index, hl_opts, *c, &chars)
            || self.highlight_strings(&mut index, hl_opts, *c, &chars)  {
                continue;
            }

            self.highlighting.push(highlighting::Type::None);
            index += 1;
        }

        self.highlight_match(word);
        if in_ml_comment && &self.string[self.string.len().saturating_sub(2)..] != "*/" {
            return true;
        }
        self.is_highlighted = true;
        false
    }
}

fn is_separator(c: char) -> bool {
    c.is_ascii_whitespace() || c.is_ascii_punctuation()
}