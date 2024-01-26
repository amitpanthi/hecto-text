use crate::FileType;
use crate::Row;
use crate::Position;
use crate::SearchDirection;
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
    file_type: FileType,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let mut rows = Vec::new();
        let file_contents = fs::read_to_string(filename)?;
        let file_type = FileType::from(filename);
        for line in file_contents.lines() {
            rows.push(Row::from(line));
        }

        Ok(Self { 
            rows,
            file_name: Some(filename.to_string()),
            dirty: false,
            file_type,
        })
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            self.file_type = FileType::from(&file_name);
            let mut file = fs::File::create(file_name)?;
            for row in &mut self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }
        self.dirty = false;
        Ok(())
    }

    pub fn get_row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn file_type(&self) -> String {
        self.file_type.name()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[allow(clippy::integer_arithmetic)]
    pub fn delete(&mut self, at: &Position) {
        let len = self.rows.len();

        if at.y >= len {
            return;
        } 

        self.dirty = true;
        
        if at.x == self.rows[at.y].len() && at.y + 1 < len {
            let next_row = self.rows.remove(at.y + 1);
            let current_row = &mut self.rows[at.y];
            current_row.append(next_row);
        } else {
            let current_row = &mut self.rows[at.y];
            current_row.delete(at.x);
        }
        self.unhighlight_rows(at.y);
    }

    pub fn insert_newline(&mut self, at: &Position) {
        if at.y > self.rows.len() { //how would that even happen
            return;
        }

        self.dirty = true;

        if at.y == self.rows.len() || at.y + 1 == self.len() {
            self.rows.push(Row::default());
            return;
        } 

        #[allow(clippy::indexing_slicing)] 
        let current_row = &mut self.rows[at.y];
        let new_row = current_row.split(at.x);
        #[allow(clippy::integer_arithmetic)]
        self.rows.insert(at.y + 1, new_row);
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.rows.len() {
            return;
        }

        self.dirty = true;

        if c == '\n' {
            self.insert_newline(at);
        } else if at.y == self.rows.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            #[allow(clippy::indexing_slicing)]
            let current_row = &mut self.rows[at.y];
            current_row.insert(at.x, c);
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn find(&self, query: &str, at: &Position, search_direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }

        let mut position = Position {x: at.x, y: at.y};

        let start = if search_direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };

        let end = if search_direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };

        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(&query, position.x, search_direction) {
                    position.x = x;
                    return Some(position);
                }

                if search_direction == SearchDirection::Forward {
                    position.x = 0;
                    position.y = position.y.saturating_add(1);
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }

    pub fn highlight(&mut self, word: &Option<String>, until: Option<usize>) {
        let mut start_with_comment = false;
        let until = if let Some(until) = until {
            if until.saturating_add(1) < self.rows.len() {
                until.saturating_add(1)
            } else {
                self.rows.len()
            }
        } else {
            self.rows.len()
        };
        #[allow(clippy::indexing_slicing)]
        for row in &mut self.rows[..until] {
            start_with_comment = row.highlight(&self.file_type.highlighting_options(), word, start_with_comment);
        }
    }

    fn unhighlight_rows(&mut self, start: usize) {
        let start = start.saturating_sub(1);
        for row in self.rows.iter_mut().skip(start) {
            row.is_highlighted = false;
        }
    }
}