pub struct CommandMode {
    pub command: String,
}
pub enum CommandReturn<'a> {
    Escape,
    SaveQuit,
    Quit,
    Save,
    FileName(&'a String),
    None,
}
impl CommandMode {
    pub fn new() -> Self {
        let command = String::new();
        Self { command }
    }
    pub fn handle_key(&mut self, c: u8, save: bool) -> CommandReturn {
        if c == b'\x1b' {
            self.command = String::new();
            return CommandReturn::Escape;
        }
        if c == b':' {
            self.command = String::new();
            return CommandReturn::None;
        }
        if c == 13 {
            if save {
                return CommandReturn::FileName(&self.command);
            }
            //enter key
            return self.execute();
        }
        if c == 127 {
            self.command.pop();
            if self.command.len() <= 1 {
                return CommandReturn::Escape;
            }
        } else if c > 32 {
            self.command.push(c as char);
        }
        CommandReturn::None
    }
    fn execute(&mut self) -> CommandReturn {
        match self.command.as_str() {
            "w" => self.save_file(),

            "q" => {
                self.command = String::from("quit");
                self.quit()
            }
            "wq" => {
                // self.command = String::from("save_quit");
                CommandReturn::SaveQuit
            }
            _ => {
                self.command = String::from("Invalid Command!");
                CommandReturn::Escape
            }
        }
    }
    fn quit(&self) -> CommandReturn {
        CommandReturn::Quit
    }
    fn save_file(&mut self) -> CommandReturn {
        // if self.buffer.filename.is_empty() {
        //     // self.terminal.status_line_left = String::from("enter a filename: ");
        // } else {
        //     self.buffer.write_buffer_to_disk()?;
        //     self.mode = EditorModes::Normal;
        // }
        CommandReturn::Save
    }
}
