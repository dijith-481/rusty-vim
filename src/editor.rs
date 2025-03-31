use crate::buffer::TextBuffer;
use crate::commandmode::{CommandMode, CommandReturn};
use crate::error::Result;

use crate::normalmode::NormalMode;
use crate::normalmode::motions::BufferMotion;
use crate::normalmode::motions::NormalAction;
use crate::terminal::Terminal;
use std::cmp::{self, max};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorModes {
    Normal,
    Insert,
    Command,
}

pub struct Editor {
    terminal: Terminal,
    buffer: TextBuffer,
    pub exit_flag: bool,
    pub mode: EditorModes,
    normal_mode: NormalMode,
    command_mode: CommandMode,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let buffer = TextBuffer::new()?;
        let terminal = Terminal::new(buffer.rows.len(), &buffer.filename)?;
        Ok(Self {
            normal_mode: NormalMode::new(),
            command_mode: CommandMode::new(),
            terminal,
            buffer,
            exit_flag: false,
            mode: EditorModes::Normal,
        })
    }
    pub fn run(&mut self) -> Result<()> {
        loop {
            if self.exit_flag {
                return Ok(());
            }
            self.process_keypress()?;
            self.render_ui()?;
        }
    }
    fn render_ui(&mut self) -> Result<()> {
        self.terminal.refresh_screen(&self.buffer)?;
        Ok(())
    }

    fn handle_operation(&mut self) {
        let repeat = max(self.normal_mode.pending_operations.repeat, 1);
        if self.normal_mode.pending_operations.is_action_given() {
            self.terminal.status_line_right =
                String::from(self.normal_mode.pending_operations.action);
            match self.normal_mode.pending_operations.action {
                'd' => match self.normal_mode.pending_operations.motion {
                    'd' => self.buffer.delete_row(),
                    'h' => self.buffer.delete(BufferMotion::Left(repeat)),
                    '$' => self.buffer.delete(BufferMotion::EndOfLine(repeat)),
                    'G' => self.buffer.delete(BufferMotion::EndOfFile),
                    'w' => self.buffer.delete(BufferMotion::Word(repeat)),
                    'W' => self.buffer.delete(BufferMotion::WORD(repeat)),
                    _ => (),
                },
                'g' => match self.normal_mode.pending_operations.motion {
                    'g' => self
                        .buffer
                        .motion(BufferMotion::GoToLine(repeat.saturating_sub(1))),
                    _ => (),
                },
                _ => (),
            }
        } else {
            match self.normal_mode.pending_operations.motion {
                'h' => self.buffer.motion(BufferMotion::Left(repeat)),
                'j' => self.buffer.motion(BufferMotion::Down(repeat)),
                'k' => self.buffer.motion(BufferMotion::Up(repeat)),
                'l' => self.buffer.motion(BufferMotion::Right(repeat)),
                '$' => self.buffer.motion(BufferMotion::EndOfLine(repeat)),
                '0' => self.buffer.motion(BufferMotion::StartOfLine),
                '^' => self.buffer.motion(BufferMotion::StartOfNonWhiteSpace),
                'w' => self.buffer.motion(BufferMotion::Word(repeat)),
                'W' => self.buffer.motion(BufferMotion::WORD(repeat)),
                // 'H' => self.buffer.motion(BufferMotion::PageTop(repeat)),
                '{' => self.buffer.motion(BufferMotion::ParagraphStart(repeat)),
                '}' => self.buffer.motion(BufferMotion::ParagraphEnd(repeat)),
                // 'M' => self.buffer.motion(BufferMotion::PageMiddle(repeat)),
                // 'L' => self.buffer.motion(BufferMotion::PageBottom(repeat)),
                'G' => self.buffer.motion(BufferMotion::EndOfFile),
                'i' => self.change_mode(EditorModes::Insert),
                ':' => {
                    self.change_mode(EditorModes::Command);
                    self.process_command_mode(self.normal_mode.pending_operations.motion as u8);
                    self.terminal.status_line_left = String::from(":");
                }
                'a' => {
                    // if !self.buffer.rows.get(self.buffer.pos.y).unwrap().len() == 0 {
                    //     self.buffer.pos.x += 1;
                    // }
                    self.change_mode(EditorModes::Insert);
                }
                'A' => {
                    // self.normal_action(NormalAction::Move(BufferMotion::EndOfLine));
                    self.buffer.pos.x += 1;
                    self.change_mode(EditorModes::Insert);
                }
                'I' => {
                    self.normal_action(NormalAction::Move(BufferMotion::StartOfNonWhiteSpace));
                    self.change_mode(EditorModes::Insert);
                }
                'o' => {
                    self.buffer.pos.y += 1;
                    self.normal_action(NormalAction::NewLine);
                    self.change_mode(EditorModes::Insert);
                }
                'O' => {
                    self.normal_action(NormalAction::NewLine);
                    self.change_mode(EditorModes::Insert);
                }
                'x' => {
                    self.buffer.delete_char();
                }
                _ => self.normal_action(NormalAction::Unknown),
            }
        }
        // }
        self.normal_mode.pending_operations.reset();

        // write!(io::stdout(), "{}", abuf);
        // stdout().flush().expect("flush");
    }

    fn process_normal_mode(&mut self, c: u8) {
        if c < 32 {
            return;
        }
        self.normal_mode.pending_operations.insert_key(c as char);
        println!("self ");

        let action_given = self.normal_mode.pending_operations.is_motion_given();
        if action_given {
            println!("given ");
            self.handle_operation();
        }
        // self.status_line_right = String::new();
    }
    fn change_mode(&mut self, mode: EditorModes) {
        self.mode = mode;
    }
    fn normal_action(&mut self, action: NormalAction) {
        match action {
            NormalAction::Move(direction) => self.buffer.motion(direction),
            NormalAction::ChangeMode(editormode) => {
                self.terminal.change_cursor(editormode);
                self.mode = editormode;
            }
            NormalAction::NewLine => self.buffer.insert_newline(),
            NormalAction::Delete => self.buffer.delete_char(),
            _ => (),
        }
    }

    fn process_insert_mode(&mut self, c: u8) {
        if c == b'\x1b' {
            if self.buffer.pos.x >= self.buffer.rows[self.buffer.pos.y].len() {
                self.buffer.pos.x =
                    cmp::max(self.buffer.rows.get(self.buffer.pos.y).unwrap().len(), 0) - 1;
            } else if self.buffer.pos.x > 0 {
                self.buffer.pos.x = self.buffer.pos.x.saturating_sub(1);
            }

            self.terminal.change_cursor(EditorModes::Normal);
            self.mode = EditorModes::Normal;
            return;
        } else if c == 127 {
            if self.buffer.pos.x == 0 && self.buffer.pos.y > 0 {
                let content = self.buffer.rows.remove(self.buffer.pos.y);
                self.buffer.rows[self.buffer.pos.y - 1].push_str(&content);
                self.buffer.pos.y -= 1;
                self.buffer.pos.x = self.buffer.rows[self.buffer.pos.y].len() - 1;
            } else {
                self.normal_action(NormalAction::Delete);
                self.buffer.pos.x -= 1;
            }
        } else if c == 13 {
            self.buffer.pos.y += 1;
            self.normal_action(NormalAction::NewLine);
        } else if !((c) < 32) {
            self.buffer.insert_char(c);
        }
    }
    fn process_command_mode(&mut self, c: u8) {
        let value = self.command_mode.handle_key(c);
        match value {
            CommandReturn::Quit => self.exit_flag = true,
            CommandReturn::None => {
                self.terminal.status_line_right = String::from("None");
            }
            CommandReturn::Save => {
                self.terminal.status_line_right = String::from("Save");
            }
            CommandReturn::Escape => {
                self.mode = EditorModes::Normal;
                self.terminal.status_line_right = String::from("Escape");
            }
        }
        if self.mode == EditorModes::Command {
            self.terminal.command_line = String::from(":");
        } else {
            self.terminal.command_line = String::new();
        }
        self.terminal
            .command_line
            .push_str(&self.command_mode.command);
        // }
    }
    fn process_keypress(&mut self) -> Result<()> {
        let c = self.terminal.read_key()?;
        match self.mode {
            EditorModes::Normal => {
                self.process_normal_mode(c);
                self.terminal.status_line_left = String::from("Normal");
            }
            EditorModes::Insert => {
                self.process_insert_mode(c);
                self.terminal.status_line_left = String::from("Insert");
            }
            EditorModes::Command => {
                self.process_command_mode(c);
                self.terminal.status_line_left = String::from("Command");
            }
        }
        Ok(())
    }
}
impl Drop for Editor {
    fn drop(&mut self) {}
}
