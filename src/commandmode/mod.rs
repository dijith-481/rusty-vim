pub struct CommandMode {
    pub command: String,
}
pub enum CommandReturn {
    Escape,
    Quit,
    Save,
    None,
}
impl CommandMode {
    pub fn new() -> Self {
        let command = String::new();
        Self { command }
    }
    pub fn handle_key(&mut self, c: u8) -> CommandReturn {
        if c == b'\x1b' {
            self.command = String::new();
            // EditorModes::Normal;
            return CommandReturn::Escape;
        }
        if c == b':' {
            self.command = String::new();
            return CommandReturn::None;
        }
        if c == 13 {
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
        // self.command = String::from("hello");
        match self.command.as_str() {
            "w" => self.save_file(),

            "q" => {
                self.command = String::from("quit");
                // CommandReturn::Quit
                self.quit()
            }
            _ => {
                self.command = String::from("Invalid Command!");
                CommandReturn::Escape
            }
        }
        // h// if self
        //     .terminal
        //     .status_line_left
        //     .as_str()
        //     .starts_with("enter a filename: ")
        // {
        //     self.buffer.filename.push_str(
        //         &(self
        //             .terminal
        //             .status_line_left
        //             .strip_prefix("enter a filename: ")
        //             .unwrap()),
        //     );
        // self.save_file()?;
        // } else {
        // match self.terminal.status_line_left.as_str() {
        //     ":w" => self.save_file()?,
        //     ":q" => self.exit_flag = true,
        //     ":wq" => {
        //         self.save_file()?;
        //         if self.mode == EditorModes::Normal {
        //             self.exit_flag = true;
        //         }
        //     }
        //     _ => self.terminal.status_line_left = String::from("!not a valid command."),
        // }
        // }
        // CommandReturn::None
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
