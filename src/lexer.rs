use crate::errors::{LoxInterpreterError, Result};

#[derive(Debug, PartialEq, Eq)]
pub enum LoxTokenType {
    // single-character tokens
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RIghtBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // one or two character(s) tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // literals
    Identifier(String),
    String(String),
    Number(i64),
    // keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EndOfFile,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LoxToken {
    kind: LoxTokenType,
    lexeme: String,
    // TODO: literal?
    line_number: usize,
}

#[derive(Debug)]
pub struct Lexer {
    source: String,
    tokens: Vec<LoxToken>,
    /// Index in the source of the first character of the lexeme being scanned.
    start: usize,
    /// Index in the source of the current character.
    current: usize,
    /// Current line in the source being scanned.
    line: usize,
}

impl Lexer {
    pub fn from_source(source: String) -> Result<Self> {
        let mut lexer = Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        };
        lexer.scan_tokens();
        Ok(lexer)
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(LoxToken {
            kind: LoxTokenType::EndOfFile,
            lexeme: "".into(),
            line_number: self.line,
        });
    }

    fn scan_token(&mut self) -> Result<()> {
        let char = self.advance();
        match char {
            '(' => self.add_token_with_kind(LoxTokenType::LeftParenthesis),
            ')' => self.add_token_with_kind(LoxTokenType::RightParenthesis),
            '{' => self.add_token_with_kind(LoxTokenType::LeftBrace),
            '}' => self.add_token_with_kind(LoxTokenType::RIghtBrace),
            ',' => self.add_token_with_kind(LoxTokenType::Comma),
            '.' => self.add_token_with_kind(LoxTokenType::Dot),
            '-' => self.add_token_with_kind(LoxTokenType::Minus),
            '+' => self.add_token_with_kind(LoxTokenType::Plus),
            ';' => self.add_token_with_kind(LoxTokenType::Semicolon),
            '*' => self.add_token_with_kind(LoxTokenType::Star),
            _ => Err(LoxInterpreterError::LexerUnexpectedCharacter(
                char.to_string(),
            )),
        }
    }

    fn add_token_with_kind(&mut self, kind: LoxTokenType) -> Result<()> {
        let lexeme = self.source[self.start..self.current].to_string();
        self.tokens.push(LoxToken {
            kind,
            lexeme,
            line_number: self.line,
        });
        Ok(())
    }

    fn advance(&mut self) -> char {
        let char = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        char
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
