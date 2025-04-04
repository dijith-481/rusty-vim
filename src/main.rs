mod buffer;
pub mod commandmode;
mod editor;
mod error;
mod file;
mod insertmode;
mod normalmode;
mod terminal;
use std::env;

use crate::editor::Editor;
use crate::error::Result;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    Editor::new(args)?.run()?;
    Ok(())
}
