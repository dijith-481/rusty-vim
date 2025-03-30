use crate::buffer::TextBuffer;
use crate::commandmode::{CommandMode, CommandReturn};
use crate::error::Result;

use crate::normalmode::NormalMode;
use crate::normalmode::motions::NormalAction;
use crate::normalmode::{motions::Motion, operation_pending::PendingOperations};
use crate::terminal::{Position, Terminal};
use crate::utils::{
    get_first_non_white_space, get_next_empty_string, get_next_word, get_previous_empty_string,
    get_word_after_white_space, go_down, go_to_last_row, go_up, handle_y_move,
};
use std::cmp::{self, max};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorModes {
    Normal,
    Insert,
    Command,
}

pub struct Editor {
    terminal: Terminal,
    buffer: TextBuffer,
    pub exit_flag: bool,
    pub mode: EditorModes,
    pos: Position,
    normal_mode: NormalMode,
    command_mode: CommandMode,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let buffer = TextBuffer::new()?;
        let terminal = Terminal::new(buffer.rows.len(), &buffer.filename)?;
        Ok(Self {
            normal_mode: NormalMode::new(),
            command_mode: CommandMode::new(),
            terminal,
            buffer,
            exit_flag: false,
            mode: EditorModes::Normal,
            pos: Position { x: 0, y: 0 },
        })
    }
    pub fn run(&mut self) -> Result<()> {
        loop {
            if self.exit_flag {
                return Ok(());
            }
            self.process_keypress()?;
            self.render_ui()?;
        }
    }
    fn render_ui(&mut self) -> Result<()> {
        self.terminal.refresh_screen(&self.buffer, &self.pos)?;
        Ok(())
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
            Motion::GoToLine => {
                self.pos.y = self.normal_mode.pending_operations.repeat.saturating_sub(1)
            }
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
        // abuf.push(self.normal_mode.pending_operations.motion); //cursor upperleft
        // abuf.push_str("v      "); //cursor upperleft
        // self.terminal.status_line_left = format!("{}", self.normal_mode.pending_operations.repeat);
        for i in 0..max(self.normal_mode.pending_operations.repeat, 1) {
            // c    abuf.push_str("mmmm"); //cursor upperleft
            // self.terminal.status_line_right = String::from("processing normal ode");
            if self.normal_mode.pending_operations.is_action_given() {
                self.terminal.status_line_right =
                    String::from(self.normal_mode.pending_operations.action);
                match self.normal_mode.pending_operations.action {
                    'd' => match self.normal_mode.pending_operations.motion {
                        'd' => self.buffer.delete_row(&mut self.pos),
                        _ => (),
                    },
                    'g' => match self.normal_mode.pending_operations.motion {
                        'g' => {
                            self.normal_action(NormalAction::Move(Motion::GoToLine));
                            // self.terminal.status_line_left = format!("{}", i);
                            break;
                        }

                        _ => (),
                    },
                    _ => (),
                }
            } else {
                // abuf.push_str("motion"); //cursor upperleft
                // abuf.push(self.normal_mode.pending_operations.motion); //cursor upperleft
                // abuf.push_str("motion"); //cursor upperleft
                match self.normal_mode.pending_operations.motion {
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
                        if self.normal_mode.pending_operations.repeat == 0 {
                            self.normal_action(NormalAction::Move(Motion::EndOfRows));
                            // self.terminal.status_line_left = format!("{}", i);
                            break;
                        }
                        self.normal_action(NormalAction::Move(Motion::GoToLine));
                        // self.terminal.status_line_left = format!("{}", i);
                        break;
                    }
                    'i' => self.normal_action(NormalAction::ChangeMode(EditorModes::Insert)),
                    ':' => {
                        self.normal_action(NormalAction::ChangeMode(EditorModes::Command));
                        self.process_command_mode(self.normal_mode.pending_operations.motion as u8);
                        // self.terminal.status_line_left = String::from(":");
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
        self.normal_mode.pending_operations.reset();

        // write!(io::stdout(), "{}", abuf);
        // stdout().flush().expect("flush");
    }

    fn process_normal_mode(&mut self, c: u8) {
        if c < 32 {
            return;
        }
        self.normal_mode.pending_operations.insert_key(c as char);
        println!("self ");

        let action_given = self.normal_mode.pending_operations.is_motion_given();
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
                self.pos.x = self.pos.x.saturating_sub(1);
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
        let value = self.command_mode.handle_key(c);
        match value {
            CommandReturn::Quit => self.exit_flag = true,
            CommandReturn::None => {
                self.terminal.status_line_right = String::from("None");
            }
            CommandReturn::Save => {
                self.terminal.status_line_right = String::from("Save");
            }
            CommandReturn::Escape => {
                self.mode = EditorModes::Normal;
                self.terminal.status_line_right = String::from("Escape");
            }
        }
        if self.mode == EditorModes::Command {
            self.terminal.command_line = String::from(":");
        } else {
            self.terminal.command_line = String::new();
        }
        self.terminal
            .command_line
            .push_str(&self.command_mode.command);
        // }
    }
    fn process_keypress(&mut self) -> Result<()> {
        let c = self.terminal.read_key()?;
        match self.mode {
            EditorModes::Normal => {
                self.process_normal_mode(c);
                self.terminal.status_line_left = String::from("Normal");
            }
            EditorModes::Insert => {
                self.process_insert_mode(c);
                self.terminal.status_line_left = String::from("Insert");
            }
            EditorModes::Command => {
                self.process_command_mode(c);
                self.terminal.status_line_left = String::from("Command");
            }
        }
        Ok(())
    }
}
impl Drop for Editor {
    fn drop(&mut self) {}
}

