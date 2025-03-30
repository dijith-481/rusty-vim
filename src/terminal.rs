use crate::{
    buffer::TextBuffer,
    editor::EditorModes,
    error::{AppError, Result},
};
use std::{
    io::{self, Read, Write, stdout},
    os::fd::AsRawFd,
};
use termios::*;

pub struct Position {
    pub x: usize,
    pub y: usize,
}
impl Position {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}
pub struct Terminal {
    termios: Termios,
    line_no_digits: usize,
    pub size: Position,
    pub camera: Position,
    pub cursor: Position,
    pub status_line_left: String,
    pub status_line_right: String,
    cursor_type: CursorType,
}
enum CursorType {
    Ibeam,
    Block,
}

impl Terminal {
    pub fn new(buffer_len: usize) -> Result<Self> {
        let line_no_digits = buffer_len.checked_ilog10().unwrap_or_else(|| 0) as usize + 1;
        let fd = io::stdin().as_raw_fd();
        let mut terminal = Self {
            line_no_digits,
            termios: Termios::from_fd(fd)?,
            size: Position { x: 0, y: 0 },
            camera: Position { x: 0, y: 0 },
            cursor: Position { x: 0, y: 0 },
            status_line_right: String::new(),
            status_line_left: String::new(),
            cursor_type: CursorType::Block,
        };

        Self::enable_raw_mode(fd)?;
        terminal.size = Self::get_window_size(&terminal)?;
        Ok(terminal)
    }
    fn enable_raw_mode(fd: i32) -> Result<()> {
        write!(io::stdout(), "\x1b[?1049h").expect("write");
        stdout().flush().expect("flush");

        let mut termios = Termios::from_fd(fd)?;
        termios.c_iflag &= !(INPCK | ISTRIP | BRKINT | IXON | ICRNL);
        termios.c_oflag &= !(OPOST);
        termios.c_cflag |= CS8;
        termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        termios.c_cc[VMIN] = 0;
        termios.c_cc[VTIME] = 1;

        tcsetattr(fd, TCSAFLUSH, &termios)?;
        Ok(())
    }
    fn get_window_size(&self) -> Result<Position> {
        write!(io::stdout(), "\x1b[999C\x1b[999B")?;
        stdout().flush()?;
        Self::get_cursor_pos()
    }
    fn get_cursor_pos() -> Result<Position> {
        let mut response = String::new();
        write!(io::stdout(), "\x1b[6n")?;
        stdout().flush()?;
        let mut buf = [0; 1];
        loop {
            io::stdin().read(&mut buf)?;
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
        let rows = parts[0].parse::<usize>()?;
        let cols = parts[1].parse::<usize>()?;
        Ok(Position { x: cols, y: rows })
    }
    fn editor_draw_rows(&self, buffer: &TextBuffer, abuf: &mut String) {
        let camera_y_end = self.camera.y + self.size.y - 1;
        for y in self.camera.y..camera_y_end {
            abuf.push_str("\x1b[K"); //clears from current position to end of line
            // if y < buffer.rows.len() {
            if let Some(line) = buffer.rows.get(y as usize) {
                abuf.push_str(&format!("{:>1$} |", y + 1, self.line_no_digits));
                abuf.push_str(line);
                abuf.push_str("\r\n");
            }
            // }
            else {
                abuf.push_str("~\r\n");
            }
        }
        abuf.push_str("\x1b[K"); //clears from current position to end of line
        abuf.push_str(&self.status_line_left);
        abuf.push_str(&format!(
            "\x1b[{};{}H",
            self.size.y,
            self.size.x - self.status_line_right.len()
        ));
        abuf.push_str(&self.status_line_right);
        abuf.push_str("\r");
    }
    pub fn refresh_screen(&mut self, buffer: &TextBuffer, pos: &Position) {
        self.cursor.x = pos.x % self.size.x + self.line_no_digits + 2;
        // self.camera.x = pos.x / self.size.x;
        self.cursor.y = pos.y.saturating_sub(self.camera.y);
        if self.cursor.y >= self.size.y - 1 {
            self.camera.y += self.cursor.y.saturating_sub(self.size.y - 2);
            self.cursor.y = self.size.y - 2;
        } else if self.cursor.y == 0 && self.camera.y != pos.y {
            self.camera.y = self
                .camera
                .y
                .saturating_sub(self.camera.y.saturating_sub(pos.y));
        }
        let mut abuf = String::new();
        let cursor_type = self.get_cursor_code();
        abuf.push_str("\x1b[?25l"); //hide cursor
        abuf.push_str("\x1b[H"); //cursor upperleft
        abuf.push_str(&format!("{}", cursor_type)); //cursor upperleft
        self.editor_draw_rows(buffer, &mut abuf);
        // abuf.push_str(&format!(
        //     "{},{},{},{},{},{}",
        //     self.camera_x,
        //     self.cursor_x,
        //     self.camera_y,
        //     self.cursor_y,
        //     self.screen_rows,
        //     self.buffer.rows[(self.camera_y + self.cursor_y) as usize].len() as i32 - 1,
        // ));
        abuf.push_str(&format!(
            "\x1b[{};{}H",
            self.cursor.y + 1,
            self.cursor.x + 1
        ));
        abuf.push_str("\x1b[?25h");
        write!(io::stdout(), "{}", abuf);
        stdout().flush().expect("flush");
    }
    pub fn read_key(&self) -> u8 {
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
    pub fn middle_screen_pos(&self) -> usize {
        self.cursor.y + self.size.y / 2 - 1
    }
    pub fn top_screen_pos(&self) -> usize {
        self.cursor.y
    }
    pub fn bottom_screen_pos(&self) -> usize {
        self.cursor.y + self.size.y - 2
    }
    fn get_cursor_code(&self) -> &str {
        match self.cursor_type {
            CursorType::Block => "\x1b[2 q",
            CursorType::Ibeam => "\x1b[6 q",
        }
    }
    pub fn change_cursor(&mut self, mode: EditorModes) {
        match mode {
            EditorModes::Insert => {
                self.cursor_type = CursorType::Ibeam;
            }
            _ => self.cursor_type = CursorType::Block,
        }
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        tcsetattr(io::stdin().as_raw_fd(), TCSAFLUSH, &self.termios).expect("tcsetattr");
        // write!(io::stdout(), "\x1b[?1049l").expect("write");
        stdout().flush().expect("flush");
    }
}
