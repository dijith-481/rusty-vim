use crate::error::{AppError, Result};

use std::{
    io::{self, Read, Write, stdout},
    os::fd::AsRawFd,
};
use termios::*;

pub struct Terminal(Termios);

impl Terminal {
    pub fn new() -> Result<Self> {
        let fd = io::stdin().as_raw_fd();
        let orig_termios = Self::store_term_data(fd)?;
        Self::enable_raw_mode(fd)?;
        Ok(Self(orig_termios))
    }
    fn enable_raw_mode(fd: i32) -> Result<()> {
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
    fn store_term_data(fd: i32) -> Result<Termios> {
        let termios = Termios::from_fd(fd)?;
        // tcgetattr(fd, &mut termios)?;
        Ok(termios)
    }
    pub fn get_window_size(&self) -> Result<(i32, i32)> {
        write!(io::stdout(), "\x1b[999C\x1b[999B")?;
        stdout().flush().expect("err");
        Self::get_cursor_pos()
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
}
impl Drop for Terminal {
    fn drop(&mut self) {
        tcsetattr(io::stdin().as_raw_fd(), TCSAFLUSH, &self.0).expect("tcsetattr");
    }
}
