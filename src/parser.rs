use crate::{
    errors::{LoxInterpreterError, Result},
    expressions::{LoxExpression, LoxLiteral, LoxOperation, LoxStatement},
    lexer::{LoxToken, LoxTokenType},
};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<LoxToken>,
    /// Index of the current token.
    current: usize,
}

impl Parser {
    pub fn from_tokens(tokens: Vec<LoxToken>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<LoxOperation>> {
        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.handle_declaration()?);
        }
        Ok(statements)
    }

    /// Discards tokens until a probable statement boundary is found.
    ///
    /// Used to avoid cascade errors when encountering a parse error.
    fn synchronize(&mut self) -> Result<()> {
        self.advance();
        while !self.is_at_end() {
            if self.peek_previous().get_kind() == &LoxTokenType::Semicolon
                || matches!(
                    self.peek().get_kind(),
                    LoxTokenType::Class
                        | LoxTokenType::Fun
                        | LoxTokenType::Var
                        | LoxTokenType::For
                        | LoxTokenType::If
                        | LoxTokenType::While
                        | LoxTokenType::Print
                        | LoxTokenType::Return
                )
            {
                return Ok(());
            }
            self.advance();
        }
        Ok(())
    }

    /// If the current token matches any of the given token types, consume it and return true.
    fn match_kinds(&mut self, kinds: &[LoxTokenType]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// If the current token matches the given token type, consume it and return it.
    ///
    /// Otherwise, return an error.
    fn consume_kind(&mut self, kind: &LoxTokenType, error_message: &str) -> Result<&LoxToken> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(Self::build_parse_error(self.peek(), error_message))
        }
    }

    /// If the current token is an identifier, consume it and return it.
    ///
    /// Otherwise, return an error.
    fn consume_identifier(&mut self, error_message: &str) -> Result<&LoxToken> {
        if !self.is_at_end() && self.peek().get_kind().is_identifier() {
            Ok(self.advance())
        } else {
            Err(Self::build_parse_error(self.peek(), error_message))
        }
    }

    /// If the current token is an identifier, consume it and return true.
    fn match_identifier(&mut self) -> bool {
        if self.is_at_end() || !self.peek().get_kind().is_identifier() {
            false
        } else {
            self.advance();
            true
        }
    }

    /// If the current token is a string literal, consume it and return true.
    fn match_string(&mut self) -> bool {
        if self.is_at_end() || !self.peek().get_kind().is_string() {
            false
        } else {
            self.advance();
            true
        }
    }

    /// If the current token is a number literal, consume it and return true.
    fn match_number(&mut self) -> bool {
        if self.is_at_end() || !self.peek().get_kind().is_number() {
            false
        } else {
            self.advance();
            true
        }
    }

    /// Consumes the current token and returns it.
    fn advance(&mut self) -> &LoxToken {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.peek_previous()
    }

    /// Returns true if the current token is of the given token type.
    fn check(&self, kind: &LoxTokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().get_kind() == kind
        }
    }

    /// Returns the previous token (assumes that the `current - 1` index is in bounds).
    fn peek_previous(&self) -> &LoxToken {
        self.tokens.get(self.current - 1).unwrap()
    }

    /// Returns the current token (assumes that the `current` index is in bounds).
    fn peek(&self) -> &LoxToken {
        self.tokens.get(self.current).unwrap()
    }

    /// Are we currently at the final (end of file) token?
    fn is_at_end(&self) -> bool {
        self.peek().get_kind() == &LoxTokenType::EndOfFile
    }

    fn build_parse_error(token: &LoxToken, message: &str) -> LoxInterpreterError {
        LoxInterpreterError::ParserError(token.clone(), message.to_string())
    }

    fn handle_declaration(&mut self) -> Result<LoxOperation> {
        let mut inner_parsing = || -> Result<LoxOperation> {
            if self.match_kinds(&[LoxTokenType::Var]) {
                self.handle_variable_declaration()
            } else {
                self.handle_statement()
            }
        };

        if let Ok(declaration) = inner_parsing() {
            Ok(declaration)
        } else {
            self.synchronize();
            Ok(LoxOperation::Invalid)
        }
    }

    fn handle_variable_declaration(&mut self) -> Result<LoxOperation> {
        let name = self.consume_identifier("Expect variable name.")?.clone();
        let initializer = if self.match_kinds(&[LoxTokenType::Equal]) {
            self.handle_expression()
        } else {
            Err(Self::build_parse_error(self.peek(), "Expect '=' symbol."))
        }?
        .as_expression()?;
        let _ = self.consume_kind(
            &LoxTokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(LoxOperation::Statement(LoxStatement::Variable {
            name,
            initializer,
        }))
    }

    fn handle_statement(&mut self) -> Result<LoxOperation> {
        if self.match_kinds(&[LoxTokenType::Print]) {
            self.handle_print_statement()
        } else {
            self.handle_expression()
        }
    }

    fn handle_print_statement(&mut self) -> Result<LoxOperation> {
        let expression = self.handle_expression()?.as_expression()?;
        let _ = self.consume_kind(&LoxTokenType::Semicolon, "Expect ';' after value.")?;
        Ok(LoxOperation::Statement(LoxStatement::Print { expression }))
    }

    fn handle_expression(&mut self) -> Result<LoxOperation> {
        Ok(LoxOperation::Expression(self.handle_equality()?))
    }

    fn handle_equality(&mut self) -> Result<LoxExpression> {
        let mut expression = self.handle_comparison()?;
        let kinds = [LoxTokenType::Equal, LoxTokenType::EqualEqual];
        while self.match_kinds(&kinds) {
            let operator = self.peek_previous().clone();
            let right = self.handle_comparison()?;
            expression = LoxExpression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn handle_comparison(&mut self) -> Result<LoxExpression> {
        let mut expression = self.handle_term()?;
        let kinds = [
            LoxTokenType::Greater,
            LoxTokenType::GreaterEqual,
            LoxTokenType::Less,
            LoxTokenType::LessEqual,
        ];
        while self.match_kinds(&kinds) {
            let operator = self.peek_previous().clone();
            let right = self.handle_term()?;
            expression = LoxExpression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn handle_term(&mut self) -> Result<LoxExpression> {
        let mut expression = self.handle_factor()?;
        let kinds = [LoxTokenType::Minus, LoxTokenType::Plus];
        while self.match_kinds(&kinds) {
            let operator = self.peek_previous().clone();
            let right = self.handle_factor()?;
            expression = LoxExpression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn handle_factor(&mut self) -> Result<LoxExpression> {
        let mut expression = self.handle_unary()?;
        let kinds = [LoxTokenType::Slash, LoxTokenType::Star];
        while self.match_kinds(&kinds) {
            let operator = self.peek_previous().clone();
            let right = self.handle_unary()?;
            expression = LoxExpression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn handle_unary(&mut self) -> Result<LoxExpression> {
        Ok(
            if self.match_kinds(&[LoxTokenType::Bang, LoxTokenType::Minus]) {
                let operator = self.peek_previous().clone();
                let right = self.handle_unary()?;
                LoxExpression::Unary {
                    operator,
                    right: Box::new(right),
                }
            } else {
                self.handle_primary()?
            },
        )
    }

    fn handle_primary(&mut self) -> Result<LoxExpression> {
        if self.match_kinds(&[LoxTokenType::False]) {
            Ok(LoxExpression::Literal {
                value: LoxLiteral::False,
            })
        } else if self.match_kinds(&[LoxTokenType::True]) {
            Ok(LoxExpression::Literal {
                value: LoxLiteral::True,
            })
        } else if self.match_kinds(&[LoxTokenType::Nil]) {
            Ok(LoxExpression::Literal {
                value: LoxLiteral::Nil,
            })
        } else if self.match_number() || self.match_string() {
            let value = self.peek_previous().build_literal().unwrap();
            Ok(LoxExpression::Literal { value })
        } else if self.match_identifier() {
            Ok(LoxExpression::Variable {
                name: self.peek_previous().clone(),
            })
        } else if self.match_kinds(&[LoxTokenType::LeftParenthesis]) {
            let expression = self.handle_expression()?.as_expression()?;
            self.consume_kind(
                &LoxTokenType::RightParenthesis,
                "Expect ')' after expression.",
            )?;
            Ok(LoxExpression::Group {
                expression: Box::new(expression),
            })
        } else {
            Err(Self::build_parse_error(self.peek(), "Expect expression."))
        }
    }
}
