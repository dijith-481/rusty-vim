use crate::buffer::TextBuffer;
use crate::error::{AppError, Result};
use crate::terminal::Terminal;
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
    StartOfLine,
    StartOfNonWhiteSpace,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NormalAction {
    Move(Direction),
    ChangeMode(EditorModes),
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CammandModeAction {
    ChangeMode(EditorModes),
}

pub struct Editor {
    terminal: Terminal,
    screen_rows: i32,
    screen_cols: i32,
    camera_x: i32,
    camera_y: i32,
    cursor_x: i32,
    cursor_y: i32,
    buffer: TextBuffer,
    pub mode: i32,
}

impl Editor {
    pub fn new(buffer: TextBuffer) -> Result<Self> {
        let terminal = Terminal::new()?;
        let (screen_rows, screen_cols) = Self::get_window_size()?;
        Ok(Self {
            terminal,
            screen_rows,
            screen_cols,
            camera_y: 0,
            camera_x: 0,
            cursor_x: 0,
            cursor_y: 0,
            buffer,
            mode: 0,
        })
    }
    fn get_window_size() -> Result<(i32, i32)> {
        write!(io::stdout(), "\x1b[999C\x1b[999B")?;
        stdout().flush().expect("err");
        Self::get_cursor_pos()
    }
    fn editor_draw_rows(&self, abuf: &mut String) {
        let camera_y_end = self.camera_y + self.screen_rows - 2;
        for y in self.camera_y..camera_y_end {
            abuf.push_str("\x1b[K");
            if y < self.buffer.row_count as i32 {
                if let Some(line) = self.buffer.rows.get(y as usize) {
                    abuf.push_str(line);
                    if y < camera_y_end - 1 {
                        abuf.push_str("\r\n");
                    } else {
                        abuf.push_str("\r");
                    }
                }
            } else {
                abuf.push_str("~\r\n");
            }
        }
    }
    fn insert_text(&mut self, c: u8) {
        if !((c) < 32) {
            self.buffer
                .rows
                .get_mut((self.camera_y + self.cursor_y) as usize)
                .unwrap()
                .insert((self.camera_x + self.cursor_x) as usize, c as char);
            self.cursor_x += 1;
        }
    }

    pub(crate) fn refresh_screen(&mut self) {
        let mut abuf = String::new();
        abuf.push_str("\x1b[?25l");
        abuf.push_str("\x1b[H");
        self.editor_draw_rows(&mut abuf);
        abuf.push_str(&format!(
            "{},{},{},{}",
            self.camera_x, self.cursor_x, self.camera_y, self.cursor_y
        ));
        abuf.push_str(&format!(
            "\x1b[{};{}H",
            self.cursor_y + 1,
            self.cursor_x + 1
        ));
        abuf.push_str("\x1b[?25h");
        write!(io::stdout(), "{}", abuf);
        stdout().flush().expect("flush");
    }
    fn get_cursor_pos() -> Result<(i32, i32)> {
        let mut response = String::new();
        write!(io::stdout(), "\x1b[6n")?;
        stdout().flush()?;
        let mut buf = [0; 1];
        loop {
            io::stdin().read(&mut buf).expect("read");
            let c = buf[0] as char;
            if c == 'R' {
                break;
            }
            response.push(c);
        }
        if !response.starts_with("\x1b[") {
            return Err(AppError::TermError);
        }
        let parts: Vec<&str> = response[2..].split(';').collect();
        let rows = parts[0].parse::<i32>().unwrap_or(0);
        let cols = parts[1].parse::<i32>().unwrap_or(0);
        Ok((rows, cols))
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
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            Direction::StartOfLine => {
                self.cursor_x = 0;
            }
            Direction::StartOfNonWhiteSpace => {
                self.cursor_x = self.buffer.rows[(self.camera_y + self.cursor_y) as usize]
                    .chars()
                    .position(|c| c != ' ')
                    .unwrap() as i32;
            }
            Direction::EndOfLine => {
                self.cursor_x =
                    self.buffer.rows[(self.camera_y + self.cursor_y) as usize].len() as i32 - 1;
            }
            Direction::Right => {
                if self.cursor_x
                    < self.buffer.rows[(self.camera_y + self.cursor_y) as usize].len() as i32 - 1
                {
                    self.cursor_x += 1;
                }
            }
            Direction::Up => {
                if self.cursor_y > 0 {
                    if self.cursor_x != 0
                        && self.cursor_x
                            == self.buffer.rows[(self.camera_y + self.cursor_y) as usize].len()
                                as i32
                                - 1
                    {
                        self.cursor_x = self.buffer.rows
                            [(self.camera_y + self.cursor_y - 1) as usize]
                            .len() as i32
                            - 1;
                    }
                    self.cursor_y -= 1;
                    if self.cursor_x != 0
                        && self.cursor_x
                            > self.buffer.rows[(self.camera_y + self.cursor_y) as usize].len()
                                as i32
                                - 1
                    {
                        self.cursor_x = self.buffer.rows[(self.camera_y + self.cursor_y) as usize]
                            .len() as i32
                            - 1;
                    }
                } else if self.camera_y > 0 {
                    self.camera_y -= 1;
                }
            }
            Direction::Down => {
                if self.cursor_y < self.screen_rows - 1 {
                    if self.cursor_x != 0
                        && self.cursor_x
                            == self.buffer.rows[(self.camera_y + self.cursor_y) as usize].len()
                                as i32
                                - 1
                    {
                        self.cursor_x = self.buffer.rows
                            [(self.camera_y + self.cursor_y + 1) as usize]
                            .len() as i32
                            - 1;
                    }
                    self.cursor_y += 1;
                    if self.cursor_x != 0
                        && self.cursor_x
                            > self.buffer.rows[(self.camera_y + self.cursor_y) as usize].len()
                                as i32
                                - 1
                    {
                        self.cursor_x = self.buffer.rows[(self.camera_y + self.cursor_y) as usize]
                            .len() as i32
                            - 1;
                    }
                } else if self.camera_y < self.buffer.row_count as i32 - self.screen_rows + 1 {
                    self.camera_y += 1;
                }
            }
        }
    }

    fn map_key_to_action_normal_mode(&self, c: u8) -> NormalAction {
        match c {
            b'h' => NormalAction::Move(Direction::Left),
            b'j' => NormalAction::Move(Direction::Down),
            b'k' => NormalAction::Move(Direction::Up),
            b'l' => NormalAction::Move(Direction::Right),
            b'$' => NormalAction::Move(Direction::EndOfLine),
            b'0' => NormalAction::Move(Direction::StartOfLine),
            b'^' => NormalAction::Move(Direction::StartOfNonWhiteSpace),
            _ => NormalAction::Unknown,
        }
    }

    fn process_normal_mode(&mut self, c: u8) {
        let action = self.map_key_to_action_normal_mode(c);
        match action {
            NormalAction::Move(direction) => self.move_cursor(direction),
            _ => (),
        }
    }
    fn process_insert_mode(&mut self, c: u8) {
        if c == b'\x1b' {
            self.mode = 0;
            return;
        }
        self.insert_text(c);
    }
    pub(crate) fn process_keypress(&mut self) -> Option<u8> {
        let c = self.read_key();
        match self.mode {
            0 => self.process_normal_mode(c),
            1 => self.process_insert_mode(c),
            _ => eprintln!("error"),
        }
        Some(c)
    }
}
impl Drop for Editor {
    fn drop(&mut self) {}
}
