mod buffer;
mod editor;
mod error;
mod file;
mod normalmode;
mod terminal;
mod utils;
use crate::editor::Editor;
use crate::error::Result;

fn main() -> Result<()> {
    Editor::new()?.run();
    Ok(())
}
