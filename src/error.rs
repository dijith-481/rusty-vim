use std::{fmt, io};
#[derive(Debug)]
pub(crate) enum AppError {
    TermError,
    Io(io::Error),
}
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::TermError => write!(f, "termerror"),
            AppError::Io(e) => write!(f, "{}", e),
        }
    }
}
impl From<io::Error> for AppError {
    fn from(err: io::Error) -> AppError {
        AppError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
