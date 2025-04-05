use crate::buffer::TextBuffer;
use crate::commandmode::{CommandMode, CommandReturn};
use crate::error::{AppError, Result};
use crate::insertmode::InsertAction;
use crate::insertmode::InsertType;
use crate::normalmode::NormalMode;
use crate::normalmode::motions::BufferAction;
use crate::normalmode::motions::Motion;
use crate::terminal::Terminal;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EditorModes {
    Normal,
    Insert,
    Command,
}

pub struct Editor {
    terminal: Terminal,
    buffers: HashMap<usize, TextBuffer>,
    buff_vec: Vec<usize>,
    current_buff_index: usize,
    mode: EditorModes,
    normal_mode: NormalMode,
    command_mode: CommandMode,
}

impl Editor {
    pub fn new(args: Vec<String>) -> Result<Self> {
        let mut buff_vec: Vec<usize> = Vec::new();
        let buffers = TextBuffer::load_buffers(args, &mut buff_vec)?;
        let current_buff_index: usize = 0;
        let curr_buff = buffers.get(&buff_vec[0]).unwrap();
        let filename = match &curr_buff.filename {
            Some(name) => name,
            None => &String::new(),
        };
        let terminal = Terminal::new(curr_buff.rows.len(), filename)?;
        Ok(Self {
            normal_mode: NormalMode::new(),
            buff_vec,
            command_mode: CommandMode::new(),
            current_buff_index,
            terminal,
            buffers,
            mode: EditorModes::Normal,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.process_keypress()?;
            if self.buff_vec.len() == 0 {
                return Ok(());
            }
            self.render_ui()?;
        }
    }

    fn get_buff_key(&mut self) -> usize {
        if self.buff_vec.len() <= self.current_buff_index {
            self.current_buff_index = self.buff_vec.len().saturating_sub(1);
        }
        let curr_buff_key = self.buff_vec.get(self.current_buff_index).unwrap().clone();
        curr_buff_key
    }

    fn render_ui(&mut self) -> Result<()> {
        let curr_buff_key = self.get_buff_key();
        if let Some(buffer) = self.buffers.get(&curr_buff_key) {
            self.terminal.refresh_screen(buffer, self.mode)?;
            return Ok(());
        }
        Err(AppError::BufferError(String::from("invalid buffer")))
    }

    fn process_normal_mode(&mut self, c: u8) {
        match self.normal_mode.handle_keypress(c) {
            Ok(action) => {
                let val = if self.normal_mode.pending_operations.repeat != 0 {
                    &self.normal_mode.pending_operations.repeat.to_string()
                } else {
                    ""
                };
                self.terminal.status_line_right = String::from(format!(
                    "{}{}{}",
                    val,
                    self.normal_mode.pending_operations.action,
                    self.normal_mode.pending_operations.motion
                ));
                let curr_buff_key = self.get_buff_key();
                if let Some(buffer) = self.buffers.get_mut(&curr_buff_key) {
                    match action {
                        BufferAction::Delete(direction) => buffer.delete(direction),
                        BufferAction::Move(direction) => buffer.motion(direction),
                        BufferAction::ChangeMode(mode, pos) => self.change_mode(mode, pos),
                        BufferAction::None => (),
                    }
                }
                self.normal_mode.pending_operations.reset();
            }
            Err(_) => (),
        }
    }

    fn change_mode(&mut self, mode: EditorModes, pos: InsertType) {
        let curr_buff_key = self.get_buff_key();
        if let Some(buffer) = self.buffers.get_mut(&curr_buff_key) {
            self.mode = mode;
            self.terminal.change_cursor(mode);
            match mode {
                EditorModes::Insert => buffer.insert(pos),
                EditorModes::Command => {
                    self.command_mode.command_string.clear();
                }
                EditorModes::Normal => buffer.fix_cursor_pos_escape_insert(),
            }
        }
    }

    fn process_insert_mode(&mut self, c: u8) {
        let curr_buff_key = self.get_buff_key();
        if let Some(buffer) = self.buffers.get_mut(&curr_buff_key) {
            match InsertAction::handle_key(c) {
                InsertAction::Backspace => buffer.delete(Motion::BackSpace(1)),
                InsertAction::Escape => self.change_mode(EditorModes::Normal, InsertType::None),
                InsertAction::Newline => buffer.split_line(),
                InsertAction::Chars(c) => buffer.insert_char(c),
                InsertAction::None => (),
            }
        }
    }

    fn process_command_mode(&mut self, c: u8) {
        let mut should_quit = false;
        let curr_buff_key = self.get_buff_key();
        if let Some(buffer) = self.buffers.get_mut(&curr_buff_key) {
            match self.command_mode.handle_key(c) {
                CommandReturn::Quit => {
                    if !buffer.is_changed {
                        self.command_mode.escape("");
                        should_quit = true;
                    } else {
                        self.command_mode
                            .escape("file  changes use q! to force quit");
                        self.change_mode(EditorModes::Normal, InsertType::None);
                    }
                    self.mode = EditorModes::Normal;
                }
                CommandReturn::ForceQuit => {
                    self.mode = EditorModes::Normal;
                    self.command_mode.escape("");
                    should_quit = true;
                }
                CommandReturn::ForceSave(filename) => {
                    let result = buffer.write_buffer_file(true, filename);
                    self.command_mode.handle_file_write_result(result);
                    self.mode = EditorModes::Normal;
                }
                CommandReturn::None => (),
                CommandReturn::Save(filename) => {
                    let result = buffer.write_buffer_file(true, filename);
                    self.command_mode.handle_file_write_result(result);
                    self.mode = EditorModes::Normal;
                }
                CommandReturn::ForceSaveQuit(filename) => {
                    let result = buffer.write_buffer_file(true, filename);
                    if let Ok(_) = &result {
                        should_quit = true;
                    }
                    self.command_mode.handle_file_write_result(result);
                    self.mode = EditorModes::Normal;
                }
                CommandReturn::SaveQuit(filename) => {
                    let result = buffer.write_buffer_file(false, filename);
                    if let Ok(_) = &result {
                        should_quit = true;
                    }
                    self.mode = EditorModes::Normal;
                    self.command_mode.handle_file_write_result(result);
                }
                CommandReturn::Escape => {
                    self.command_mode.escape("");
                    self.mode = EditorModes::Normal;
                }
                CommandReturn::BuffNext => self.buf_next(),
                CommandReturn::BuffPrev => self.buf_prev(),
                CommandReturn::BuffN(n) => self.buf_n(n),
            }
            if self.mode == EditorModes::Command {
                self.terminal.command_line = String::from(":");
            } else {
                self.terminal.command_line = String::new();
            }
            self.terminal
                .command_line
                .push_str(&self.command_mode.command_string);
        }
        if should_quit {
            self.buffers.remove(&curr_buff_key);
            self.buff_vec.remove(self.current_buff_index);
            self.current_buff_index = self.current_buff_index.saturating_sub(1);
        }
    }

    fn buf_prev(&mut self) {
        if self.current_buff_index == 0 {
            self.current_buff_index = self.buff_vec.len().saturating_sub(1);
        } else {
            self.current_buff_index = self.current_buff_index.saturating_sub(1);
        }
        self.command_mode.escape("");
        self.mode = EditorModes::Normal;
    }

    fn buf_n(&mut self, n: usize) {
        if n == self.buff_vec.len() {
            let buffer = TextBuffer::new(None).unwrap();
            let key = self.buff_vec.last().unwrap() + 1;
            self.buff_vec.push(key);
            self.buffers.insert(key, buffer);
            self.current_buff_index = n;
            self.command_mode.escape("");
        } else if self.buff_vec.len() > n {
            self.current_buff_index = n;
            self.command_mode.escape("");
        } else {
            self.command_mode.escape("Buffer not found");
        }
        self.mode = EditorModes::Normal;
    }

    fn buf_next(&mut self) {
        if self.buff_vec.len().saturating_sub(1) == self.current_buff_index {
            self.current_buff_index = 0;
        } else {
            self.current_buff_index += 1;
        }
        self.command_mode.escape("");
        self.mode = EditorModes::Normal;
    }

    fn process_keypress(&mut self) -> Result<()> {
        let c = self.terminal.read_key()?;
        match self.mode {
            EditorModes::Normal => {
                self.process_normal_mode(c);
                self.terminal.status_line_left = String::from(" Normal ");
            }
            EditorModes::Insert => {
                self.process_insert_mode(c);
                self.terminal.status_line_left = String::from(" Insert ");
            }
            EditorModes::Command => {
                self.process_command_mode(c);
                self.terminal.status_line_left = String::from(" Command ");
            }
        }

        Ok(())
    }
}
impl Drop for Editor {
    fn drop(&mut self) {}
}
