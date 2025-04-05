use crate::{
    error::{AppError, FileError},
    file::{load_file, write_file_to_disk},
    insertmode::InsertType,
    normalmode::motions::Motion,
    terminal::Position,
};
use std::{
    collections::HashMap,
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
    usize,
};

pub struct TextBuffer {
    pub filename: Option<String>,
    pub modified_time: Duration,
    pub rows: Vec<String>,
    pub pos: Position,
    x_end: usize,
    pub is_changed: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CharClass {
    Keyword,
    WhiteSpace,
    Other,
}

impl TextBuffer {
    pub fn load_buffers(
        args: Vec<String>,
        buff_vec: &mut Vec<usize>,
    ) -> Result<HashMap<usize, TextBuffer>, AppError> {
        let mut count: usize = 1000;
        let mut buffers = HashMap::new();

        if args.len() > 1 {
            for filename in args.iter().skip(1) {
                let buffer = TextBuffer::new(Some(filename.clone()))?;
                buffers.insert(count, buffer);
                buff_vec.push(count);
                count += 1;
            }
        } else {
            let buffer = Self::create_empty_buffer().unwrap();
            buff_vec.push(count);
            buffers.insert(count, buffer);
        }

        Ok(buffers)
    }

    pub fn get_modified_time(filename: &String) -> Duration {
        let modified_time: Duration;
        match fs::metadata(filename) {
            Ok(metadata) => {
                if let Ok(modified) = metadata.modified() {
                    modified_time = modified.duration_since(UNIX_EPOCH).unwrap();
                } else {
                    let now = SystemTime::now();
                    modified_time = now.duration_since(UNIX_EPOCH).unwrap();
                }
            }
            Err(_) => {
                let now = SystemTime::now();
                modified_time = now.duration_since(UNIX_EPOCH).unwrap();
            }
        }
        modified_time
    }

    fn create_empty_buffer() -> Result<TextBuffer, AppError> {
        let now = SystemTime::now();
        let modified_time = now.duration_since(UNIX_EPOCH).unwrap();
        Ok(TextBuffer {
            is_changed: false,
            filename: None,
            modified_time,
            x_end: 0,
            rows: Vec::new(),
            pos: Position::new(),
        })
    }

    fn create_file_buffer(filename: String) -> Result<TextBuffer, AppError> {
        let modified_time = Self::get_modified_time(&filename);
        let rows: Vec<String>;
        let pos = Position::new();
        rows = load_file(&filename)?;
        Ok(Self {
            is_changed: false,
            modified_time,
            filename: Some(filename),
            x_end: 0,
            rows,
            pos,
        })
    }

    pub fn new(filename: Option<String>) -> Result<Self, AppError> {
        match filename {
            None => Self::create_empty_buffer(),
            Some(name) => Self::create_file_buffer(name),
        }
    }

    pub fn insert_char(&mut self, c: u8) {
        if self.rows.is_empty() {
            let row: String;
            if c == 9 {
                row = String::from("    ");
                self.pos.x += 4;
            } else {
                row = String::from(c as char);
                self.pos.x += 1;
            }
            self.rows.push(row);
            return;
        }
        if let Some(row) = self.rows.get_mut(self.pos.y) {
            if c == 9 {
                row.insert_str(self.pos.x, "    ");
                self.pos.x += 4;
            } else {
                row.insert(self.pos.x, c as char);
                self.pos.x += 1;
            }
        }
    }

    fn is_valid_y(&self, y: usize) -> bool {
        y < self.rows.len()
    }

    fn get_current_line(&self) -> Option<&String> {
        self.rows.get(self.pos.y)
    }

    fn is_valid_x(&self, x: usize) -> bool {
        if self.is_valid_y(self.pos.y) {
            return x < self.rows[self.pos.y].len();
        }
        x < self.rows.last().map_or(0, |row| row.len())
    }

    pub fn set_x_or(&mut self, default: usize, x: usize) {
        if self.is_valid_x(x) {
            self.pos.x = x;
        } else {
            self.pos.x = default;
        }
    }

    fn set_y_or(&mut self, default: usize, y: usize) {
        if self.is_valid_y(y) {
            self.pos.y = y
        } else {
            self.pos.y = default;
        }
    }

