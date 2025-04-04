use crate::buffer::TextBuffer;
use crate::commandmode::{CommandMode, CommandReturn};
use crate::error::{AppError, FileError, Result};
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
    current_buffer: usize,
    pub exit_flag: bool,
    save_flag: bool,
    pub mode: EditorModes,
    normal_mode: NormalMode,
    command_mode: CommandMode,
}

impl Editor {
    pub fn new(args: Vec<String>) -> Result<Self> {
        let buffers = TextBuffer::new(args)?;
        let current_buffer: usize = 0;
        let curr_buff = buffers.get(&current_buffer).unwrap();
        let terminal = Terminal::new(curr_buff.rows.len(), &curr_buff.filename)?;
        Ok(Self {
            normal_mode: NormalMode::new(),
            command_mode: CommandMode::new(),
            current_buffer,
            terminal,
            buffers,
            exit_flag: false,
            save_flag: false,
            mode: EditorModes::Normal,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            if self.buffers.len() == 0 {
                return Ok(());
            }
            self.process_keypress()?;
            self.render_ui()?;
        }
    }

    fn render_ui(&mut self) -> Result<()> {
        if let Some(buffer) = self.buffers.get(&self.current_buffer) {
            self.terminal.refresh_screen(buffer)?;
            return Ok(());
        }
        Err(AppError::BufferError)
    }

    fn process_normal_mode(&mut self, c: u8) {
        match self.normal_mode.handle_keypress(c) {
            Ok(action) => self.handle_operation(action),
            Err(_) => (),
        }
        self.terminal.status_line_right = String::from(self.normal_mode.pending_operations.motion);
    }
    fn handle_operation(&mut self, action: BufferAction) {
        if let Some(buffer) = self.buffers.get_mut(&self.current_buffer) {
            match action {
                BufferAction::Delete(direction) => buffer.delete(direction),
                BufferAction::Move(direction) => buffer.motion(direction),
                BufferAction::ChangeMode(mode, pos) => self.change_mode(mode, pos),
                BufferAction::None => (),
            }
        }

        self.normal_mode.pending_operations.reset();
    }
    fn change_mode(&mut self, mode: EditorModes, pos: InsertType) {
        let buffer = self.buffers.get_mut(&self.current_buffer).unwrap();
        self.mode = mode;
        match mode {
            EditorModes::Insert => buffer.insert(pos),
            EditorModes::Command => {
                self.process_command_mode(self.normal_mode.pending_operations.motion as u8);
                self.terminal.status_line_left = String::from(":");
            }
            _ => (),
        }
    }
    fn activate_normal_mode(&mut self) {
        if let Some(buffer) = self.buffers.get_mut(&self.current_buffer) {
            buffer.fix_cursor_pos_escape_insert();
            self.terminal.change_cursor(EditorModes::Normal);
            self.mode = EditorModes::Normal;
        }
    }

    fn process_insert_mode(&mut self, c: u8) {
        if let Some(buffer) = self.buffers.get_mut(&self.current_buffer) {
            match InsertAction::handle_key(c) {
                InsertAction::Backspace => buffer.delete(Motion::BackSpace(1)),
                InsertAction::Escape => self.activate_normal_mode(),
                InsertAction::Newline => buffer.split_line(),
                InsertAction::Chars(c) => buffer.insert_char(c),
                InsertAction::None => (),
            }
        }
    }

    fn process_command_mode(&mut self, c: u8) {
        if let Some(buffer) = self.buffers.get_mut(&self.current_buffer) {
            let value = self.command_mode.handle_key(c, self.save_flag);
            match value {
                CommandReturn::FileName(filename) => {
                    buffer.filename = filename.to_string();
                    // match buffer.write_buffer_file() {
                    //     Ok(_) => buffer.,
                    // }
                    buffer.write_buffer_file();
                    self.exit_flag = true;
                }
                CommandReturn::Quit => {
                    if !buffer.is_changed {
                        buffer.exit_flag = true
                    } else {
                        self.command_mode.command =
                            String::from("file  changes use q! to force quit");
                    }
                }
                CommandReturn::ForceQuit => self.exit_flag = true,
                CommandReturn::None => {
                    self.terminal.status_line_right = String::from("None");
                }
                CommandReturn::ForceSave => {
                    buffer.write_buffer_file();
                    self.terminal.status_line_right = String::from("Save");
                }
                CommandReturn::Save => {
                    match buffer.write_buffer_file() {
                        Ok(_) => self.command_mode.command = String::from("file saved"),
                        Err(_) => self.command_mode.command = String::from("file saving error"),
                    }
                    self.terminal.status_line_right = String::from("Save");
                }
                CommandReturn::SaveQuit => match buffer.write_buffer_file() {
                    Ok(_) => buffer.exit_flag = true,
                    Err(val) => match val {
                        FileError::FileChanged => {
                            self.command_mode.command = String::from("file changed use w!")
                        }
                        FileError::EmptyFileName => {
                            self.command_mode.command = String::new();
                            self.terminal.status_line_right = String::from("enter file name:");
                        }
                    },
                },
                CommandReturn::Escape => {
                    self.mode = EditorModes::Normal;
                    self.terminal.status_line_right = String::from("Escape");
                }
            }
            if self.mode == EditorModes::Command {
                if self.save_flag {
                    self.terminal.command_line = String::from("enter file name: ");
                } else {
                    self.terminal.command_line = String::from(":");
                }
            } else {
                self.terminal.command_line = String::new();
            }
            self.terminal
                .command_line
                .push_str(&self.command_mode.command);
        }
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
