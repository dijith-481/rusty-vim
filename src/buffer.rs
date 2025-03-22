use crate::error::Result;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines},
    path::Path,
};
pub(crate) struct TextBuffer {
    pub rows: Vec<String>,
    pub row_count: usize,
}

impl TextBuffer {
    pub(crate) fn new() -> Self {
        Self {
            rows: Vec::new(),
            row_count: 0,
        }
    }

    pub(crate) fn load_file(&mut self, filename: &String) {
        if let Ok(lines) = self.read_lines(filename) {
            for line in lines {
                self.rows.push(line.expect("line"));
                self.row_count += 1;
            }
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
