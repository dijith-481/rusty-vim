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
fn main() -> Result<()> {
    Editor::new().run();
    Ok(())
}