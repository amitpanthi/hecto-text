use crate::Terminal;
use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;
use std::io::{self, stdout, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Position {
    pub x: usize,
    pub y: usize
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
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
        Self { 
            should_quit : false,
            terminal : Terminal::default().expect("Failed to initialize terminal."),
            cursor_position: Position {x : 0, y : 0},
        }
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
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

        Ok(())
    }

    fn move_cursor(&mut self, pressed_key: Key) {
        let Position { mut x, mut y } = self.cursor_position;
        let height = self.terminal.size().height.saturating_sub(1) as usize; // heights and widths for terminal follow 1-based indexing, offset by 1
        let width = self.terminal.size().width.saturating_sub(1) as usize;

        match pressed_key {
            Key::Up => y = y.saturating_sub(1),

            Key::Down =>{
                if y < height {
                    y = y.saturating_add(1);
                }
            },

            Key::Left => x = x.saturating_sub(1),

            Key::Right => {
                if x < width {
                    x = x.saturating_add(1);
                }
            },

            Key::PageUp => y = 0,

            Key::PageDown => y = height,

            Key::Home => x = 0,

            Key::End => x = width,

            _ => ()
        }

        self.cursor_position = Position { x , y };
    }
    
    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::clear_screen();
        Terminal::cursor_position(&Position {x : 0, y : 0});
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye\r");
        } else {
            self.draw_rows();
            Terminal::cursor_position(&self.cursor_position);
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;

        for row in 0..height - 1 {
            Terminal::clear_current_line();
            if row == height/3 {
                self.print_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn print_welcome_message(&self) {
        let mut welcome_message = format!("Welcome to Hecto v{}.\r", VERSION);
        let width =self.terminal.size().width as usize;
        let padding = width.saturating_sub(welcome_message.len())/2; 
        let spaces = " ".repeat(padding);
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);

        println!("{}\r", welcome_message);
    }
    
}

fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}

