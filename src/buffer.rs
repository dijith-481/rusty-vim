use crate::{editor::Direction, error::Result, terminal::Size};
use std::{
    env::{self},
    fs::File,
    io::{self, BufRead, BufReader, Lines, Write},
    path::Path,
};
pub(crate) struct TextBuffer {
    pub filename: String,
    pub rows: Vec<String>,
}

impl TextBuffer {
    pub(crate) fn new() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut buffer = Self {
            filename: String::new(),
            rows: Vec::new(),
        };
        if args.len() > 1 {
            let filename = args[1].clone();
            buffer.filename = filename;
            TextBuffer::load_file(&mut buffer);
        } else {
            buffer.rows.push(String::new());
        }
        buffer
    }

    fn load_file(&mut self) {
        if let Ok(lines) = self.read_lines(&self.filename) {
            for line in lines {
                self.rows.push(line.expect("line"));
            }
        }
    }
    pub fn write_buffer_to_disk(&self) {
        let long_string = self.rows.join("\n");
        if !self.filename.is_empty() {
            let mut file = File::create(&self.filename).expect("error saving file");
            file.write_all(long_string.as_bytes()).expect("error");
        } else {
        }
    }
    fn read_lines<P>(&self, filename: P) -> Result<Lines<BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    pub fn insert_char(&mut self, c: u8, pos: &mut Size) {
        self.rows.get_mut(pos.y).unwrap().insert(pos.x, c as char);
        pos.x += 1;
    }
    pub fn insert_str(&mut self, str: &str, pos: &mut Size) {
        let row = self.rows.get_mut(pos.y).unwrap();
        row.insert_str(pos.x, str);
        pos.x += str.len();
    }
    pub fn append_char(&mut self, c: u8, pos: &mut Size) {
        let row = self.rows.get_mut(pos.y).unwrap();
        row.push(c as char);
        pos.x = row.len() - 1;
    }
    pub fn append_str(&mut self, str: &str, pos: &mut Size) {
        let row = self.rows.get_mut(pos.y).unwrap();
        row.push_str(str);
        pos.x = str.len() - 1;
    }
    pub fn delete_char(&mut self, pos: &mut Size) {
        let row = self.rows.get_mut(pos.y).unwrap();
        if row.len() == 0 {
            return;
        }
        if !pos.x == 0 {
            row.remove(pos.x);
            return;
        }
        if pos.y > 0 {
            let text = self.rows.remove(pos.y);
            self.rows[pos.y - 1].push_str(&text);
        }
    }
    pub fn delete_row(&mut self, pos: &mut Size) {
        if self.rows.len() == 0 {
            return;
        };
        self.rows.remove(pos.y);
    }

    pub fn pop_char(&mut self, pos: &mut Size) {
        let row = self.rows.get_mut(pos.y).unwrap();
        row.pop();
    }

    pub fn insert_newline(&mut self, pos: &mut Size) {
        self.rows.insert(pos.y, String::new());
        pos.x = 0;
    }
}
