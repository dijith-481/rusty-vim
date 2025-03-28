mod buffer;
mod editor;
mod error;
mod terminal;

use crate::error::Result;
use buffer::TextBuffer;
use editor::Editor;
use std::{
    env::{self},
    io::{self, Write, stdout},
};

// const fn ctrl_key(c: u8) -> u8 {
//     c & 0x1f
// }

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut buffer = TextBuffer::new();
    buffer.load_file(&args[1]);
    let mut e = Editor::new(buffer)?;
    loop {
        e.refresh_screen();
        e.process_keypress();
        // stdout().flush().expect("flush");
        if e.exit_flag {
            // write!(io::stdout(), "\x1b[2J").expect("write");
            // stdout().flush().expect("flush");
            // write!(io::stdout(), "\x1b[H").expect("write");
            // stdout().flush().expect("flush");
            break;
        }
    }
    write!(io::stdout(), "\x1b[2J").expect("write");
    stdout().flush().expect("flush");
    write!(io::stdout(), "\x1b[H").expect("write");
    stdout().flush().expect("flush");
    Ok(())
}
