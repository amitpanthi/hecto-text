use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;
use std::io::{self, stdout, Write};

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn run(&mut self) {
        let _stdout = stdout().into_raw_mode().unwrap();

        loop {
            if let Err(error) = self.clear_screen() {
                die(error);
            }

            if let Err(error) = self.process_keypress() {
                die(error);
            }

            if self.should_quit {
                break;
            } 
        }
    }

    pub fn default() -> Self {
        Self { should_quit : false }
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = read_key()?;
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            _ => (),
        }

        Ok(())
    }

    
    fn clear_screen(&self) -> Result<(), std::io::Error> {
        println!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));

        if self.should_quit {
            print!("goodbye");
        } else {
            self.draw_rows();
            print!("{}", termion::cursor::Goto(1, 1));
        }
        
        io::stdout().flush()
    }

    fn draw_rows(&self) {
        for _ in 0..24 {
            println!("~\r");
        }
    }
    
}

fn die(e: std::io::Error) {
    println!("{}", termion::clear::All);
    panic!("{}", e);
}

fn read_key() -> Result<Key, std::io::Error> {
    loop {
        if let Some(key) = io::stdin().lock().keys().next() {
            return key;
        }
    }
}
