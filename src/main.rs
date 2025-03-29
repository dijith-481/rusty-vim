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
    string,
};

// const fn ctrl_key(c: u8) -> u8 {
//     c & 0x1f
// }

fn main() -> Result<()> {
    Editor::new().run();
    Ok(())
}

