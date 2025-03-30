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
        .map_or(strings.len().saturating_sub(1), |(idx, _)| idx)
}
pub fn get_first_non_white_space(str: &str) -> usize {
    str.chars()
        .position(|c| !c.is_whitespace())
        .map_or(0, |index| index)
}

fn find_char_class(c: char) -> CharClass {
    match c {
        c if c.is_whitespace() => CharClass::WhiteSpace,
        c if c.is_alphanumeric() || c == '_' => CharClass::Keyword,
        _ => CharClass::Other,
    }
}
