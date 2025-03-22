use crate::error::Result;
use std::{io, os::fd::AsRawFd};
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
}
impl Drop for Terminal {
    fn drop(&mut self) {
        tcsetattr(io::stdin().as_raw_fd(), TCSAFLUSH, &self.0).expect("tcsetattr");
    }
}
