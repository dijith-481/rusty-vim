use crate::editor::EditorModes;
use crate::insertmode::InsertType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left(usize),
    Right(usize),
    Up(usize),
    Down(usize),
    EndOfLine(usize),
    EndOfFile,
    GoToLine(usize),
    BackSpace(usize),
    StartOfLine,
    StartOfNonWhiteSpace,
    Word(usize),
    ParagraphEnd(usize),
    ParagraphStart(usize),
    WORD(usize),
}
pub enum BufferAction {
    Delete(Motion),
    ChangeMode(EditorModes, InsertType),
    None,
    Move(Motion),
}
