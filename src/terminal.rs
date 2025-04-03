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
    pub size: Position,
    pub camera: Position,
    pub cursor: Position,
    line_no_digits: usize,
    pub status_line_left: String,
    pub command_line: String,
    pub status_line_right: String,
    cursor_type: CursorType,
    is_start_first_time: bool,
}
enum CursorType {
    Ibeam,
    Block,
}

impl Terminal {
    pub fn new(buffer_len: usize, filename: &str) -> Result<Self> {
        let line_no_digits = Self::get_line_no_padding(buffer_len);
        let fd = io::stdin().as_raw_fd();
        let mut terminal = Self {
            line_no_digits,
            command_line: String::new(),
            is_start_first_time: true,
            termios: Termios::from_fd(fd)?,
            size: Position { x: 0, y: 0 },
            camera: Position { x: 0, y: 0 },
            cursor: Position { x: 0, y: 0 },
            status_line_right: String::new(),
            status_line_left: String::from(filename),
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
    fn render_start_page(&self, abuf: &mut String) {
        abuf.push_str("\x1b[38;2;129;161;193m");
        abuf.push_str("\x1b[48;2;46;52;64m");
        abuf.push_str("\x1b[2J");
        abuf.push_str("\r\n");
        abuf.push_str("\r\n");
        abuf.push_str("\r\n");
        abuf.push_str(
            "\x1b[K     ██████╗ ██╗   ██╗ ██████╗████████╗██╗   ██╗   ██╗   ██╗██╗███╗   ███╗\r\n",
        );
        abuf.push_str(
            "\x1b[K     ██╔══██╗██║   ██║██╔════╝╚══██╔══╝╚██╗ ██╔╝   ██║   ██║██║████╗ ████║\r\n",
        );
        abuf.push_str(
            "\x1b[K     ██████╔╝██║   ██║╚█████╗    ██║    ╚████╔╝    ╚██╗ ██╔╝██║██╔████╔██║\r\n",
        );
        abuf.push_str(
            "\x1b[K     ██╔══██╗██║   ██║ ╚═══██╗   ██║     ╚██╔╝      ╚████╔╝ ██║██║╚██╔╝██║\r\n",
        );
        abuf.push_str(
            "\x1b[K     ██║  ██║╚██████╔╝██████╔╝   ██║      ██║        ╚██╔╝  ██║██║ ╚═╝ ██║\r\n",
        );
        abuf.push_str(
            "\x1b[K     ╚═╝  ╚═╝ ╚═════╝ ╚═════╝    ╚═╝      ╚═╝         ╚═╝   ╚═╝╚═╝     ╚═╝\r\n",
        );
        abuf.push_str("\r\n");
        // abuf.push_str("]x1b[K                 Rusty-VIM               \r\n");
        abuf.push_str(
            "\x1b[K                Implementation of Vim-like text-editor in rust            \r\n",
        );
        abuf.push_str(
            "\x1b[K                              version 0.0.1                               \r\n",
        );

        abuf.push_str(
            "\x1b[K                             By Dijith Dinesh                             \r\n",
        );
        abuf.push_str("\r\n");
        abuf.push_str(
            "\x1b[K                   type  :q<Enter>       to exit                          \r\n",
        );
        abuf.push_str(
            "\x1b[K                   type  i               to enter insert mode             \r\n",
        );
    }
    fn render_rows(&mut self, buffer: &TextBuffer, abuf: &mut String) {
        if self.is_start_first_time && buffer.rows.is_empty() {
            self.render_start_page(abuf);
            abuf.push_str("\x1b[999B");
            self.render_command_line(abuf);
            abuf.push_str("\r\x1b[A");
            self.render_status_line(abuf, &buffer.pos);
            return;
        }
        self.is_start_first_time = false;
        self.line_no_digits = Self::get_line_no_padding(buffer.rows.len());

        let camera_y_end = self.camera.y + self.size.y - 2;
        for y in self.camera.y..camera_y_end {
            if let Some(line) = buffer.rows.get(y as usize) {
                if self.cursor.y + self.camera.y == y {
                    abuf.push_str("\x1b[48;2;76;86;106m");
                    abuf.push_str("\x1b[38;2;129;161;193m");
                } else {
                    abuf.push_str("\x1b[48;2;46;52;64m");
                    abuf.push_str("\x1b[38;2;76;86;106m");
                }
                abuf.push_str("\x1b[K"); //clears from current position to end of line
                abuf.push_str("\x1b[48;2;46;52;64m");
                abuf.push_str("\r");
                abuf.push_str(&format!("{:>1$} |", y + 1, self.line_no_digits,));
                abuf.push_str("\x1b[38;2;216;222;233m");
                if self.cursor.y + self.camera.y == y {
                    abuf.push_str("\x1b[48;2;76;86;106m");
                } else {
                    abuf.push_str("\x1b[48;2;46;52;64m");
                }
                abuf.push_str(line);
                abuf.push_str("\r\n");
            } else {
                abuf.push_str("\x1b[48;2;46;52;64m");
                abuf.push_str("\x1b[K"); //clears from current position to end of line
                abuf.push_str("\x1b[38;2;76;86;106m");
                abuf.push_str("~\r\n");
            }
        }
        self.render_status_line(abuf, &buffer.pos);
        abuf.push_str("\r\n");
        self.render_command_line(abuf);
        // abuf.push_str(&self.status_line_right);
    }
    fn render_status_line(&self, abuf: &mut String, pos: &Position) {
        abuf.push_str("\x1b[K"); //clears from current position to end of line
        let spaces = " ".repeat(self.size.x);
        abuf.push_str("\x1b[38;2;236;239;244m");
        abuf.push_str("\x1b[48;2;76;86;106m");
        abuf.push_str(&spaces);
        abuf.push_str("\r\x1b[38;2;46;52;64m");
        abuf.push_str("\x1b[48;2;129;161;193m");
        abuf.push_str(&self.status_line_left);
        abuf.push_str(&format!("\r\x1b[{}C", self.size.x - 8));
        let spaces = " ".repeat(8);
        abuf.push_str(&spaces);
        abuf.push_str(&format!("\r\x1b[{}C", self.size.x - 7));
        abuf.push_str(&format!("{}:{}", pos.y, pos.x));
    }
    fn render_command_line(&self, abuf: &mut String) {
        abuf.push_str("\r");

        abuf.push_str("\x1b[48;2;46;52;64m");
        abuf.push_str("\x1b[38;2;216;222;233m");
        abuf.push_str("\x1b[K"); //clears from current position to end of line
        abuf.push_str(&self.command_line);
        abuf.push_str(&format!(
            "\x1b[{};{}H",
            self.size.y,
            self.size.x - self.status_line_right.len()
        ));
        abuf.push_str(&self.status_line_right);
    }
    fn render_cursor_position(&mut self, pos: &Position, abuf: &mut String) {
        let bottom_ui_size = 2;
        let left_ui_size = self.line_no_digits + 2;
        self.cursor.x = pos.x % self.size.x + left_ui_size;
        self.cursor.y = pos.y.saturating_sub(self.camera.y);
        if self.cursor.y >= self.size.y - bottom_ui_size {
            self.camera.y += self
                .cursor
                .y
                .saturating_sub(self.size.y - bottom_ui_size - 1);
            self.cursor.y = self.size.y - bottom_ui_size - 1;
        } else if self.cursor.y == 0 && self.camera.y != pos.y {
            self.camera.y = self
                .camera
                .y
                .saturating_sub(self.camera.y.saturating_sub(pos.y));
        }
        let cursorcode = self.get_cursor_code();
        abuf.push_str("\x1b[?25l"); //hide cursor
        abuf.push_str("\x1b[H"); //cursor upperleft
        abuf.push_str(&format!("{}", cursorcode)); //cursor upperleft
    }

    fn update_mouse_pos(&self, abuf: &mut String) {
        abuf.push_str(&format!(
            "\x1b[{};{}H",
            self.cursor.y + 1,
            self.cursor.x + 1
        ));
        abuf.push_str("\x1b[?25h");
    }

    pub fn refresh_screen(&mut self, buffer: &TextBuffer) -> Result<()> {
        let mut abuf = String::new();
        self.render_cursor_position(&buffer.pos, &mut abuf);
        self.render_rows(buffer, &mut abuf);
        self.update_mouse_pos(&mut abuf);
        write!(io::stdout(), "{}", abuf)?;
        stdout().flush()?;
        Ok(())
    }

    pub fn read_key(&self) -> Result<u8> {
        let mut buffer = [0; 1];
        io::stdin().read(&mut buffer)?;
        let key;
        if buffer[0] == b'\x1b' {
            key = self.handle_other_keys()?;
        } else {
            key = buffer[0];
        }
        Ok(key)
    }

    fn handle_other_keys(&self) -> Result<u8> {
        let mut seq = [0; 3];
        io::stdin().read(&mut seq)?;
        let key: u8;
        if seq[0] == b'[' {
            match seq[1] as char {
                'A' => key = b'k',
                'B' => key = b'j',
                'C' => key = b'l',
                'D' => key = b'h',
                _ => key = b'\x1b',
            }
        } else {
            key = b'\x1b';
        }
        Ok(key)
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
    fn get_line_no_padding(buffer_len: usize) -> usize {
        buffer_len.checked_ilog10().unwrap_or_else(|| 0) as usize + 1
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        tcsetattr(io::stdin().as_raw_fd(), TCSAFLUSH, &self.termios).expect("tcsetattr");
        // write!(io::stdout(), "\x1b[?1049l").expect("write");
        stdout().flush().expect("flush");
    }
}
