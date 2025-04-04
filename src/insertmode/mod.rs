pub enum InsertAction {
    Backspace,
    Newline,
    Escape,
    None,
    Chars(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsertType {
    None,
    Append,
    InsertStart,
    AppendEnd,
    Next,
    Prev,
}
impl InsertAction {
    pub fn handle_key(c: u8) -> InsertAction {
        match c {
            b'\x1b' => InsertAction::Escape,
            127 => InsertAction::Backspace,
            13 => InsertAction::Newline,
            9 => InsertAction::Chars(c),
            c if c > 32 => InsertAction::Chars(c),
            _ => InsertAction::None,
        }
    }
}
