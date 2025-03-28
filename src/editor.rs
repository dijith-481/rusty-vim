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
    screen_rows: i32,
    screen_cols: i32,
    camera_x: i32,
    camera_y: i32,
    cursor_x: i32,
    cursor_y: i32,
    buffer: TextBuffer,
    pub exitFlag: bool,
    pub mode: EditorModes,
}

impl Editor {
    pub fn new(buffer: TextBuffer) -> Result<Self> {
        let terminal = Terminal::new()?;
        let (screen_rows, screen_cols) = terminal.get_window_size()?;
        Ok(Self {
            terminal,
            screen_rows,
            screen_cols,
            camera_y: 0,
            camera_x: 0,
            cursor_x: 0,
            cursor_y: 0,
            exitFlag: false,
            buffer,
            mode: EditorModes::Normal,
        })
    }

    fn editor_draw_rows(&self, abuf: &mut String) {
        let camera_y_end = self.camera_y + self.screen_rows;
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
    fn insert_newline(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                self.buffer
                    .rows
                    .insert((self.camera_y + self.cursor_y) as usize, String::new());
            }
            Direction::Down => {
                self.buffer
                    .rows
                    .insert((self.camera_y + self.cursor_y + 1) as usize, String::new());
                self.cursor_y += 1;
            }
            _ => (),
        }
        self.cursor_x = 0;
    }

    pub(crate) fn refresh_screen(&mut self) {
        let mut abuf = String::new();
        abuf.push_str("\x1b[?25l");
        abuf.push_str("\x1b[H");
        self.editor_draw_rows(&mut abuf);
        abuf.push_str(&format!(
            "{},{},{},{},{},{}",
            self.camera_x,
            self.cursor_x,
            self.camera_y,
            self.cursor_y,
            (self.camera_y + self.cursor_y) as usize,
            self.buffer.rows[(self.camera_y + self.cursor_y) as usize].len() as i32 - 1,
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
        let buffer_pos_y = (self.camera_y + self.cursor_y) as usize;
        let buffer_pos_x = (self.camera_x + self.cursor_x) as usize;
        match direction {
            Direction::Left => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            Direction::StartOfLine => {
                self.cursor_x = 0;
            }
            Direction::EndOfRows => {
                self.cursor_y = self.screen_rows - 1;
                self.camera_y = self.buffer.rows.len() as i32 - self.screen_rows;
            }
            Direction::StartOfNonWhiteSpace => {
                self.cursor_x = self.buffer.rows[buffer_pos_y]
                    .chars()
                    .position(|c| c != ' ')
                    .unwrap() as i32;
            }
            Direction::EndOfLine => {
                self.cursor_x = self.buffer.rows[buffer_pos_y].len() as i32 - 1;
            }
            Direction::Right => {
                if self.cursor_x < self.buffer.rows[buffer_pos_y].len() as i32 - 1 {
                    self.cursor_x += 1;
                }
            }
            Direction::Up => {
                if self.cursor_y > 0 {
                    if self.cursor_x != 0
                        && self.cursor_x == self.buffer.rows[buffer_pos_y].len() as i32 - 1
                    {
                        self.cursor_x = self.buffer.rows[buffer_pos_y - 1].len() as i32 - 1;
                    }
                    self.cursor_y -= 1;

                    if self.cursor_x > self.buffer.rows[buffer_pos_y - 1].len() as i32 - 1 {
                        self.cursor_x = self.buffer.rows[buffer_pos_y - 1].len() as i32 - 1;
                    }
                } else if self.camera_y > 0 {
                    self.camera_y -= 1;
                }
            }
            Direction::Down => {
                if self.cursor_y < self.screen_rows - 1 {
                    if self.cursor_x != 0
                        && self.cursor_x == self.buffer.rows[buffer_pos_y].len() as i32 - 1
                    {
                        self.cursor_x = self.buffer.rows[buffer_pos_y + 1].len() as i32 - 1;
                    }
                    self.cursor_y += 1;
                    if self.cursor_x > self.buffer.rows[buffer_pos_y + 1].len() as i32 - 1 {
                        self.cursor_x = self.buffer.rows[buffer_pos_y + 1].len() as i32 - 1;
                    }
                } else if self.camera_y < self.buffer.row_count as i32 - self.screen_rows + 1 {
                    self.camera_y += 1;
                }
            }
        }
    }

    fn map_key_to_action_normal_mode(&mut self, c: u8) -> Vec<NormalAction> {
        match c {
            b'h' => vec![NormalAction::Move(Direction::Left)],
            b'j' => vec![NormalAction::Move(Direction::Down)],
            b'k' => vec![NormalAction::Move(Direction::Up)],
            b'l' => vec![NormalAction::Move(Direction::Right)],
            b'$' => vec![NormalAction::Move(Direction::EndOfLine)],
            b'0' => vec![NormalAction::Move(Direction::StartOfLine)],
            b'^' => vec![NormalAction::Move(Direction::StartOfNonWhiteSpace)],
            b'i' => vec![NormalAction::ChangeMode(EditorModes::Insert)],
            b'a' => {
                self.cursor_x += 1;
                vec![NormalAction::ChangeMode(EditorModes::Insert)]
            }
            b'A' => {
                let mut temp = vec![NormalAction::Move(Direction::EndOfLine)];
                self.cursor_x += 1;
                temp.push(NormalAction::ChangeMode(EditorModes::Insert));
                temp
            }
            b'I' => vec![
                NormalAction::Move(Direction::StartOfNonWhiteSpace),
                NormalAction::ChangeMode(EditorModes::Insert),
            ],

            b'o' => vec![
                NormalAction::NewLine(Direction::Down),
                NormalAction::ChangeMode(EditorModes::Insert),
            ],
            b'O' => vec![
                NormalAction::NewLine(Direction::Up),
                NormalAction::ChangeMode(EditorModes::Insert),
            ],
            b'G' => vec![NormalAction::Move(Direction::EndOfRows)],
            b':' => vec![NormalAction::ChangeMode(EditorModes::Command)],
            _ => vec![NormalAction::Unknown],
        }
    }

    fn process_normal_mode(&mut self, c: u8) {
        let actions = self.map_key_to_action_normal_mode(c);
        for action in actions {
            match action {
                NormalAction::Move(direction) => self.move_cursor(direction),
                NormalAction::ChangeMode(editormode) => self.mode = editormode,
                NormalAction::NewLine(direction) => self.insert_newline(direction),
                _ => (),
            }
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
            self.mode = EditorModes::Normal;
            return;
        }
        if c == b'q' {
            self.exitFlag = true;
            return;
        }
        if c == b'w' {
            // self.exitFlag = true;
            return;
        }
    }
    pub(crate) fn process_keypress(&mut self) {
        let c = self.read_key();
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
