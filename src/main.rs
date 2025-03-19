use std::{
    fmt,
    io::{self, Read, Write, stdout},
    os::fd::AsRawFd,
};
use termios::*;

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

struct EditorConfig {
    orig_termios: Termios,
    screen_rows: i32,
    screen_cols: i32,
    cx: i32,
    cy: i32,
}

const fn ctrl_key(c: u8) -> u8 {
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
        c if c == ctrl_key(b'q') => b'0',
        _ => c,
    }
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

fn get_window_size() -> Result<(i32, i32)> {
    write!(io::stdout(), "\x1b[999C\x1b[999B")?;
    stdout().flush().expect("err");
    get_cursor_pos()
}

fn editor_draw_rows(rows: i32, abuf: &mut String) {
    for y in 0..rows - 1 {
        if y == rows / 3 {
            abuf.push_str("Rust Text Editor ~ version:0.0.1\r\n");
        } else {
            abuf.push_str("~\r\n");
        }
        abuf.push_str("\x1b[K");
    }
    abuf.push_str("~");
    abuf.push_str("\x1b[K");
}
fn editor_refresh_screen(rows: i32) {
    let mut abuf = String::new();
    abuf.push_str("\x1b[?25l");
    abuf.push_str("\x1b[H");
    editor_draw_rows(rows, &mut abuf);
    abuf.push_str("\x1b[H");
    abuf.push_str("\x1b[?25h");
    write!(io::stdout(), "{}", abuf);
    stdout().flush().expect("flush");
}

fn init_editor() -> Result<EditorConfig> {
    let fd = io::stdin().as_raw_fd();
    let mut orig_termios = Termios::from_fd(fd)?;
    tcgetattr(fd, &mut orig_termios)?;
    enable_raw_mode();
    let (screen_rows, screen_cols) = get_window_size()?;
    let e = EditorConfig {
        orig_termios,
        screen_rows,
        screen_cols,
        cx: 0,
        cy: 0,
    };
    Ok(e)
}
fn main() -> Result<()> {
    let e = init_editor()?;
    loop {
        editor_refresh_screen(e.screen_rows);
        let exitcode = editor_process_keypress();
        if exitcode == b'0' {
            write!(io::stdout(), "\x1b[2J").expect("write");
            stdout().flush().expect("flush");
            write!(io::stdout(), "\x1b[H").expect("write");
            stdout().flush().expect("flush");
            break;
        }
    }
    disable_raw_mode(&e.orig_termios);
    write!(io::stdout(), "\x1b[2J").expect("write");
    stdout().flush().expect("flush");
    write!(io::stdout(), "\x1b[H").expect("write");
    stdout().flush().expect("flush");
    Ok(())
}
