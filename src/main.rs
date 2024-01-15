use std::io::{self, stdout, Read};
use termion::raw::IntoRawMode;

fn convert_to_control(c: char) -> u8 {
    let byte = c as u8;
    byte & 0b0001_1111 // create a mask to keep the last 5 bits only
}

fn main() {
    let _stdout = stdout().into_raw_mode().unwrap();
    
    for b in io::stdin().bytes() {
        let b = b.unwrap();
        let c = b as char;
        if c.is_control() {
            println!("{:?} \r", b);
        } else {
            println!("{:?} ({})\r", b, c)
        }
        if b == convert_to_control('q') {
            break;
        }
    }
}
