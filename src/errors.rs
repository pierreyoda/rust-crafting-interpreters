use thiserror::Error;

use crate::lexer::LoxToken;

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
    #[error("Parse error")]
    ParserError(LoxToken, String),
    #[error("Parse error: unexpected operation")]
    ParserUnexpectedOperation,
    #[error("Unexpected operation: {0}")]
    InterpreterUnexpectedOperation(String),
    #[error("Not a number: {0}")]
    InterpreterNotANumber(String),
}