    pub fn split_line(&mut self) {
        if self.rows.is_empty() {
            self.rows.push(String::new());
            self.pos.y += 1;
            self.rows.push(String::new());
            return;
        }
        if self.pos.x > self.end_of_line() {
            self.pos.y += 1;
            self.rows.insert(self.pos.y, String::new());
            self.pos.x = 0;
            return;
        }
        if let Some(line) = self.rows.get_mut(self.pos.y) {
            let split_string = line.get(self.pos.x..).map(|s| s.to_string());
            if let Some(split_string) = split_string {
                let len = line.len();
                line.drain(self.pos.x..len);
                if self.pos.y < self.end_of_file() {
                    let whitespace = self.first_non_white_space();
                    self.pos.x = whitespace;
                    let mut new_split_string = String::new();
                    for _ in 0..whitespace {
                        new_split_string.push(' ');
                    }
                    new_split_string.push_str(&split_string);
                    self.pos.y += 1;
                    self.rows.insert(self.pos.y, new_split_string);
                } else {
                    self.pos.y += 1;
                    self.rows.insert(self.pos.y, split_string);
                    self.pos.x = self.end_of_line();
                }
            }
        };
    }

    fn delete_str(&mut self, start: usize, mut end: usize) {
        if end < start {
            return;
        }
        if end > self.end_of_line() + 1 {
            end = self.end_of_line() + 1;
        }
        let line = match self.rows.get_mut(self.pos.y) {
            Some(curr_line) => curr_line,
            None => return,
        };
        line.drain(start..end);
        self.set_x_or(self.end_of_line(), self.pos.x);
    }

    fn delete_lines(&mut self, start: usize, mut end: usize) {
        if end > self.end_of_file() + 1 {
            end = self.end_of_file() + 1;
        }
        self.rows.drain(start..end);
        self.set_y_or(self.end_of_file(), self.pos.y);
        self.set_x_or(0, self.pos.x);
    }

    fn move_to_end_of_line(&mut self, repeat: usize) {
        self.move_down(repeat - 1);
        self.move_to_x(self.end_of_line());
        self.x_end = usize::MAX
    }

    fn delete_to_end_of_line(&mut self, repeat: usize) {
        self.delete_str(self.pos.x, self.end_of_line() + 1);
        // self.move_left(1);
        if self.is_rows_full(self.pos.y) {
            return;
        }
        if repeat > 1 {
            self.delete_lines(self.pos.y + 1, self.pos.y + repeat);
        }
        // self.x_end =
    }

    fn move_to_first_non_white_space(&mut self) {
        self.move_to_x(self.first_non_white_space());
        self.x_end = self.pos.x
    }

    fn delete_to_first_non_white_space(&mut self) {
        if self.pos.x > self.first_non_white_space() {
            self.delete_str(self.first_non_white_space(), self.pos.x + 1);
        } else {
            self.delete_str(self.pos.x, self.first_non_white_space());
        }
        self.pos.x = self.first_non_white_space();
    }

    fn delete_start_of_line(&mut self) {
        self.delete_str(0, self.pos.x + 1);
    }

    fn move_left(&mut self, repeat: usize) {
        self.pos.x = self.pos.x.saturating_sub(repeat);
        self.x_end = self.pos.x;
    }

    fn delete_left(&mut self, repeat: usize) {
        let new_x = self.pos.x.saturating_sub(repeat);
        self.delete_str(new_x, self.pos.x);
        self.x_end = new_x;
    }

    fn append_line_to_prev_line(&mut self) {
        if !self.is_valid_y(self.pos.y) || !self.is_valid_y(self.pos.y.saturating_sub(1)) {
            return;
        }
        let addingline = self.rows.get(self.pos.y).unwrap().clone();
        self.rows
            .get_mut(self.pos.y.saturating_sub(1))
            .unwrap()
            .push_str(addingline.as_str());
        self.delete_lines(self.pos.y, self.pos.y + 1);
    }

