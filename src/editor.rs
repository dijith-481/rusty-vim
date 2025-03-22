use crate::buffer::TextBuffer;
use crate::error::{AppError, Result};
use crate::terminal::Terminal;
use std::io::{self, Read, Write, stdout};
pub struct Editor {
    terminal: Terminal,
    screen_rows: i32,
    screen_cols: i32,
    camera_x: i32,
    camera_y: i32,
    cursor_x: i32,
    cursor_y: i32,
    buffer: TextBuffer,
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
        })
    }
    fn get_window_size() -> Result<(i32, i32)> {
        write!(io::stdout(), "\x1b[999C\x1b[999B")?;
        stdout().flush().expect("err");
        Self::get_cursor_pos()
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
    pub(crate) fn refresh_screen(&mut self) {
        let mut abuf = String::new();
        abuf.push_str("\x1b[?25l");
        abuf.push_str("\x1b[H");
        self.editor_draw_rows(&mut abuf);
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
                    _ => return b'0',
                }
            }
            return b'\x1b';
        }
        buffer[0]
    }
    fn move_cursor(&mut self, a: char) {
        match a {
            'h' => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            '$' => {
                self.cursor_x =
                    self.buffer.rows[(self.camera_y + self.cursor_y) as usize].len() as i32 - 1;
            }
            'l' => {
                if self.cursor_x
                    < self.buffer.rows[(self.camera_y + self.cursor_y) as usize].len() as i32 - 1
                {
                    self.cursor_x += 1;
                }
            }
            'k' => {
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
            'j' => {
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
            _ => println!(""),
        }
    }
    pub(crate) fn process_keypress(&mut self) -> Option<u8> {
        let c = self.read_key();
        match c {
            c if c == b'q' => Some(b'0'),
            c if c == b'h' || c == b'j' || c == b'k' || c == b'l' || c == b'$' => {
                self.move_cursor(c as char);
                Some(c)
            }
            _ => Some(c),
        }
    }
}
impl Drop for Editor {
    fn drop(&mut self) {}
}
