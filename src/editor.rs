use crate::buffer::{self, TextBuffer};
use crate::error::{AppError, Result};
use crate::terminal::{Size, Terminal};
use std::cmp::{self, max};
use std::collections::hash_set;
use std::io::{self, Read, Write, stdout};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorModes {
    Normal,
    Insert,
    Command,
    Visual,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
    EndOfLine,
    EndOfRows,
    StartOfLine,
    StartOfNonWhiteSpace,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalAction {
    Move(Direction),
    ChangeMode(EditorModes),
    NewLine,
    Delete,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CammandModeAction {
    ChangeMode(EditorModes),
}

pub struct Editor {
    terminal: Terminal,
    buffer: TextBuffer,
    pub exit_flag: bool,
    pub mode: EditorModes,
    pos: Size,
}

impl Editor {
    pub fn new() -> Self {
        let buffer = TextBuffer::new();
        let terminal = Terminal::new().expect("error loading terminal");
        Self {
            terminal,
            buffer,
            exit_flag: false,
            mode: EditorModes::Normal,
            pos: Size { x: 0, y: 0 },
        }
    }
    pub fn run(&mut self) {
        loop {
            self.terminal.refresh_screen(&self.buffer, &self.pos);
            self.process_keypress();
            if self.exit_flag {
                break;
            }
        }
    }

    fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Left => {
                if self.pos.x > 0 {
                    self.pos.x -= 1;
                }
            }
            Direction::StartOfLine => {
                self.pos.x = 0;
            }
            Direction::EndOfRows => {
                self.pos.y = (self.buffer.rows.len()) - 1;
            }
            Direction::StartOfNonWhiteSpace => {
                self.pos.x = self.buffer.rows[self.pos.y]
                    .chars()
                    .position(|c| c != ' ')
                    .unwrap();
            }
            Direction::EndOfLine => {
                self.pos.x = self.buffer.rows[self.pos.y].len() - 1;
            }
            Direction::Right => {
                if self.pos.x < self.buffer.rows[self.pos.y].len() - 1 {
                    self.pos.x += 1;
                }
            }
            Direction::Up => {
                if self.pos.y > 0 {
                    if self.pos.x != 0 && self.pos.x == self.buffer.rows[self.pos.y].len() - 1 {
                        self.pos.x = self.buffer.rows[self.pos.y - 1].len() - 1;
                    }
                    self.pos.y -= 1;
                    if self.pos.x != 0 && self.pos.x >= self.buffer.rows[self.pos.y].len() - 1 {
                        self.pos.x = self.buffer.rows[self.pos.y].len() - 1;
                    }
                }
            }
            Direction::Down => {
                if self.pos.y < self.buffer.rows.len() - 1 {
                    if self.pos.x != 0 && self.pos.x == self.buffer.rows[self.pos.y].len() - 1 {
                        self.pos.x = self.buffer.rows[self.pos.y + 1].len() - 1;
                    }
                    self.pos.y += 1;
                    if self.pos.x != 0 && self.pos.x >= self.buffer.rows[self.pos.y].len() - 1 {
                        self.pos.x = self.buffer.rows[self.pos.y].len() - 1;
                    }
                }
            }
        }
    }

    fn process_normal_mode(&mut self, c: u8) {
        match c {
            b'h' => self.normal_action(NormalAction::Move(Direction::Left)),
            b'j' => self.normal_action(NormalAction::Move(Direction::Down)),
            b'k' => self.normal_action(NormalAction::Move(Direction::Up)),
            b'l' => self.normal_action(NormalAction::Move(Direction::Right)),
            b'$' => self.normal_action(NormalAction::Move(Direction::EndOfLine)),
            b'0' => self.normal_action(NormalAction::Move(Direction::StartOfLine)),
            b'^' => self.normal_action(NormalAction::Move(Direction::StartOfNonWhiteSpace)),
            b'G' => self.normal_action(NormalAction::Move(Direction::EndOfRows)),
            b'i' => self.normal_action(NormalAction::ChangeMode(EditorModes::Insert)),
            b':' => {
                self.normal_action(NormalAction::ChangeMode(EditorModes::Command));
                self.terminal.status_line_left = String::from(":");
            }
            b'a' => {
                self.pos.x += 1;
                // self.pos.x = cmp::min(self.pos.x + 1, self.buffer.rows[self.pos.y].len());
                self.normal_action(NormalAction::ChangeMode(EditorModes::Insert))
            }
            b'A' => {
                self.normal_action(NormalAction::Move(Direction::EndOfLine));
                self.pos.x += 1;
                self.normal_action(NormalAction::ChangeMode(EditorModes::Insert))
            }
            b'I' => {
                self.normal_action(NormalAction::Move(Direction::StartOfNonWhiteSpace));
                self.normal_action(NormalAction::ChangeMode(EditorModes::Insert));
            }
            b'o' => {
                self.pos.y += 1;
                self.normal_action(NormalAction::NewLine);
                self.normal_action(NormalAction::ChangeMode(EditorModes::Insert));
            }
            b'O' => {
                self.normal_action(NormalAction::NewLine);
                self.normal_action(NormalAction::ChangeMode(EditorModes::Insert));
            }
            b'x' => {
                if self.pos.x > 0 {
                    self.normal_action(NormalAction::Delete);
                }
            }
            _ => self.normal_action(NormalAction::Unknown),
        }
        // self.status_line_right = String::new();
    }
    fn normal_action(&mut self, action: NormalAction) {
        match action {
            NormalAction::Move(direction) => self.move_cursor(direction),
            NormalAction::ChangeMode(editormode) => {
                self.terminal.change_cursor(editormode);
                self.mode = editormode;
            }
            NormalAction::NewLine => self.buffer.insert_newline(&mut self.pos),
            NormalAction::Delete => self.buffer.delete_char(&mut self.pos),
            _ => (),
        }
    }

    fn process_insert_mode(&mut self, c: u8) {
        if c == b'\x1b' {
            if self.pos.x >= self.buffer.rows[self.pos.y].len() {
                self.pos.x = self.buffer.rows[self.pos.y].len() - 1;
            }

            self.terminal.change_cursor(EditorModes::Normal);
            self.mode = EditorModes::Normal;
            return;
        } else if c == 127 {
            if self.pos.x == 0 && self.pos.y > 0 {
                let content = self.buffer.rows.remove(self.pos.y);
                self.buffer.rows[self.pos.y - 1].push_str(&content);
                self.pos.y -= 1;
                self.pos.x = self.buffer.rows[self.pos.y].len() - 1;
            } else {
                self.normal_action(NormalAction::Delete);
                self.pos.x -= 1;
            }
        } else if c == 13 {
            self.normal_action(NormalAction::NewLine);
        } else if !((c) < 32) {
            self.buffer.insert_char(c, &mut self.pos);
        }
    }
    fn process_command_mode(&mut self, c: u8) {
        if c == b'\x1b' {
            self.terminal.status_line_left = String::new();
            self.mode = EditorModes::Normal;
            return;
        }
        if c == 13 {
            //enter key
            self.do_command();
            // self.exit_flag = true;
            // self.mode = EditorModes::Normal;
            return;
        }
        if c > 32 {
            self.terminal.status_line_left.push(c as char);
        }
    }
    fn do_command(&mut self) {
        if self
            .terminal
            .status_line_left
            .as_str()
            .starts_with("enter a filename: ")
        {
            self.buffer.filename.push_str(
                &(self
                    .terminal
                    .status_line_left
                    .strip_prefix("enter a filename: ")
                    .unwrap()),
            );
            self.save_file();
        } else {
            match self.terminal.status_line_left.as_str() {
                ":w" => self.save_file(),
                ":q" => self.exit_flag = true,
                ":wq" => {
                    self.save_file();
                    if self.mode == EditorModes::Normal {
                        self.exit_flag = true;
                    }
                }
                _ => self.terminal.status_line_left = String::from("!not a valid command."),
            }
        }
    }
    fn save_file(&mut self) {
        if self.buffer.filename.is_empty() {
            self.terminal.status_line_left = String::from("enter a filename: ");
        } else {
            self.buffer.write_buffer_to_disk();
            self.mode = EditorModes::Normal;
        }
    }
    fn process_keypress(&mut self) {
        let c = self.terminal.read_key();
        if c > 32 {
            // self.status_line_right.push(c as char);
        }
        match self.mode {
            EditorModes::Normal => self.process_normal_mode(c),
            EditorModes::Insert => self.process_insert_mode(c),
            EditorModes::Command => self.process_command_mode(c),
            _ => eprintln!("error"),
        }
    }
}
impl Drop for Editor {
    fn drop(&mut self) {}
}
