use crate::errors::BResult;

use super::LoxBytecodeChunk;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LoxBytecodeTokenType {
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
    Identifier,
    String,
    Number,
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

    Error,
    EndOfFile,
}

#[derive(Clone, Debug)]
pub struct LoxBytecodeToken {
    kind: LoxBytecodeTokenType,
    start: usize,
    length: usize,
    line_number: usize,
    error_message: Option<&'static str>,
}

impl LoxBytecodeToken {
    pub fn get_kind(&self) -> &LoxBytecodeTokenType {
        &self.kind
    }

    pub fn get_start(&self) -> usize {
        self.start
    }

    pub fn get_length(&self) -> usize {
        self.length
    }

    pub fn get_line_number(&self) -> usize {
        self.line_number
    }

    pub fn get_lexeme<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.start + self.length]
    }
}

#[derive(Default)]
pub struct LoxBytecodeLexer {
    /// Start index of the lexeme currently being scanned.
    start: usize,
    /// Index of the current character being looked at.
    current: usize,
    /// Current line number.
    line_number: usize,
}

impl LoxBytecodeLexer {
    pub fn compile(&mut self, source: &str) -> BResult<()> {
        self.line_number = 0;
        loop {
            let token = self.scan_token(source);
        }
    }

    pub fn scan_token(&mut self, source: &str) -> BResult<LoxBytecodeToken> {
        self.skip_whitespace(source);
        self.start = self.current;
        if self.is_at_end(source) {
            return Ok(self.build_token(LoxBytecodeTokenType::EndOfFile));
        }

        let char = self.advance(source);
        if Self::is_digit(char) {
            return self.handle_number(source);
        }
        Ok(match char {
            '(' => self.build_token(LoxBytecodeTokenType::LeftParenthesis),
            ')' => self.build_token(LoxBytecodeTokenType::RightParenthesis),
            '{' => self.build_token(LoxBytecodeTokenType::LeftBrace),
            '}' => self.build_token(LoxBytecodeTokenType::RightBrace),
            ';' => self.build_token(LoxBytecodeTokenType::Semicolon),
            ',' => self.build_token(LoxBytecodeTokenType::Comma),
            '.' => self.build_token(LoxBytecodeTokenType::Dot),
            '-' => self.build_token(LoxBytecodeTokenType::Minus),
            '+' => self.build_token(LoxBytecodeTokenType::Plus),
            '/' => self.build_token(LoxBytecodeTokenType::Slash),
            '*' => self.build_token(LoxBytecodeTokenType::Star),
            '!' => {
                if self.match_char(source, '=') {
                    self.build_token(LoxBytecodeTokenType::BangEqual)
                } else {
                    self.build_token(LoxBytecodeTokenType::Bang)
                }
            }
            '=' => {
                if self.match_char(source, '=') {
                    self.build_token(LoxBytecodeTokenType::EqualEqual)
                } else {
                    self.build_token(LoxBytecodeTokenType::Equal)
                }
            }
            '<' => {
                if self.match_char(source, '=') {
                    self.build_token(LoxBytecodeTokenType::LessEqual)
                } else {
                    self.build_token(LoxBytecodeTokenType::Less)
                }
            }
            '>' => {
                if self.match_char(source, '=') {
                    self.build_token(LoxBytecodeTokenType::GreaterEqual)
                } else {
                    self.build_token(LoxBytecodeTokenType::Greater)
                }
            }
            '"' => self.handle_string(source),
            _ => self.build_token_error("Unexpected character."),
        })
    }

    fn handle_identifier(&mut self, source: &str) -> LoxBytecodeToken {
        while Self::is_alpha(self.peek(source)) || Self::is_digit(self.peek(source)) {
            self.advance(source);
        }
        self.build_token(self.identifier_type(source))
    }