    fn delete_backspace(&mut self, repeat: usize) {
        if self.pos.x == 0 {
            if self.pos.y == 0 {
                return;
            }
            self.pos.y = self.pos.y.saturating_sub(1);
            let new_x = if self.end_of_line() == 0 {
                0
            } else {
                self.end_of_line() + 1
            };
            self.pos.y += 1;
            self.append_line_to_prev_line();
            self.pos.y = self.pos.y.saturating_sub(1);
            self.pos.x = new_x;
            return;
        }
        let new_x = self.pos.x.saturating_sub(repeat);
        self.delete_str(new_x, self.pos.x);
        self.pos.x = new_x;
        self.x_end = new_x;
    }

    fn move_backspace(&mut self, repeat: usize) {
        if self.pos.x == 0 {
            if self.pos.y == 0 {
                return;
            }
            self.pos.y = self.pos.y.saturating_sub(1);
            self.pos.x = self.end_of_line();
            return;
        }
        self.pos.x = self.pos.x.saturating_sub(repeat);
        self.x_end = self.pos.x;
    }

    fn delete_right(&mut self, repeat: usize) {
        let mut new_x = self.pos.x + repeat;
        if new_x > self.end_of_line() + 1 {
            new_x = self.end_of_line() + 1;
        }
        if self.get_current_line().unwrap().len() == 0 {
            return;
        }

        self.delete_str(self.pos.x, new_x);
        self.set_x_or(self.end_of_line(), self.pos.x);
        self.x_end = self.pos.x;
    }

    fn move_to_start_of_line(&mut self) {
        self.pos.x = 0;
        self.x_end = 0;
    }

    fn move_right(&mut self, repeat: usize) {
        self.move_to_x(self.pos.x + repeat);
        self.x_end = self.pos.x;
    }

    pub fn end_of_line(&self) -> usize {
        self.get_current_line()
            .map_or(0, |row| row.len().saturating_sub(1))
    }

    pub fn first_non_white_space(&self) -> usize {
        self.get_current_line().map_or(0, |row| {
            row.chars()
                .position(|c| !c.is_whitespace())
                .map_or(0, |index| index)
        })
    }

    fn end_of_file(&self) -> usize {
        self.rows.len().saturating_sub(1)
    }

    fn move_to_line(&mut self, line: usize) {
        self.set_y_or(self.end_of_file(), line);
    }

    fn move_to_x(&mut self, x: usize) {
        self.set_x_or(self.end_of_line(), x);
    }

    fn move_up(&mut self, repeat: usize) {
        self.set_y_or(0, self.pos.y.saturating_sub(repeat));
        self.set_x_or(self.end_of_line(), self.x_end);
    }

    fn move_down(&mut self, repeat: usize) {
        self.set_y_or(self.end_of_file(), self.pos.y + repeat);
        self.set_x_or(self.end_of_line(), self.x_end);
    }

    fn delete_down(&mut self, repeat: usize) {
        self.delete_lines(self.pos.y, self.pos.y + repeat + 1);
    }

    fn get_next_empty_string(&self) -> usize {
        self.rows
            .iter()
            .skip(self.pos.y + 1)
            .enumerate()
            .find(|(_, s)| s.len() == 0)
            .map_or(self.rows.len().saturating_sub(1), |(idx, _)| {
                self.pos.y + 1 + idx
            })
    }

    fn get_previous_empty_string(&self) -> usize {
        self.rows
            .iter()
            .take(self.pos.y)
            .enumerate()
            .rev()
            .find(|(_, s)| s.len() == 0)
            .map_or(0, |(idx, _)| idx)
    }

    fn move_previous_paragraph(&mut self, repeat: usize) {
        for _ in 0..repeat {
            let line = self.get_previous_empty_string();
            self.move_to_line(line);
        }
    }

    fn move_next_paragraph(&mut self, repeat: usize) {
        for _ in 0..repeat {
            let line = self.get_next_empty_string();
            self.move_to_line(line);
        }
    }

