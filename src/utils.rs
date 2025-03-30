use crate::{
    buffer::{self, TextBuffer},
    terminal::Size,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CharClass {
    Keyword,
    WhiteSpace,
    Other,
}

pub fn get_next_word(str: &str, index: usize) -> usize {
    let initial_char = str.chars().nth(index).unwrap();
    let mut initial_type = find_char_class(initial_char);
    for (i, c) in str.char_indices().skip(index).map(|(i, c)| (i, c)) {
        let char_type = find_char_class(c);
        if char_type != initial_type {
            if char_type == CharClass::WhiteSpace {
                initial_type = CharClass::WhiteSpace;
            } else {
                return i;
            }
        }
    }
    str.len()
}
pub fn get_word_after_white_space(str: &str, index: usize) -> usize {
    let mut flag = false;
    for (i, c) in str.char_indices().skip(index).map(|(i, c)| (i, c)) {
        if c.is_whitespace() {
            flag = true;
        } else if flag {
            return i;
        }
    }
    str.len()
}
pub fn get_next_empty_string(strings: &Vec<String>, index: usize) -> usize {
    strings
        .iter()
        .skip(index + 1)
        .enumerate()
        .find(|(_, s)| s.len() == 0)
        .map_or(strings.len().saturating_sub(1), |(idx, _)| index + 1 + idx)
}
pub fn get_previous_empty_string(strings: &Vec<String>, index: usize) -> usize {
    strings
        .iter()
        .take(index)
        .enumerate()
        .rev()
        .find(|(_, s)| s.len() == 0)
        .map_or(0, |(idx, _)| idx)
}
pub fn get_first_non_white_space(str: &str) -> usize {
    str.chars()
        .position(|c| !c.is_whitespace())
        .map_or(0, |index| index)
}
pub fn go_down(buffer: &TextBuffer, pos: &Size) -> Size {
    let mut new_pos = Size::new();
    new_pos.x = pos.x;
    new_pos.y = pos.y;
    let current_row_len = buffer.rows.get(pos.y).map_or(0, |row| row.len());
    if pos.y < buffer.rows.len().saturating_sub(1) {
        new_pos.y += 1;
    }
    let new_row_len = buffer.rows.get(new_pos.y).map_or(0, |row| row.len());
    if pos.x != 0
        && (pos.x == current_row_len.saturating_sub(1) || pos.x > new_row_len.saturating_sub(1))
    {
        new_pos.x = new_row_len.saturating_sub(1);
    } else {
        new_pos.x = 0;
    }
    new_pos
}
pub fn handle_y_move(buffer: &TextBuffer, pos: &Size, newY: usize) -> Size {
    let mut new_pos = Size::new();
    let current_row_len = buffer.rows.get(pos.y).map_or(0, |row| row.len());
    new_pos.y = newY;
    let new_row_len = buffer.rows.get(new_pos.y).map_or(0, |row| row.len());
    new_pos.x = handle_new_x(pos, current_row_len, new_row_len);
    new_pos
}

pub fn go_up(buffer: &TextBuffer, pos: &Size) -> Size {
    let mut new_pos = Size::new();
    let current_row_len = buffer.rows.get(pos.y).map_or(0, |row| row.len());
    new_pos.y = pos.y.saturating_sub(1);
    let new_row_len = buffer.rows.get(new_pos.y).map_or(0, |row| row.len());
    new_pos.x = handle_new_x(pos, current_row_len, new_row_len);
    new_pos
}
pub fn go_to_last_row(buffer: &TextBuffer, pos: &Size) -> Size {
    let mut new_pos = Size::new();
    new_pos.y = buffer.rows.len().saturating_sub(1);
    let current_row_len = buffer.rows.get(pos.y).map_or(0, |row| row.len());
    let new_row_len = buffer.rows.get(new_pos.y).map_or(0, |row| row.len());
    new_pos.x = handle_new_x(pos, current_row_len, new_row_len);
    new_pos
}
fn handle_new_x(pos: &Size, current_row_len: usize, new_row_len: usize) -> usize {
    let newx: usize;
    if pos.x != 0
        && (pos.x == current_row_len.saturating_sub(1) || pos.x > new_row_len.saturating_sub(1))
    {
        newx = new_row_len.saturating_sub(1);
    } else {
        newx = 0;
    }
    newx
}

fn find_char_class(c: char) -> CharClass {
    match c {
        c if c.is_whitespace() => CharClass::WhiteSpace,
        c if c.is_alphanumeric() || c == '_' => CharClass::Keyword,
        _ => CharClass::Other,
    }
}