    fn identifier_type(&self, source: &str) -> LoxBytecodeTokenType {
        match self.peek(source) {
            'a' => self.check_keyword(source, 1, 2, "nd", LoxBytecodeTokenType::And),
            'c' => self.check_keyword(source, 1, 4, "lass", LoxBytecodeTokenType::Class),
            'e' => self.check_keyword(source, 1, 3, "lse", LoxBytecodeTokenType::Else),
            'f' => {
                if self.current - self.start > 1 {
                    match self.peek_next(source) {
                        Some('a') => {
                            self.check_keyword(source, 2, 3, "lse", LoxBytecodeTokenType::False)
                        }
                        Some('o') => {
                            self.check_keyword(source, 2, 1, "r", LoxBytecodeTokenType::For)
                        }
                        Some('u') => {
                            self.check_keyword(source, 2, 1, "n", LoxBytecodeTokenType::Fun)
                        }
                        _ => LoxBytecodeTokenType::Identifier,
                    }
                } else {
                    LoxBytecodeTokenType::Identifier
                }
            }
            'i' => self.check_keyword(source, 1, 1, "f", LoxBytecodeTokenType::If),
            'n' => self.check_keyword(source, 1, 2, "il", LoxBytecodeTokenType::Nil),
            'o' => self.check_keyword(source, 1, 1, "r", LoxBytecodeTokenType::Or),
            'p' => self.check_keyword(source, 1, 4, "rint", LoxBytecodeTokenType::Print),
            'r' => self.check_keyword(source, 1, 5, "eturn", LoxBytecodeTokenType::Return),
            's' => self.check_keyword(source, 1, 4, "uper", LoxBytecodeTokenType::Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.peek_next(source) {
                        Some('h') => {
                            self.check_keyword(source, 2, 2, "is", LoxBytecodeTokenType::This)
                        }
                        Some('r') => {
                            self.check_keyword(source, 2, 2, "ue", LoxBytecodeTokenType::True)
                        }
                        _ => LoxBytecodeTokenType::Identifier,
                    }
                } else {
                    LoxBytecodeTokenType::Identifier
                }
            }
            'v' => self.check_keyword(source, 1, 2, "ar", LoxBytecodeTokenType::Var),
            'w' => self.check_keyword(source, 1, 4, "hile", LoxBytecodeTokenType::While),
            _ => LoxBytecodeTokenType::Identifier,
        }
    }

    fn check_keyword(
        &self,
        source: &str,
        start: usize,
        length: usize,
        rest: &str,
        kind: LoxBytecodeTokenType,
    ) -> LoxBytecodeTokenType {
        let index = self.start + start;
        if self.current - self.start == start + length && source[index..index + length] == *rest {
            kind
        } else {
            LoxBytecodeTokenType::Identifier
        }
    }

    fn handle_string(&mut self, source: &str) -> LoxBytecodeToken {
        while self.peek(source) != '"' && !self.is_at_end(source) {
            if self.peek(source) == '\n' {
                self.line_number += 1;
            }
            self.advance(source);
        }

        if self.is_at_end(source) {
            self.build_token_error("Unterminated string.")
        } else {
            self.advance(source);
            self.build_token(LoxBytecodeTokenType::String)
        }
    }

    fn handle_number(&mut self, source: &str) -> BResult<LoxBytecodeToken> {
        while Self::is_digit(self.peek(source)) {
            self.advance(source);
        }

        // look for a fractional part
        if self.peek(source) == '.'
            && Self::is_digit(
                self.peek_next(source)
                    .expect("compiler expects a digit after number fractional separator '.'"),
            )
        {
            self.advance(source); // consume the '.'
            while Self::is_digit(self.peek(source)) {
                self.advance(source);
            }
        }

        Ok(self.build_token(LoxBytecodeTokenType::Number))
    }

    fn build_token(&self, kind: LoxBytecodeTokenType) -> LoxBytecodeToken {
        LoxBytecodeToken {
            kind,
            start: self.start,
            length: self.current - self.start,
            line_number: self.line_number,
            error_message: None,
        }
    }

    fn build_token_error(&self, message: &'static str) -> LoxBytecodeToken {
        LoxBytecodeToken {
            kind: LoxBytecodeTokenType::Error,
            start: self.start,
            length: message.len(),
            line_number: self.line_number,
            error_message: Some(message),
        }
    }

    /// Skip over whitespace, line breaks and comments.
    fn skip_whitespace(&mut self, source: &str) {
        loop {
            match self.peek(source) {
                ' ' | '\r' | '\t' => {
                    self.advance(source);
                }
                '\n' => {
                    self.line_number += 1;
                    self.advance(source);
                }
                '/' => {
                    if let Some(next) = self.peek_next(source) {
                        if next == '/' {
                            while self.peek(source) != '\n' && !self.is_at_end(source) {
                                self.advance(source);
                            }
                        } else {
                            return;
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn advance(&mut self, source: &str) -> char {
        self.current += 1;
        source
            .chars()
            .nth(self.current - 1)
            .expect("compiler expects a character")
    }

    fn match_char(&mut self, source: &str, expected: char) -> bool {
        if self.is_at_end(source) || self.peek(source) != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek(&self, source: &str) -> char {
        source
            .chars()
            .nth(self.current)
            .expect("compiler expects a character at current index")
    }

    fn peek_next(&self, source: &str) -> Option<char> {
        source.chars().nth(self.current + 1)
    }

    fn is_at_end(&self, source: &str) -> bool {
        self.current >= source.len()
    }

    fn is_alpha(char: char) -> bool {
        (char >= 'a' && char <= 'z') || (char >= 'A' && char <= 'Z') || char == '_'
    }

    fn is_digit(char: char) -> bool {
        char >= '0' && char <= '9'
    }
}
