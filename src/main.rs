use std::{
    io::{self, Read, Write, stdin},
    os::fd::AsRawFd,
    process::exit,
};
use termios::*;

const fn CTRL_KEY(c: u8) -> u8 {
#[derive(Debug)]
enum AppError {
    TermError,
    Io(io::Error),
}
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::TermError => write!(f, "termerror"),
            AppError::Io(e) => write!(f, "{}", e),
        }
    }
}
impl From<io::Error> for AppError {
    fn from(err: io::Error) -> AppError {
        AppError::Io(err)
    }
}

type Result<T> = std::result::Result<T, AppError>;

    c & 0x1f
}

fn disable_raw_mode(original_termios: &Termios) {
    tcsetattr(io::stdin().as_raw_fd(), TCSAFLUSH, &original_termios).expect("tcsetattr")
}
fn enable_raw_mode() {
    let fd = io::stdin().as_raw_fd();
    let mut termios = Termios::from_fd(fd).expect("termios");
    termios.c_iflag &= !(INPCK | ISTRIP | BRKINT | IXON | ICRNL);
    termios.c_oflag &= !(OPOST);
    termios.c_cflag |= CS8;
    termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    termios.c_cc[VMIN] = 0;
    termios.c_cc[VTIME] = 1;

    tcsetattr(fd, TCSAFLUSH, &termios).expect("tcsetattr");
}
fn editor_key_read() -> u8 {
    let mut buffer = [0; 1];
    while io::stdin().read(&mut buffer).expect("read") == 1 {}
    buffer[0]
}
fn editor_process_keypress() -> u8 {
    let c = editor_key_read();
    match c {
        c if c == CTRL_KEY(b'q') => b'0',
        _ => c,
    }
}
fn editor_refresh_screen() {
    write!(io::stdout(), "\x1b[2J").expect("write");
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
fn main() -> io::Result<()> {
fn get_window_size() -> Result<(i32, i32)> {
    write!(io::stdout(), "\x1b[999C\x1b[999B")?;
    stdout().flush().expect("err");
    get_cursor_pos()
}

    let fd = io::stdin().as_raw_fd();
    let mut orig_termios = Termios::from_fd(fd)?;
    tcgetattr(fd, &mut orig_termios).expect("tcgetattr");
    enable_raw_mode();
    loop {
        // editor_refresh_screen();
        let exitcode = editor_process_keypress();
        if exitcode == b'0' {
            break;
        }
    }
    disable_raw_mode(&orig_termios);
    Ok(())
}
