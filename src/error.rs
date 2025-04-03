use std::{
    fmt, io,
    num::{self, ParseIntError},
};
#[derive(Debug)]
pub enum AppError {
    TermError,
    Io(io::Error),
    FileWriteError(FileError),
    ParseIntError(num::ParseIntError),
}
#[derive(Debug)]
pub enum FileError {
    EmptyFileName,
    FileChanged,
}
impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::EmptyFileName => write!(f, "No filename"),
            FileError::FileChanged => write!(f, "file changed"),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::TermError => write!(f, "Terminal error"),
            AppError::Io(e) => write!(f, "I/O error: {}", e),
            AppError::ParseIntError(e) => write!(f, "I/O error: {}", e),
            AppError::FileWriteError(file_error) => write!(f, "File write error: {}", file_error),
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
