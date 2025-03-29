use crate::buffer::{self, TextBuffer};
use crate::error::{AppError, Result};
use crate::terminal::{Size, Terminal};
use std::cmp;
use std::io::{self, Read, Write, stdout};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorModes {
    Normal,
    Insert,
    Command,
    Visual,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
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
enum NormalAction {
    Move(Direction),
    ChangeMode(EditorModes),
    NewLine(Direction),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CammandModeAction {
    ChangeMode(EditorModes),
}

pub struct Editor {
    terminal: Terminal,
    buffer: TextBuffer,
    pos: Size,
    pub exit_flag: bool,
    pub mode: EditorModes,
}

impl Editor {
    pub fn new() -> Self {
        let buffer = TextBuffer::new();
        let terminal = Terminal::new().expect("error loading terminal");
        Self {
            terminal,
            buffer,
            exit_flag: false,
            pos: Size { x: 0, y: 0 },
            mode: EditorModes::Normal,
        }
    }
    pub fn run(&mut self) {
        loop {
            self.terminal.refresh_screen(&self.pos, &self.buffer);
            self.process_keypress();
            if self.exit_flag {
                break;
            }
        }
    }

    fn insert_text(&mut self, c: u8) {
        if !((c) < 32) {
            self.buffer
                .rows
                .get_mut((self.pos.y) as usize)
                .unwrap()
                .insert((self.pos.x) as usize, c as char);
            self.pos.x += 1;
        }
    }
    fn insert_newline(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                self.buffer
                    .rows
                    .insert((self.pos.y) as usize, String::new());
            }
            Direction::Down => {
                self.buffer
                    .rows
                    .insert((self.pos.y + 1) as usize, String::new());
                self.pos.y += 1;
            }
            _ => (),
        }
        self.pos.x = 0;
    }

    fn read_key(&self) -> u8 {
        let mut buffer = [0; 1];
        io::stdin().read(&mut buffer).expect("read");
        if buffer[0] == b'\x1b' {
            let mut seq = [0; 3];
            io::stdin().read(&mut seq).expect("read");
            if seq[0] == b'[' {
                match seq[1] as char {
                    'A' => return b'k',
                    'B' => return b'j',
                    'C' => return b'l',
                    'D' => return b'h',
                    _ => (),
                }
            }
            return b'\x1b';
        }
        buffer[0]
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
                self.pos.y = (self.buffer.rows.len() as i32) - 1;
            }
            Direction::StartOfNonWhiteSpace => {
                self.pos.x = self.buffer.rows[self.pos.y as usize]
                    .chars()
                    .position(|c| c != ' ')
                    .unwrap() as i32;
            }
            Direction::EndOfLine => {
                self.pos.x = self.buffer.rows[self.pos.y as usize].len() as i32 - 1;
            }
            Direction::Right => {
                if self.pos.x < self.buffer.rows[self.pos.y as usize].len() as i32 - 1 {
                    self.pos.x += 1;
                }
            }
            Direction::Up => {
                if self.pos.y > 0 {
                    if self.pos.x != 0
                        && self.pos.x == self.buffer.rows[self.pos.y as usize].len() as i32 - 1
                    {
                        self.pos.x = self.buffer.rows[self.pos.y as usize - 1].len() as i32 - 1;
                    }
                    self.pos.y -= 1;
                    if self.pos.x != 0
                        && self.pos.x >= self.buffer.rows[self.pos.y as usize].len() as i32 - 1
                    {
                        self.pos.x = self.buffer.rows[self.pos.y as usize].len() as i32 - 1;
                    }
                }
            }
            Direction::Down => {
                if self.pos.y < self.buffer.rows.len() as i32 - 1 {
                    if self.pos.x != 0
                        && self.pos.x == self.buffer.rows[self.pos.y as usize].len() as i32 - 1
                    {
                        self.pos.x = self.buffer.rows[self.pos.y as usize + 1].len() as i32 - 1;
                    }
                    self.pos.y += 1;
                    if self.pos.x != 0
                        && self.pos.x >= self.buffer.rows[self.pos.y as usize].len() as i32 - 1
                    {
                        self.pos.x = self.buffer.rows[self.pos.y as usize].len() as i32 - 1;
                    }
                }
            }
        }
    }

    fn map_key_to_action_normal_mode(&mut self, c: u8) {
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
                self.pos.x = cmp::min(
                    self.pos.x + 1,
                    self.buffer.rows[self.pos.y as usize].len() as i32,
                );
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
                self.normal_action(NormalAction::NewLine(Direction::Down));
                self.normal_action(NormalAction::ChangeMode(EditorModes::Insert));
            }
            b'O' => {
                self.normal_action(NormalAction::NewLine(Direction::Up));
                self.normal_action(NormalAction::ChangeMode(EditorModes::Insert));
            }
            _ => self.normal_action(NormalAction::Unknown),
        }
        // self.status_line_right = String::new();
    }
    fn normal_action(&mut self, action: NormalAction) {
        match action {
            NormalAction::Move(direction) => self.move_cursor(direction),
            NormalAction::ChangeMode(editormode) => self.mode = editormode,
            NormalAction::NewLine(direction) => self.insert_newline(direction),
            _ => (),
        }
    }

    fn process_insert_mode(&mut self, c: u8) {
        if c == b'\x1b' {
            self.mode = EditorModes::Normal;
            return;
        }
        self.insert_text(c);
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
            self.buffer.write_file_to_disk();
            self.mode = EditorModes::Normal;
        }
    }
    pub(crate) fn process_keypress(&mut self) {
        let c = self.read_key();
        if c > 32 {
            // self.status_line_right.push(c as char);
        }
        match self.mode {
            EditorModes::Normal => self.map_key_to_action_normal_mode(c),
            EditorModes::Insert => self.process_insert_mode(c),
            EditorModes::Command => self.process_command_mode(c),
            _ => eprintln!("error"),
        }
    }
}
impl Drop for Editor {
    fn drop(&mut self) {}
}
