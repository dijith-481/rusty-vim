use std::{
    io::{self, Read, stdin},
    os::fd::AsRawFd,
};
use termios::*;

fn disable_raw_mode(original_termios: &Termios) -> io::Result<()> {
    tcsetattr(io::stdin().as_raw_fd(), TCSAFLUSH, &original_termios)
}
fn enable_raw_mode() -> io::Result<()> {
    let fd = io::stdin().as_raw_fd();
    let mut termios = Termios::from_fd(fd)?;
    termios.c_lflag &= !(ECHO | ICANON);
    tcsetattr(fd, TCSAFLUSH, &termios)?;
    Ok(())
}
fn main() -> io::Result<()> {
    let fd = io::stdin().as_raw_fd();
    let mut orig_termios = Termios::from_fd(fd)?;
    tcgetattr(fd, &mut orig_termios)?;
    enable_raw_mode()?;
    let mut buffer = [0; 1];
    while io::stdin().read(&mut buffer)? == 1 && buffer[0] != b'q' {
        let c = buffer[0];
        if c.is_ascii_control() {
            println!("{}", c);
        } else {
            println!("{}{}", c, c as char)
        }
    }
    disable_raw_mode(&orig_termios);
    Ok(())
}
