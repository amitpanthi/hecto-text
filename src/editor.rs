use crate::Terminal;
use crate::Document;
use crate::Row;
use crate::document;
use crate::row;
use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::color;
use termion::input::TermRead;
use std::io::{self, stdout, Write};
use std::env;
use std::time::Instant;
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const QUIT_TIMES: u8 = 3;

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}
pub struct StatusMessage {
    text: String,
    time: Instant,
}
impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            text: message,
            time: Instant::now(),
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
    offset: Position,
    status_message: StatusMessage,
    quit_times: u8,
}

impl Editor {
    pub fn run(&mut self) {
        let _stdout = stdout().into_raw_mode().unwrap();

        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }


            if self.should_quit {
                break;
            } 

            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
    }

    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("Tip: Ctrl-F -> Find | Ctrl-S -> Save | Ctrl-Q -> Quit");
        let document = if let Some(file_name) = args.get(1) {
            let file = Document::open(&file_name);
            
            if let Ok(file) = file {
                file
            } else {
                initial_status = format!("ERROR: Could not open file - {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self { 
            should_quit : false,
            terminal : Terminal::default().expect("Failed to initialize terminal."),
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
        }
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Need a file name to save!".to_string());
                return;
            } 
        
        self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully!".to_string());
        } else {
            self.status_message = StatusMessage::from("There was an error saving this file!".to_string());
        }
    }

    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        let mut direction = SearchDirection::Forward;

        let query = self.prompt("Search (ESC to cancel, arrow keys to navigate): ", |editor, key, query|  {
            let mut moved = false;
            match key {
                Key::Right | Key::Down => {
                    direction = SearchDirection::Forward;
                    editor.move_cursor(Key::Right);
                    moved = true;
                }
                Key::Left | Key::Up => direction = SearchDirection::Backward,
                _ => direction = SearchDirection::Forward,
            }
             if let Some(position) = editor.document.find(&query, &editor.cursor_position, direction) {
                editor.cursor_position = position;
                editor.scroll();
             } else if moved {
                editor.move_cursor(Key::Left);
             }
             editor.document.highlight(Some(query));
        }).unwrap_or(None);
        
        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
        self.document.highlight(None);
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => {
                if self.document.is_dirty() && self.quit_times > 0 {
                    self.status_message = StatusMessage::from(format!("Warning! Quitting without saving. Press Ctrl-Q {} more times to quit.", self.quit_times));
                    self.quit_times -= 1;
                    return Ok(());
                } 
                self.should_quit = true
            },
            Key::Ctrl('s') => self.save(),
            Key::Ctrl('f') => self.search(),
            Key::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right);
            },
            Key::Delete => self.document.delete(&self.cursor_position),
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            Key::Up 
            | Key::Down 
            | Key::Left 
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::Home
            | Key::End => self.move_cursor(pressed_key),
            _ => (),
        }

        self.scroll();
        if self.quit_times < QUIT_TIMES { //Some button other than Ctrl-Q was pressed, so reset
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }

    fn move_cursor(&mut self, pressed_key: Key) {
        let Position { mut x, mut y } = self.cursor_position;
        let terminal_height = self.terminal.size().height as usize;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.get_row(y) {
            row.len()
        } else { 0 };

        match pressed_key {
            Key::Up => y = y.saturating_sub(1),

            Key::Down =>{
                if y < height {
                    y = y.saturating_add(1);
                }
            },

            Key::Left => {
                if x > 0 {
                    x = x.saturating_sub(1)
                } else if y > 0 {
                    y -= 1;
                    x = if let Some(row) = self.document.get_row(y) {
                        row.len()
                    } else {
                        0
                    }
                }
            },

            Key::Right => {
                if x < width {
                    x = x.saturating_add(1);
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            },

            Key::PageUp => {
                y = if y <= terminal_height {
                    0
                } else {
                   y.saturating_sub(terminal_height)
                }
               },

            Key::PageDown => {
                y = if height.saturating_sub(y) <= terminal_height {
                    height
                } else {
                    y.saturating_add(terminal_height)
                }},

            Key::Home => x = 0,

            Key::End => x = width,

            _ => ()
        }

        width = if let Some(row) = self.document.get_row(y) {
            row.len()
        } else { 0 };

        if x > width {
            x = width;
        }

        self.cursor_position = Position { x , y };
    }
    
    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());

        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }

        Terminal::cursor_show();
        Terminal::flush()
    }
    
    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, std::io::Error>
    where
        C: FnMut(&mut Self, Key, &String), {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;

            let key = Terminal::read_key()?;

            match key {
                Key::Char('\n') => {
                    break;
                }
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                },
                Key::Esc => {
                    result.truncate(0);
                    break;
                },
                Key::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len().saturating_sub(1));
                    }
                },
                _ => ()
            }
            callback(self, key, &result);
        }

        self.status_message = StatusMessage::from(String::new());

        if(result.is_empty()){
            return Ok(None);
        }

        Ok(Some(result))
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    #[allow(clippy::integer_arithmetic, clippy::integer_division)]
    fn draw_rows(&self) {
        let height = self.terminal.size().height;

        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.get_row(self.offset.y.saturating_add(terminal_row as usize)) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height/3 {
                self.print_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn print_welcome_message(&self) {
        let mut welcome_message = format!("Welcome to Hecto v{}.\r", VERSION);
        let width =self.terminal.size().width as usize;
        #[allow(clippy::integer_arithmetic, clippy::integer_division)]
        let padding = width.saturating_sub(welcome_message.len())/2; 
        let spaces = " ".repeat(padding);
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);

        println!("{}\r", welcome_message);
    }

    fn scroll(&mut self) {
        let Position {x, y} = self.cursor_position;
        let mut offset = &mut self.offset;
        let height = self.terminal.size().height as usize;
        let width = self.terminal.size().width as usize;

        if y < offset.y {
            // if you scroll up, update offset to the new reduced y value
            offset.y = y; 
        } else if y >= offset.y.saturating_add(height) {
            // if y is out of screen, update offset.y, which always represents the top most line visible in a given file.
            // ex: if y is one line out of screen, offset.y increases by 1.
            offset.y = y.saturating_sub(height).saturating_add(1); 
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }
    
    fn draw_status_bar(&self) {
        let mut status;
        let mut file_name = "[No file name]".to_string();
        let width = self.terminal.size().width as usize;
        let modified_message = if self.document.is_dirty() {
            "(modified)"
        } else {
            ""
        };
    
        if let Some(fname) = &self.document.file_name {
            file_name = fname.clone();
            file_name.truncate(20);
        }

        status = format!("{} - {} lines {}", file_name, self.document.len(), modified_message);
        let line_indicator = format!("{}/{}", self.cursor_position.y.saturating_add(1), self.document.len());
        #[allow(clippy::integer_arithmetic)]
        let status_len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(status_len)));
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);
        Terminal::set_bg_color(STATUS_COLOR);
        Terminal::set_fg_color(FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let status = &self.status_message;
        if(Instant::now() - status.time < Duration::new(5, 0)) {
            let mut text = status.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

}

fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}

