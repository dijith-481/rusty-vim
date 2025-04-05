use std::{
    fmt, io,
    num::{self, ParseIntError},
};
#[derive(Debug)]
pub enum AppError {
    TermError,
    BufferError(String),
    Io(io::Error),
    FileWriteError,
    ParseIntError(num::ParseIntError),
}
#[derive(Debug)]
pub enum FileError {
    EmptyFileName,
    FileChanged,
    OtherError(AppError),
}
impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::EmptyFileName => write!(f, "No filename"),
            FileError::OtherError(_) => write!(f, "other error"),
            FileError::FileChanged => write!(f, "file changed"),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::TermError => write!(f, "Terminal error"),
            AppError::BufferError(e) => write!(f, "Error opening buffer {}", e),
            AppError::Io(e) => write!(f, "I/O error: {}", e),
            AppError::ParseIntError(e) => write!(f, "I/O error: {}", e),
            AppError::FileWriteError => write!(f, "File write error: ",),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> AppError {
        AppError::Io(err)
    }
}
impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        AppError::ParseIntError(err)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
