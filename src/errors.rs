use thiserror::Error;

use crate::{lexer::LoxToken, values::LoxValue};

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
    #[error("Parse error: unexpected operation: {0}")]
    ParserUnexpectedOperation(String),
    #[error("Resolver error: unexpected operation: {0}")]
    ResolverUnexpectedOperation(String),
    #[error("Can't read local variable in its own initializer.")]
    ResolverRecursiveLocalAssignment(LoxToken),
    #[error("Already a variable with this name in this scope.")]
    ResolverDuplicateVariableDeclaration(LoxToken),
    #[error("Can't return from top-level code.")]
    ResolverImpossibleTopLevelReturn(LoxToken),
    #[error("Unexpected operation: {0}")]
    InterpreterUnexpectedOperation(String),
    #[error("Not a number: {0}")]
    InterpreterNotANumber(String),
    #[error("Undefined variable '{0}'.")]
    InterpreterUndefinedVariable(String),
    #[error("Can only call functions and classes.")]
    InterpreterNonCallableValue(LoxToken),
    #[error("Expected {0} arguments but got {1}.")]
    InterpreterCallableWrongArity(usize, usize),
    #[error("Return value")]
    InterpreterReturn(LoxValue), // TODO: find a better way
}
