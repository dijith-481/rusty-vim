use crate::buffer::TextBuffer;
use crate::commandmode::{CommandMode, CommandReturn};
use crate::error::{AppError, Result};
use crate::normalmode::NormalMode;
use crate::normalmode::motions::NormalAction;
use crate::normalmode::motions::{BufferMotion, Motion};
use crate::terminal::Terminal;
use std::cmp::{self, max};
use std::env;
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
    save_flag: bool,
    pub mode: EditorModes,
    normal_mode: NormalMode,
    command_mode: CommandMode,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let args: Vec<String> = env::args().collect();
        let buffer = TextBuffer::new(args)?;

        let terminal = Terminal::new(buffer.rows.len(), &buffer.filename)?;
        Ok(Self {
            normal_mode: NormalMode::new(),
            command_mode: CommandMode::new(),
            terminal,
            buffer,
            exit_flag: false,
            save_flag: false,
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
                    'd' => self.buffer.delete(BufferMotion::Down(0)),
                    'h' => self.buffer.delete(BufferMotion::Left(repeat)),
                    'l' => self.buffer.delete(BufferMotion::Right(repeat)),
                    '$' => self.buffer.delete(BufferMotion::EndOfLine(repeat)),
                    'G' => self.buffer.delete(BufferMotion::EndOfFile),
                    'w' => self.buffer.delete(BufferMotion::Word(repeat)),
                    'W' => self.buffer.delete(BufferMotion::WORD(repeat)),
                    '{' => self.buffer.delete(BufferMotion::ParagraphStart(repeat)),
                    '}' => self.buffer.delete(BufferMotion::ParagraphEnd(repeat)),
                    '^' => self.buffer.delete(BufferMotion::StartOfNonWhiteSpace),
                    'j' => self.buffer.delete(BufferMotion::Down(repeat)),
                    'k' => self.buffer.delete(BufferMotion::Up(repeat)),
                    '\x7F' => self.buffer.delete(BufferMotion::BackSpace(repeat)),
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
                '\x7F' => self.buffer.motion(BufferMotion::BackSpace(repeat)),
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
                    self.buffer.pos.y += repeat;
                    self.buffer.rows.insert(self.buffer.pos.y, String::new());
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
        self.normal_mode.pending_operations.reset();
    }

    fn process_normal_mode(&mut self, c: u8) {
        if c < 32 {
            return;
        }
        self.normal_mode.pending_operations.insert_key(c as char);
        let action_given = self.normal_mode.pending_operations.is_motion_given();
        if action_given {
            self.handle_operation();
        }
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

    fn activate_normal_mode(&mut self) {
        self.buffer.fix_cursor_pos_escape_insert();
        self.terminal.change_cursor(EditorModes::Normal);
        self.mode = EditorModes::Normal;
    }

    fn process_insert_mode(&mut self, c: u8) {
        if c == b'\x1b' {
            self.activate_normal_mode();
            return;
        } else if c == 127 {
            self.buffer.delete(BufferMotion::BackSpace(1));
            // self.buffer.delete(BufferMotion::Left(1));
            // self.buffer.motion(BufferMotion::Left(1));
        } else if c == 13 {
            self.buffer.split_line();
            // self.buffer.pos.y += 1;
            // self.normal_action(NormalAction::NewLine);
        } else if !((c) < 32) {
            self.buffer.insert_char(c);
        }
    }
    fn process_command_mode(&mut self, c: u8) {
        let value = self.command_mode.handle_key(c, self.save_flag);
        match value {
            CommandReturn::FileName(filename) => {
                self.buffer.filename = filename.to_string();
                self.buffer.write_buffer_file();
                self.exit_flag = true;
            }
            CommandReturn::Quit => self.exit_flag = true,
            CommandReturn::None => {
                self.terminal.status_line_right = String::from("None");
            }
            CommandReturn::Save => {
                self.buffer.write_buffer_file();
                self.terminal.status_line_right = String::from("Save");
            }
            CommandReturn::SaveQuit => {
                match self.buffer.write_buffer_file() {
                    Ok(_) => self.save_flag = false,
                    Err(_) => self.save_flag = true,
                }
                if !self.save_flag {
                    self.exit_flag = true;
                }
                self.command_mode.command = String::new();
                self.terminal.status_line_right = String::from("enter file name:");
            }
            CommandReturn::Escape => {
                self.mode = EditorModes::Normal;
                self.terminal.status_line_right = String::from("Escape");
            }
        }
        if self.mode == EditorModes::Command {
            if self.save_flag {
                self.terminal.command_line = String::from("enter file name: ");
            } else {
                self.terminal.command_line = String::from(":");
            }
        } else {
            self.terminal.command_line = String::new();
        }
        self.terminal
            .command_line
            .push_str(&self.command_mode.command);
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
