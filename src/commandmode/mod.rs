use crate::error::FileError;

pub struct CommandMode {
    pub command_string: String,
    command: String,
    value: Option<String>,
}
pub enum CommandReturn {
    BuffNext,
    BuffPrev,
    Escape,
    SaveQuit(Option<String>),
    BuffN(usize),
    ForceSaveQuit(Option<String>),
    Save(Option<String>),
    Quit,
    ForceQuit,
    ForceSave(Option<String>),
    None,
}
impl CommandMode {
    pub fn new() -> Self {
        let command = String::new();
        Self {
            command_string: command,
            command: String::new(),
            value: None,
        }
    }

    pub fn handle_file_write_result(&mut self, result: Result<String, FileError>) {
        self.escape("");
        match result {
            Ok(filename) => self.command_string = String::from(format!("saved file {}", filename)),
            Err(e) => self.handle_file_error(e),
        }
    }

    fn handle_file_error(&mut self, e: FileError) {
        match e {
            FileError::EmptyFileName => self.command_string = String::from("Empty file name"),
            FileError::OtherError(_) => (),
            FileError::FileChanged => {
                self.command_string = String::from("file changed use w! to overwrite ")
            }
        }
    }

    pub fn handle_key(&mut self, c: u8) -> CommandReturn {
        if c == b'\x1b' {
            return CommandReturn::Escape;
        }
        if c == b':' {
            return CommandReturn::None;
        }
        if c == 13 {
            if self.command.is_empty() {
                self.command = self.command_string.clone();
            } else {
                let mut words = self.command_string.split_whitespace();
                words.next();
                self.value = words.next().map_or(None, |s| Some(s.to_string()));
            }
            return self.execute();
        }
        if c == 127 {
            self.command_string.pop();
            if self.command_string.len() <= 1 {
                return CommandReturn::Escape;
            }
            return CommandReturn::None;
        }
        if c == 32 {
            if self.command.is_empty() {
                self.command = self.command_string.clone();
                self.command_string.push(c as char);
            }
        } else if c > 32 {
            self.command_string.push(c as char);
        }
        CommandReturn::None
    }

    fn buffer_command(&mut self) -> CommandReturn {
        let val = self.command.split_off(1);
        if let Ok(num) = val.as_str().parse::<usize>() {
            return CommandReturn::BuffN(num);
        }
        match val.as_str() {
            "n" => CommandReturn::BuffNext,
            "p" => CommandReturn::BuffPrev,
            _ => CommandReturn::Escape,
        }
    }

    pub fn escape(&mut self, err: &str) {
        self.command.clear();
        self.command_string = String::from(err);
    }

    fn execute(&mut self) -> CommandReturn {
        if self.command.starts_with("b") {
            return self.buffer_command();
        }
        match self.command.as_str() {
            "w" => CommandReturn::Save(self.value.clone()),
            "w!" => CommandReturn::ForceSave(self.value.clone()),
            "q!" => CommandReturn::ForceQuit,
            "q" => CommandReturn::Quit,
            "wq" => CommandReturn::SaveQuit(self.value.clone()),
            "wq!" => CommandReturn::ForceSaveQuit(self.value.clone()),
            _ => {
                self.command_string = String::from("Invalild Command");
                CommandReturn::Escape
            }
        }
    }
}
