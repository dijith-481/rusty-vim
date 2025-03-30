use crate::editor::EditorModes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    EndOfLine,
    EndOfRows,
    PageTop,
    PageMiddle,
    PageBottom,
    GoToLine,
    StartOfLine,
    StartOfNonWhiteSpace,
    Word,
    ParagraphEnd,
    ParagraphStart,
    WORD,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalAction {
    Move(Motion),
    ChangeMode(EditorModes),
    NewLine,
    Delete,
    Unknown,
}
