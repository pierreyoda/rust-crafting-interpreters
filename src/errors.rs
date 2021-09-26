use thiserror::Error;

pub type Result<T> = std::result::Result<T, LoxInterpreterError>;

#[derive(Debug, Error)]
pub enum LoxInterpreterError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Unterminated string")]
    LexerUnterminatedString,
    #[error("Invalid number: {0}")]
    LexerInvalidNumber(String),
    #[error("Unexpected character at line {0}")]
    LexerUnexpectedCharacter(String),
}
