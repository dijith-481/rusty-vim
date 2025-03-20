use std::{
    env::{self, Args},
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Lines, Read, Write, stdout},
    option,
    os::fd::AsRawFd,
    path::Path,
    str::Utf8Chunk,
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
    numrows: i32,
    erow: Vec<String>,
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

fn editor_process_keypress(e: &mut EditorConfig) -> Option<u8> {
    let c = editor_key_read();
    // println!("{}", c as char);
    match c {
        c if c == ctrl_key(b'q') => Some(b'0'),
        c if c == b'h' || c == b'j' || c == b'k' || c == b'l' => {
            editor_move_cursor(c as char, e);
            Some(c)
        }
        _ => Some(c),
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

fn editor_draw_rows(e: &EditorConfig, abuf: &mut String) {
    for y in 0..e.screen_rows - 1 {
        if y < e.numrows {
            if let Some(line) = e.erow.get(y as usize) {
                abuf.push_str(line);
                abuf.push_str("\r\n");
            }
        } else {
            if y == e.screen_rows / 3 {
                abuf.push_str("Rust Text Editor ~ version:0.0.1\r\n");
            } else {
                abuf.push_str("~\r\n");
            }
        }
        abuf.push_str("\x1b[K");
    }
    abuf.push_str("~");
    abuf.push_str("\x1b[K");
}
fn editor_move_cursor(a: char, e: &mut EditorConfig) {
    match a {
        'h' => {
            if e.cx > 0 {
                e.cx -= 1;
            }
        }
        'l' => {
            if e.cx < e.screen_cols {
                e.cx += 1;
            }
        }
        'k' => {
            if e.cy > 0 {
                e.cy -= 1;
            }
        }
        'j' => {
            if e.cy < e.screen_rows - 1 {
                e.cy += 1;
            }
        }
        _ => println!(""),
    }
}

fn editor_refresh_screen(e: &EditorConfig) {
    let mut abuf = String::new();
    abuf.push_str("\x1b[?25l");
    abuf.push_str("\x1b[H");
    editor_draw_rows(e, &mut abuf);
    abuf.push_str(&format!("\x1b[{};{}H", e.cy + 1, e.cx + 1));
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
        cx: 4,
        cy: 5,
        numrows: 0,
        erow: Vec::new(),
    };
    Ok(e)
}

fn read_lines<P>(filename: P) -> Result<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

fn editor_open(e: &mut EditorConfig, filename: &String) {
    if let Ok(lines) = read_lines(filename) {
        for line in lines {
            e.erow.push(line.expect("line"));
            e.numrows += 1;
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut e = init_editor()?;
    editor_open(&mut e, &args[1]);
    loop {
        editor_refresh_screen(&e);
        let exitcode = editor_process_keypress(&mut e);
        if let Some(code) = exitcode {
            stdout().flush().expect("flush");
            if code == b'0' {
                write!(io::stdout(), "\x1b[2J").expect("write");
                stdout().flush().expect("flush");
                write!(io::stdout(), "\x1b[H").expect("write");
                stdout().flush().expect("flush");
                break;
            }
        }
    }
    disable_raw_mode(&e.orig_termios);
    write!(io::stdout(), "\x1b[2J").expect("write");
    stdout().flush().expect("flush");
    write!(io::stdout(), "\x1b[H").expect("write");
    stdout().flush().expect("flush");
    Ok(())
}
