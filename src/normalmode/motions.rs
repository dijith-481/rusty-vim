use crate::editor::EditorModes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferMotion {
    Left(usize),
    Right(usize),
    Up(usize),
    Down(usize),
    EndOfLine(usize),
    EndOfFile,
    GoToLine(usize),
    BackSpace(usize),
    GoToX(usize),
    StartOfLine,
    StartOfNonWhiteSpace,
    Word(usize),
    ParagraphEnd(usize),
    ParagraphStart(usize),
    WORD(usize),
}
pub enum Motion {
    BufferMotion,
    PageTop,
    PageMiddle,
    PageBottom,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalAction {
    Move(BufferMotion),
    ChangeMode(EditorModes),
    NewLine,
    Delete,
    Unknown,
}
