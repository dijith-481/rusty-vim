pub mod motions;
pub mod operation_pending;

use crate::editor::EditorModes;
use crate::insertmode::InsertType;
use motions::{BufferAction, Motion};
use operation_pending::PendingOperations;
use std::cmp::max;

pub enum NormalKeyError {
    InvalidKey,
    NotMotion,
}

pub struct NormalMode {
    pub pending_operations: PendingOperations,
}
impl NormalMode {
    pub fn new() -> Self {
        let pending_operations = PendingOperations::new();
        Self { pending_operations }
    }
    pub fn handle_keypress(&mut self, c: u8) -> Result<BufferAction, NormalKeyError> {
        if c < 32 {
            return Err(NormalKeyError::InvalidKey);
        }
        self.pending_operations.insert_key(c as char);
        let motion_given = self.pending_operations.is_motion_given();
        if motion_given {
            let repeat = max(self.pending_operations.repeat, 1);
            let action = self.handle_operation(repeat);
            return Ok(action);
        }
        Err(NormalKeyError::NotMotion)
    }

    pub fn handle_operation(&mut self, repeat: usize) -> BufferAction {
        if self.pending_operations.is_action_given() {
            match self.pending_operations.action {
                'd' => match self.pending_operations.motion {
                    'd' => BufferAction::Delete(Motion::Down(repeat.saturating_sub(1))),
                    'h' => BufferAction::Delete(Motion::Left(repeat)),
                    'l' => BufferAction::Delete(Motion::Right(repeat)),
                    '0' => BufferAction::Delete(Motion::StartOfLine),
                    '$' => BufferAction::Delete(Motion::EndOfLine(repeat)),
                    'G' => BufferAction::Delete(Motion::EndOfFile),
                    'w' => BufferAction::Delete(Motion::Word(repeat)),
                    'W' => BufferAction::Delete(Motion::WORD(repeat)),
                    '{' => BufferAction::Delete(Motion::ParagraphStart(repeat)),
                    '}' => BufferAction::Delete(Motion::ParagraphEnd(repeat)),
                    '^' => BufferAction::Delete(Motion::StartOfNonWhiteSpace),
                    'j' => BufferAction::Delete(Motion::Down(repeat)),
                    'k' => BufferAction::Delete(Motion::Up(repeat)),
                    '\x7F' => BufferAction::Delete(Motion::BackSpace(repeat)),
                    _ => BufferAction::None,
                },
                'g' => match self.pending_operations.motion {
                    'g' => BufferAction::Move(Motion::GoToLine(repeat.saturating_sub(1))),
                    _ => BufferAction::None,
                },
                _ => BufferAction::None,
            }
        } else {
            match self.pending_operations.motion {
                'd' => BufferAction::Move(Motion::Down(1)),
                'h' => BufferAction::Move(Motion::Left(repeat)),
                'l' => BufferAction::Move(Motion::Right(repeat)),
                '$' => BufferAction::Move(Motion::EndOfLine(repeat)),
                'G' => BufferAction::Move(Motion::EndOfFile),
                'w' => BufferAction::Move(Motion::Word(repeat)),
                'W' => BufferAction::Move(Motion::WORD(repeat)),
                '{' => BufferAction::Move(Motion::ParagraphStart(repeat)),
                '}' => BufferAction::Move(Motion::ParagraphEnd(repeat)),
                '^' => BufferAction::Move(Motion::StartOfNonWhiteSpace),
                'j' => BufferAction::Move(Motion::Down(repeat)),
                'k' => BufferAction::Move(Motion::Up(repeat)),
                '0' => BufferAction::Move(Motion::StartOfLine),
                'x' => BufferAction::Delete(Motion::Right(1)),
                '\x7F' => BufferAction::Delete(Motion::BackSpace(repeat)),
                'i' => BufferAction::ChangeMode(EditorModes::Insert, InsertType::None),
                ':' => BufferAction::ChangeMode(EditorModes::Command, InsertType::None),
                'a' => BufferAction::ChangeMode(EditorModes::Insert, InsertType::Append),
                'A' => BufferAction::ChangeMode(EditorModes::Insert, InsertType::AppendEnd),
                'I' => BufferAction::ChangeMode(EditorModes::Insert, InsertType::InsertStart),
                'o' => BufferAction::ChangeMode(EditorModes::Insert, InsertType::Next),
                'O' => BufferAction::ChangeMode(EditorModes::Insert, InsertType::Prev),
                _ => BufferAction::None,
            }
        }
    }
}
