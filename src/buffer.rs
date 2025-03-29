use crate::error::Result;
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
    pub fn write_file_to_disk(&self) {
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
}
