mod buffer;
mod editor;
mod error;
mod terminal;
mod utils;
// use buffer::TextBuffer;
use editor::Editor;

fn main() {
    Editor::new().run();
}
