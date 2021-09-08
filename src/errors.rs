use thiserror::Error;

pub type Result<T> = std::result::Result<T, LoxInterpreterError>;

#[derive(Debug, Error)]
pub enum LoxInterpreterError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
}