    fn delete_word(&mut self, repeat: usize) {
        let start = self.pos.x;
        let start_line = self.pos.y;
        self.move_next_word(repeat);
        let end = self.pos.x;
        let end_line = self.pos.y;
        if start_line == end_line {
            if start == end {
                self.delete_str(start, end + 1);
            }
            self.delete_str(start, end);
            self.move_to_x(start);
            return;
        }
        if end == self.first_non_white_space() {
            self.move_to_line(start_line);
            if start == 0 || self.get_current_line().unwrap().is_empty() {
                self.delete_lines(start_line, end_line);
                self.move_to_x(start);
            } else {
                self.delete_str(start, self.end_of_line() + 1);
                self.delete_lines(start_line + 1, end_line);
                self.move_to_x(start);
            }
            return;
        }
        self.move_to_line(start_line);
        if start == 0 || self.get_current_line().unwrap().is_empty() {
            self.delete_lines(start_line, end_line);
            self.move_to_line(start_line + 1);
            self.delete_str(0, end);
            let addingline = self.rows.get(end_line).unwrap().clone();
            self.rows
                .get_mut(start_line)
                .unwrap()
                .push_str(addingline.as_str());
            self.delete_lines(start_line + 1, start_line + 2);
        } else {
            self.delete_str(start, self.end_of_line() + 1);
            self.delete_lines(start_line + 1, end_line);
            self.move_to_line(start_line + 1);
            self.delete_str(0, end);
            let addingline = self.rows.get(end_line).unwrap().clone();
            self.rows
                .get_mut(start_line)
                .unwrap()
                .push_str(addingline.as_str());
            self.delete_lines(start_line + 1, start_line + 2);
        }
        return;
    }

    fn move_next_word(&mut self, repeat: usize) {
        for _ in 0..repeat {
            let mut word = self.get_next_word();
            if let Some(val) = self
                .get_current_line()
                .unwrap()
                .chars()
                .nth(self.pos.x + word)
            {
                if val.is_whitespace() {
                    self.move_to_x(word);
                    word = self.get_next_word();
                }
            }

            if self.rows[self.pos.y].len() == word {
                self.move_down(1);
                self.move_to_x(self.first_non_white_space());
            } else {
                self.move_to_x(word);
            }
        }
    }

    fn move_next_word_after_white_space(&mut self, repeat: usize) {
        for _ in 0..repeat {
            let word = self.get_word_after_white_space();
            if self.rows[self.pos.y].len() == word {
                self.move_down(1);
                self.move_to_x(self.first_non_white_space());
            } else {
                self.move_to_x(word);
            }
        }
    }

    fn get_next_word(&self) -> usize {
        let curr_line = match self.get_current_line() {
            Some(line) => line,
            None => return 0,
        };
        if curr_line.is_empty() {
            return 0;
        }
        let initial_char = match curr_line.chars().nth(self.pos.x) {
            Some(val) => val,
            None => return 0,
        };
        let mut initial_type = self.find_char_class(initial_char);
        for (i, c) in curr_line
            .char_indices()
            .skip(self.pos.x)
            .map(|(i, c)| (i, c))
        {
            let char_type = self.find_char_class(c);
            if char_type != initial_type {
                if char_type == CharClass::WhiteSpace {
                    initial_type = CharClass::WhiteSpace;
                } else {
                    return i;
                }
            }
        }
        curr_line.len()
    }

    fn get_word_after_white_space(&self) -> usize {
        let curr_line = match self.get_current_line() {
            Some(line) => line,
            None => return 0,
        };
        if curr_line.is_empty() {
            return curr_line.len();
        }
        let mut flag = false;
        for (i, c) in curr_line
            .char_indices()
            .skip(self.pos.x)
            .map(|(i, c)| (i, c))
        {
            if c.is_whitespace() {
                flag = true;
            } else if flag {
                return i;
            }
        }
        curr_line.len()
    }

    fn find_char_class(&self, c: char) -> CharClass {
        match c {
            c if c.is_whitespace() => CharClass::WhiteSpace,
            c if c.is_alphanumeric() || c == '_' => CharClass::Keyword,
            _ => CharClass::Other,
        }
    }

