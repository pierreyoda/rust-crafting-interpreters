use std::collections::HashMap;

use crate::{
    errors::{LoxInterpreterError, Result},
    expressions::LoxLiteral,
};

#[derive(Clone, Debug, PartialEq)]
pub enum LoxTokenType {
    // single-character tokens
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
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
    Number(f64),
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

impl LoxTokenType {
    pub fn is_string(&self) -> bool {
        matches!(self, LoxTokenType::String(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, LoxTokenType::Number(_))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoxToken {
    kind: LoxTokenType,
    lexeme: String,
    line_number: usize,
}

impl LoxToken {
    pub fn get_kind(&self) -> &LoxTokenType {
        &self.kind
    }

    pub fn build_literal(&self) -> Option<LoxLiteral> {
        match &self.kind {
            LoxTokenType::String(string) => Some(LoxLiteral::String(string.clone())),
            LoxTokenType::Number(number) => Some(LoxLiteral::Number(*number)),
            LoxTokenType::True => Some(LoxLiteral::True),
            LoxTokenType::False => Some(LoxLiteral::False),
            LoxTokenType::Nil => Some(LoxLiteral::Nil),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Lexer {
    keywords: HashMap<&'static str, LoxTokenType>,
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
        let mut keywords = HashMap::new();
        keywords.insert("and", LoxTokenType::And);
        keywords.insert("class", LoxTokenType::Class);
        keywords.insert("else", LoxTokenType::Else);
        keywords.insert("false", LoxTokenType::False);
        keywords.insert("for", LoxTokenType::For);
        keywords.insert("fun", LoxTokenType::Fun);
        keywords.insert("if", LoxTokenType::If);
        keywords.insert("nil", LoxTokenType::Nil);
        keywords.insert("or", LoxTokenType::Or);
        keywords.insert("print", LoxTokenType::Print);
        keywords.insert("return", LoxTokenType::Return);
        keywords.insert("super", LoxTokenType::Super);
        keywords.insert("this", LoxTokenType::This);
        keywords.insert("true", LoxTokenType::True);
        keywords.insert("var", LoxTokenType::Var);
        keywords.insert("while", LoxTokenType::While);

        let mut lexer = Self {
            keywords,
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        };
        lexer.scan_tokens()?;
        Ok(lexer)
    }

    fn scan_tokens(&mut self) -> Result<()> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(LoxToken {
            kind: LoxTokenType::EndOfFile,
            lexeme: "".into(),
            line_number: self.line,
        });
        Ok(())
    }

    fn scan_token(&mut self) -> Result<()> {
        let char = self.advance();
        match char {
            '(' => self.add_token_with_kind(LoxTokenType::LeftParenthesis),
            ')' => self.add_token_with_kind(LoxTokenType::RightParenthesis),
            '{' => self.add_token_with_kind(LoxTokenType::LeftBrace),
            '}' => self.add_token_with_kind(LoxTokenType::RightBrace),
            ',' => self.add_token_with_kind(LoxTokenType::Comma),
            '.' => self.add_token_with_kind(LoxTokenType::Dot),
            '-' => self.add_token_with_kind(LoxTokenType::Minus),
            '+' => self.add_token_with_kind(LoxTokenType::Plus),
            ';' => self.add_token_with_kind(LoxTokenType::Semicolon),
            '*' => self.add_token_with_kind(LoxTokenType::Star),
            '!' => {
                if self.advance_if_match('=') {
                    self.add_token_with_kind(LoxTokenType::BangEqual)
                } else {
                    self.add_token_with_kind(LoxTokenType::Bang)
                }
            }
            '=' => {
                if self.advance_if_match('=') {
                    self.add_token_with_kind(LoxTokenType::EqualEqual)
                } else {
                    self.add_token_with_kind(LoxTokenType::Equal)
                }
            }
            '<' => {
                if self.advance_if_match('=') {
                    self.add_token_with_kind(LoxTokenType::LessEqual)
                } else {
                    self.add_token_with_kind(LoxTokenType::Less)
                }
            }
            '>' => {
                if self.advance_if_match('=') {
                    self.add_token_with_kind(LoxTokenType::GreaterEqual)
                } else {
                    self.add_token_with_kind(LoxTokenType::Greater)
                }
            }
            '/' => {
                if self.advance_if_match('/') {
                    // a comment goes until the end of the line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    Ok(())
                } else {
                    self.add_token_with_kind(LoxTokenType::Slash)
                }
            }
            ' ' | '\r' | '\t' => Ok(()),
            '\n' => {
                self.line += 1;
                Ok(())
            }
            '"' => {
                let mut current_character = self.peek();
                while current_character != '"' && !self.is_at_end() {
                    if current_character == '\n' {
                        self.line += 1;
                    }
                    self.advance();
                    current_character = self.peek();
                }

                if self.is_at_end() {
                    Err(LoxInterpreterError::LexerUnterminatedString)
                } else {
                    self.advance(); // the closing "
                    let value = self.source[self.start + 1..self.current - 1].to_string(); // trim the surrounding quotes
                    self.add_token_with_kind(LoxTokenType::String(value))
                }
            }
            _ => {
                if Self::is_digit(char) {
                    self.handle_number()
                } else if Self::is_alpha(char) {
                    self.handle_identifier()
                } else {
                    Err(LoxInterpreterError::LexerUnexpectedCharacter(
                        char.to_string(),
                    ))
                }
            }
        }
    }

    fn handle_number(&mut self) -> Result<()> {
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        // look for a fractional part
        if self.peek() == '.' && Self::is_digit(self.peek_next()) {
            self.advance(); // consume the .
            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }

        let raw = &self.source[self.start..self.current];
        let value = raw
            .parse()
            .map_err(|_| LoxInterpreterError::LexerInvalidNumber(raw.to_string()))?;
        self.add_token_with_kind(LoxTokenType::Number(value));

        Ok(())
    }

    fn handle_identifier(&mut self) -> Result<()> {
        while Self::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let kind = self
            .keywords
            .get(text)
            .cloned()
            .unwrap_or(LoxTokenType::Identifier(text.to_string()));
        self.add_token_with_kind(kind);

        Ok(())
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

    fn advance_if_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if let Some(current_character) = self.source.chars().nth(self.current) {
            if current_character == expected {
                self.current += 1;
                return true;
            }
        }
        false
    }

    fn peek(&self) -> char {
        if self.current >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        let next = self.current + 1;
        if next >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(next).unwrap()
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn is_digit(char: char) -> bool {
        char >= '0' && char <= '9'
    }

    fn is_alpha(char: char) -> bool {
        (char >= 'a' && char <= 'z') || (char >= 'A' && char <= 'Z') || char == '_'
    }

    fn is_alphanumeric(char: char) -> bool {
        Self::is_digit(char) || Self::is_alpha(char)
    }
}
