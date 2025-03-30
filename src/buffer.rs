use crate::{
    error::Result,
    file::{self, load_file, write_file_to_disk},
    terminal::Position,
};
use std::env;

pub struct TextBuffer {
    pub filename: String,
    pub rows: Vec<String>,
}

impl TextBuffer {
    pub fn new() -> Result<Self> {
        let args: Vec<String> = env::args().collect();
        let rows: Vec<String>;
        let filename: String;
        if args.len() > 1 {
            filename = args[1].clone();
            rows = load_file(&filename)?;
        } else {
            filename = String::new();
            rows = Vec::new();
        }
        Ok(Self { filename, rows })
    }

    pub fn write_buffer_to_disk(&self) -> Result<()> {
        write_file_to_disk(&self.filename, &self.rows)?;
        Ok(())
    }

    pub fn insert_char(&mut self, c: u8, pos: &mut Position) {
        if self.rows.is_empty() {
            let row = String::from(c as char);
            self.rows.push(row);
            return;
        }
        if let Some(row) = self.rows.get_mut(pos.y) {
            row.insert(pos.x, c as char);
        }
        pos.x += 1;
    }

    // pub fn insert_str(&mut self, str: &str, pos: &mut Position) {
    //     let row = self.rows.get_mut(pos.y).unwrap();
    //     row.insert_str(pos.x, str);
    //     pos.x += str.len();
    // }
    // pub fn append_char(&mut self, c: u8, pos: &mut Position) {
    //     let row = self.rows.get_mut(pos.y).unwrap();
    //     row.push(c as char);
    //     pos.x = row.len() - 1;
    // }
    // pub fn append_str(&mut self, str: &str, pos: &mut Position) {
    //     let row = self.rows.get_mut(pos.y).unwrap();
    //     row.push_str(str);
    //     pos.x = str.len() - 1;
    // }
    pub fn delete_char(&mut self, pos: &mut Position) {
        if let Some(row) = self.rows.get_mut(pos.y) {
            if row.is_empty() {
                return;
            }
            if is_line_full(&row, pos.x) {
                row.pop();
                pos.x = row.len().saturating_sub(1);
                return;
            }
            row.remove(pos.x);
            if pos.x >= row.len() {
                pos.x = row.len().saturating_sub(1);
            }
        }
    }
    // pub fn back_space(&mut self, pos: &mut Position) {
    //     if pos.x == 0 && pos.y > 0 {
    //         let text = self.rows.remove(pos.y);
    //         self.rows[pos.y - 1].push_str(&text);
    //         return;
    //     }
    //     pos.x -= 1;
    //     self.delete_char(pos);
    // }
    pub fn delete_row(&mut self, pos: &mut Position) {
        if self.rows.is_empty() || self.is_rows_full(pos.y) {
            return;
        };
        self.rows.remove(pos.y);
        if self.is_rows_full(pos.y) {
            pos.y = pos.y.saturating_sub(1);
        }
    }
    fn is_rows_full(&self, end: usize) -> bool {
        self.rows.len() <= end
    }

    // pub fn pop_char(&mut self, pos: &mut Position) {
    //     let row = self.rows.get_mut(pos.y).unwrap();
    //     row.pop();
    // }

    pub fn insert_newline(&mut self, pos: &mut Position) {
        if self.rows.is_empty() || self.is_rows_full(pos.y) {
            self.rows.push(String::new());
            pos.x = 0;
            return;
        }
        self.rows.insert(pos.y, String::new());
        pos.x = 0;
    }
}
fn is_line_full(line: &String, end: usize) -> bool {
    line.len() <= end
}