    pub fn motion(&mut self, direction: Motion) {
        match direction {
            Motion::Left(repeat) => self.move_left(repeat),
            Motion::Right(repeat) => self.move_right(repeat),
            Motion::Up(repeat) => self.move_up(repeat),
            Motion::Down(repeat) => self.move_down(repeat),
            Motion::BackSpace(repeat) => self.move_backspace(repeat),
            Motion::Word(repeat) => self.move_next_word(repeat),
            Motion::WORD(repeat) => self.move_next_word_after_white_space(repeat),
            Motion::ParagraphStart(repeat) => self.move_previous_paragraph(repeat),
            Motion::ParagraphEnd(repeat) => self.move_next_paragraph(repeat),
            Motion::StartOfLine => self.move_to_start_of_line(),
            Motion::EndOfLine(repeat) => self.move_to_end_of_line(repeat),
            Motion::StartOfNonWhiteSpace => self.move_to_first_non_white_space(),
            Motion::GoToX(pos) => self.move_to_x(pos),
            Motion::GoToLine(line) => self.move_to_line(line),
            Motion::EndOfFile => self.move_to_line(self.end_of_file()),
        }
    }

    fn insert_append(&mut self, pos: usize) {
        if self.rows.len() > self.pos.y {
            let line_len = self.rows.get(self.pos.y).unwrap().len();
            if pos <= line_len {
                self.pos.x = pos;
            } else {
                self.pos.x = line_len;
            }
        }
    }

    fn insert_prev_line(&mut self) {
        if self.rows.len() > self.pos.y {
            let first_char = self.first_non_white_space();
            self.rows
                .insert(self.pos.y, String::from(" ".repeat(first_char)));
            self.pos.x = first_char;
        }
    }

    fn insert_next_line(&mut self) {
        if self.rows.len() > self.pos.y {
            let first_char = self.first_non_white_space();
            self.pos.y += 1;
            self.rows
                .insert(self.pos.y, String::from(" ".repeat(first_char)));
            self.pos.x = first_char;
        }
        if self.rows.is_empty() {
            self.pos.y += 1;
            self.rows.push(String::new());
            self.rows.push(String::new());
        }
    }

    pub fn insert(&mut self, pos: InsertType) {
        self.is_changed = true;
        match pos {
            InsertType::Append => self.insert_append(self.pos.x + 1),
            InsertType::InsertStart => self.move_to_first_non_white_space(),
            InsertType::AppendEnd => self.insert_append(self.end_of_line() + 1),
            InsertType::Next => self.insert_next_line(),
            InsertType::Prev => self.insert_prev_line(),
            InsertType::None => (),
        }
    }

    pub fn delete(&mut self, direction: Motion) {
        self.is_changed = true;
        match direction {
            Motion::Left(repeat) => self.delete_left(repeat),
            Motion::Right(repeat) => self.delete_right(repeat),
            Motion::Down(repeat) => self.delete_down(repeat),
            Motion::Word(repeat) => self.delete_word(repeat),
            Motion::BackSpace(repeat) => self.delete_backspace(repeat),
            Motion::ParagraphEnd(repeat) => self.move_next_paragraph(repeat),
            Motion::EndOfLine(repeat) => self.delete_to_end_of_line(repeat),
            Motion::StartOfNonWhiteSpace => self.delete_to_first_non_white_space(),
            Motion::StartOfLine => self.delete_start_of_line(),
            Motion::EndOfFile => self.delete_lines(self.pos.y, self.end_of_file() + 1),
            _ => (),
        }
    }

    fn is_rows_full(&self, end: usize) -> bool {
        self.rows.len() <= end
    }

    pub fn fix_cursor_pos_escape_insert(&mut self) {
        self.set_x_or(self.end_of_line(), self.pos.x);
    }

    pub fn write_buffer_file(
        &mut self,
        force: bool,
        filename: Option<String>,
    ) -> Result<String, FileError> {
        if let Some(name) = filename {
            write_file_to_disk(&name, &self.rows).map_err(|e| FileError::OtherError(e))?;
            self.modified_time = Self::get_modified_time(&name);
            self.is_changed = false;
            if self.filename.is_none() {
                self.filename = Some(name.clone())
            }
            return Ok(name);
        } else if let Some(name) = self.filename.clone() {
            if !force {
                let modified_time = Self::get_modified_time(&name);
                if modified_time != self.modified_time {
                    return Err(FileError::FileChanged);
                }
            };
            write_file_to_disk(&name, &self.rows).map_err(|e| FileError::OtherError(e))?;
            self.modified_time = Self::get_modified_time(&name);
            self.is_changed = false;
            return Ok(name);
        } else {
            return Err(FileError::EmptyFileName);
        }
    }
}
