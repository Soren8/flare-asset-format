use thiserror::Error;

pub type Result<T> = std::result::Result<T, FlareError>;

#[derive(Debug, Error)]
pub enum FlareError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("parse error in {file}:{line}: {message}")]
    Parse {
        file: String,
        line: usize,
        message: String,
    },

    #[error("invalid value at {file}:{line}: {message}")]
    InvalidValue {
        file: String,
        line: usize,
        message: String,
    },

    #[error("recursive INCLUDE detected for {0}")]
    RecursiveInclude(String),

    #[error("{0}")]
    Other(String),
}

impl FlareError {
    pub fn parse(file: impl Into<String>, line: usize, message: impl Into<String>) -> Self {
        Self::Parse {
            file: file.into(),
            line,
            message: message.into(),
        }
    }

    pub fn invalid(file: impl Into<String>, line: usize, message: impl Into<String>) -> Self {
        Self::InvalidValue {
            file: file.into(),
            line,
            message: message.into(),
        }
    }
}
