use crate::buffer::{self, TextBuffer};
use crate::error::{AppError, Result};
use crate::terminal::{Size, Terminal};
use crate::utils::{
    get_first_non_white_space, get_next_empty_string, get_next_word, get_previous_empty_string,
    get_word_after_white_space, go_down, go_to_last_row, go_up, handle_y_move,
};
use std::cmp::{self, max};
use std::collections::HashSet;
use std::io::{self, Read, Write, stdout};
use std::iter::repeat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorModes {
    Normal,
    Insert,
    Command,
    Visual,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    EndOfLine,
    EndOfRows,
    PageTop,
    PageMiddle,
    PageBottom,
    GoToLine,
    StartOfLine,
    StartOfNonWhiteSpace,
    Word,
    ParagraphEnd,
    ParagraphStart,
    WORD,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalAction {
    Move(Motion),
    ChangeMode(EditorModes),
    NewLine,
    Delete,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CammandModeAction {
    ChangeMode(EditorModes),
}
struct PendingOperations {
    repeat: usize,
    action: char,
    valid_actions: HashSet<char>,
    valid_modifiers: HashSet<char>,
    valid_motions: HashSet<char>,
    modifier: char,
    motion: char,
}

impl PendingOperations {
    fn new() -> PendingOperations {
        let keys_action = ['d', 'f', 'g'];
        let valid_actions: HashSet<char> = keys_action.iter().cloned().collect();
        let keys_modifier = ['i', 'a', 'f'];
        let valid_modifiers: HashSet<char> = keys_modifier.iter().cloned().collect();
        let keys_motion = [
            'h', 'j', 'k', 'l', 'x', 'd', 'g', 'G', 'a', 'i', 'A', 'o', 'O', 'H', 'M', 'L', 'w',
            'W', 'e', '{', '}', ':', 'y', '^', '$', '0',
        ];
        let valid_motions: HashSet<char> = keys_motion.iter().cloned().collect();
        Self {
            repeat: 0,
            action: 0 as char,
            modifier: 0 as char,
            motion: 0 as char,
            valid_actions,
            valid_modifiers,
            valid_motions,
        }
    }
    fn reset(&mut self) {
        self.repeat = 0;
        self.action = 0 as char;
        self.modifier = 0 as char;
        self.motion = 0 as char;
    }
    fn is_action_given(&self) -> bool {
        self.action != '\0'
    }
    fn is_modifier_given(&self) -> bool {
        self.modifier != '\0'
    }
    fn is_motion_given(&self) -> bool {
        self.motion != '\0'
    }
    fn insert_key(&mut self, key: char) {
        if key != '0' && key.is_numeric() {
            self.repeat = self.repeat.saturating_mul(10);
            self.repeat = self
                .repeat
                .saturating_add(key.to_digit(10).map_or(0, |digit| digit as usize));
        } else if !self.is_action_given() && self.valid_actions.contains(&key) {
            self.action = key;
        } else if !self.is_modifier_given() && self.valid_modifiers.contains(&key) {
            self.modifier = key;
        } else if !self.is_motion_given() && self.valid_motions.contains(&key) {
            self.motion = key;
        } else {
        }
    }
}

pub struct Editor {
    terminal: Terminal,
    buffer: TextBuffer,
    pub exit_flag: bool,
    pub mode: EditorModes,
    pos: Size,
    pending_operations: PendingOperations,
}

impl Editor {
    pub fn new() -> Self {
        let buffer = TextBuffer::new();
        let terminal = Terminal::new().expect("error loading terminal");
        Self {
            terminal,
            buffer,
            exit_flag: false,
            mode: EditorModes::Normal,
            pos: Size { x: 0, y: 0 },
            pending_operations: PendingOperations::new(),
        }
    }
    pub fn run(&mut self) {
        loop {
            self.terminal.refresh_screen(&self.buffer, &self.pos);
            self.process_keypress();
            if self.exit_flag {
                break;
            }
        }
    }

    fn move_cursor(&mut self, direction: Motion) {
        match direction {
            Motion::Left => self.pos.x = self.pos.x.saturating_sub(1),
            Motion::PageTop => {
                self.pos = handle_y_move(
                    &self.buffer,
                    &self.pos,
                    self.pos.y.saturating_sub(self.terminal.top_screen_pos()),
                )
            }
            Motion::PageMiddle => {
                self.pos = handle_y_move(
                    &self.buffer,
                    &self.pos,
                    self.pos.y.saturating_sub(self.terminal.middle_screen_pos()),
                )
            }
            Motion::PageBottom => {
                self.pos = handle_y_move(
                    &self.buffer,
                    &self.pos,
                    self.pos.y.saturating_sub(self.terminal.bottom_screen_pos()),
                )
            }
            Motion::StartOfLine => self.pos.x = 0,
            Motion::GoToLine => self.pos.y = self.pending_operations.repeat.saturating_sub(1),
            Motion::EndOfRows => self.pos = go_to_last_row(&self.buffer, &self.pos),
            Motion::ParagraphEnd => {
                self.pos.y = get_next_empty_string(&self.buffer.rows, self.pos.y)
            }
            Motion::ParagraphStart => {
                self.pos.y = get_previous_empty_string(&self.buffer.rows, self.pos.y)
            }
            Motion::Word => {
                let word = get_next_word(self.buffer.rows.get(self.pos.y).unwrap(), self.pos.x);
                if word == self.buffer.rows.get(self.pos.y).unwrap().len() {
                    self.pos.y = go_down(&self.buffer, &self.pos).y;
                    let line = self.buffer.rows.get(self.pos.y).unwrap();
                    self.pos.x = get_first_non_white_space(line);
                } else {
                    self.pos.x = word;
                }
            }
            Motion::WORD => {
                let word = get_word_after_white_space(
                    self.buffer.rows.get(self.pos.y).unwrap(),
                    self.pos.x,
                );
                if word == self.buffer.rows.get(self.pos.y).unwrap().len() {
                    self.pos.y = go_down(&self.buffer, &self.pos).y;

                    let line = self.buffer.rows.get(self.pos.y).unwrap();
                    self.pos.x = get_first_non_white_space(line);
                } else {
                    self.pos.x = word;
                }
            }
            Motion::StartOfNonWhiteSpace => {
                let line = self.buffer.rows.get(self.pos.y).unwrap();
                self.pos.x = get_first_non_white_space(line);
            }

            Motion::EndOfLine => {
                self.pos.x = self
                    .buffer
                    .rows
                    .get(self.pos.y)
                    .map_or(0, |row| row.len().saturating_sub(1));
            }
            Motion::Right => {
                let current_row_len = self.buffer.rows.get(self.pos.y).map_or(0, |row| row.len());
                if self.pos.x < current_row_len.saturating_sub(1) {
                    self.pos.x += 1;
                }
            }
            Motion::Up => self.pos = go_up(&self.buffer, &self.pos),
            Motion::Down => self.pos = go_down(&self.buffer, &self.pos),
        }
    }
    fn handle_operation(&mut self) {
        // let mut abuf = String::new();
        // abuf.push_str("\x1b[H"); //cursor upperleft
        // abuf.push_str("      p"); //cursor upperleft
        // abuf.push(self.pending_operations.motion); //cursor upperleft
        // abuf.push_str("v      "); //cursor upperleft
        self.terminal.status_line_left = format!("{}", self.pending_operations.repeat);
        for i in 0..max(self.pending_operations.repeat, 1) {
            // c    abuf.push_str("mmmm"); //cursor upperleft
            // self.terminal.status_line_right = String::from("processing normal ode");
            if self.pending_operations.is_action_given() {
                self.terminal.status_line_right = String::from(self.pending_operations.action);
                match self.pending_operations.action {
                    'd' => match self.pending_operations.motion {
                        'd' => self.buffer.delete_row(&mut self.pos),
                        _ => (),
                    },
                    'g' => match self.pending_operations.motion {
                        'g' => {
                            self.normal_action(NormalAction::Move(Motion::GoToLine));
                            self.terminal.status_line_left = format!("{}", i);
                            break;
                        }

                        _ => (),
                    },
                    _ => (),
                }
            } else {
                // abuf.push_str("motion"); //cursor upperleft
                // abuf.push(self.pending_operations.motion); //cursor upperleft
                // abuf.push_str("motion"); //cursor upperleft
                match self.pending_operations.motion {
                    'h' => self.normal_action(NormalAction::Move(Motion::Left)),
                    'j' => self.normal_action(NormalAction::Move(Motion::Down)),
                    'k' => self.normal_action(NormalAction::Move(Motion::Up)),
                    'l' => self.normal_action(NormalAction::Move(Motion::Right)),
                    '$' => self.normal_action(NormalAction::Move(Motion::EndOfLine)),
                    '0' => self.normal_action(NormalAction::Move(Motion::StartOfLine)),
                    '^' => self.normal_action(NormalAction::Move(Motion::StartOfNonWhiteSpace)),
                    'w' => self.normal_action(NormalAction::Move(Motion::Word)),
                    'W' => self.normal_action(NormalAction::Move(Motion::WORD)),
                    'H' => self.normal_action(NormalAction::Move(Motion::PageTop)),
                    '{' => self.normal_action(NormalAction::Move(Motion::ParagraphStart)),
                    '}' => self.normal_action(NormalAction::Move(Motion::ParagraphEnd)),
                    'M' => self.normal_action(NormalAction::Move(Motion::PageMiddle)),
                    'L' => self.normal_action(NormalAction::Move(Motion::PageBottom)),
                    'G' => {
                        if self.pending_operations.repeat == 0 {
                            self.normal_action(NormalAction::Move(Motion::EndOfRows));
                            self.terminal.status_line_left = format!("{}", i);
                            break;
                        }
                        self.normal_action(NormalAction::Move(Motion::GoToLine));
                        self.terminal.status_line_left = format!("{}", i);
                        break;
                    }
                    'i' => self.normal_action(NormalAction::ChangeMode(EditorModes::Insert)),
                    ':' => {
                        self.normal_action(NormalAction::ChangeMode(EditorModes::Command));
                        self.terminal.status_line_left = String::from(":");
                    }
                    'a' => {
                        if !self.buffer.rows.get(self.pos.y).unwrap().len() == 0 {
                            self.pos.x += 1;
                        }
                        self.normal_action(NormalAction::ChangeMode(EditorModes::Insert))
                    }
                    'A' => {
                        self.normal_action(NormalAction::Move(Motion::EndOfLine));
                        self.pos.x += 1;
                        self.normal_action(NormalAction::ChangeMode(EditorModes::Insert))
                    }
                    'I' => {
                        self.normal_action(NormalAction::Move(Motion::StartOfNonWhiteSpace));
                        self.normal_action(NormalAction::ChangeMode(EditorModes::Insert));
                    }
                    'o' => {
                        self.pos.y += 1;
                        self.normal_action(NormalAction::NewLine);
                        self.normal_action(NormalAction::ChangeMode(EditorModes::Insert));
                    }
                    'O' => {
                        self.normal_action(NormalAction::NewLine);
                        self.normal_action(NormalAction::ChangeMode(EditorModes::Insert));
                    }
                    'x' => {
                        self.buffer.delete_char(&mut self.pos);
                    }
                    _ => self.normal_action(NormalAction::Unknown),
                }
            }
        }
        self.pending_operations.reset();

        // write!(io::stdout(), "{}", abuf);
        // stdout().flush().expect("flush");
    }

    fn process_normal_mode(&mut self, c: u8) {
        if c < 32 {
            return;
        }
        self.pending_operations.insert_key(c as char);
        println!("self ");

        let action_given = self.pending_operations.is_motion_given();
        if action_given {
            println!("given ");
            self.handle_operation();
        }
        // self.status_line_right = String::new();
    }
    fn normal_action(&mut self, action: NormalAction) {
        match action {
            NormalAction::Move(direction) => self.move_cursor(direction),
            NormalAction::ChangeMode(editormode) => {
                self.terminal.change_cursor(editormode);
                self.mode = editormode;
            }
            NormalAction::NewLine => self.buffer.insert_newline(&mut self.pos),
            NormalAction::Delete => self.buffer.delete_char(&mut self.pos),
            _ => (),
        }
    }

    fn process_insert_mode(&mut self, c: u8) {
        if c == b'\x1b' {
            if self.pos.x >= self.buffer.rows[self.pos.y].len() {
                self.pos.x = cmp::max(self.buffer.rows.get(self.pos.y).unwrap().len(), 0) - 1;
            } else if self.pos.x > 0 {
                self.pos.x -= 1;
            }

            self.terminal.change_cursor(EditorModes::Normal);
            self.mode = EditorModes::Normal;
            return;
        } else if c == 127 {
            if self.pos.x == 0 && self.pos.y > 0 {
                let content = self.buffer.rows.remove(self.pos.y);
                self.buffer.rows[self.pos.y - 1].push_str(&content);
                self.pos.y -= 1;
                self.pos.x = self.buffer.rows[self.pos.y].len() - 1;
            } else {
                self.normal_action(NormalAction::Delete);
                self.pos.x -= 1;
            }
        } else if c == 13 {
            self.pos.y += 1;
            self.normal_action(NormalAction::NewLine);
        } else if !((c) < 32) {
            self.buffer.insert_char(c, &mut self.pos);
        }
    }
    fn process_command_mode(&mut self, c: u8) {
        if c == b'\x1b' {
            self.terminal.status_line_left = String::new();
            self.mode = EditorModes::Normal;
            return;
        }
        if c == 13 {
            //enter key
            self.do_command();
            // self.exit_flag = true;
            // self.mode = EditorModes::Normal;
            return;
        }
        if c > 32 {
            self.terminal.status_line_left.push(c as char);
        }
    }
    fn do_command(&mut self) {
        if self
            .terminal
            .status_line_left
            .as_str()
            .starts_with("enter a filename: ")
        {
            self.buffer.filename.push_str(
                &(self
                    .terminal
                    .status_line_left
                    .strip_prefix("enter a filename: ")
                    .unwrap()),
            );
            self.save_file();
        } else {
            match self.terminal.status_line_left.as_str() {
                ":w" => self.save_file(),
                ":q" => self.exit_flag = true,
                ":wq" => {
                    self.save_file();
                    if self.mode == EditorModes::Normal {
                        self.exit_flag = true;
                    }
                }
                _ => self.terminal.status_line_left = String::from("!not a valid command."),
            }
        }
    }
    fn save_file(&mut self) {
        if self.buffer.filename.is_empty() {
            self.terminal.status_line_left = String::from("enter a filename: ");
        } else {
            self.buffer.write_buffer_to_disk();
            self.mode = EditorModes::Normal;
        }
    }
    fn process_keypress(&mut self) {
        let c = self.terminal.read_key();
        if c > 32 {
            // self.status_line_right.push(c as char);
        }
        match self.mode {
            EditorModes::Normal => self.process_normal_mode(c),
            EditorModes::Insert => self.process_insert_mode(c),
            EditorModes::Command => self.process_command_mode(c),
            _ => eprintln!("error"),
        }
    }
}
impl Drop for Editor {
    fn drop(&mut self) {}
}
