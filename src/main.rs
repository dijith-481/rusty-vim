mod buffer;
pub mod commandmode;
mod editor;
mod error;
mod file;
mod normalmode;
mod terminal;
use crate::editor::Editor;
use crate::error::Result;

fn main() -> Result<()> {
    Editor::new()?.run()?;
    Ok(())
}
