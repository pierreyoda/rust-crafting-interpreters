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
        let mut operations = vec![];
        while !self.is_at_end() {
            operations.push(self.handle_declaration()?);
        }
        Ok(operations)
    }

    /// Discards tokens until a probable statement boundary is found.
    ///
    /// Used to avoid cascade errors when encountering a parse error.
    fn synchronize(&mut self) {
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
                return;
            }
            self.advance();
        }
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
            if self.match_kinds(&[LoxTokenType::Class]) {
                self.handle_class_declaration()
            } else if self.match_kinds(&[LoxTokenType::Fun]) {
                self.handle_function_declaration("function")
            } else if self.match_kinds(&[LoxTokenType::Var]) {
                self.handle_variable_declaration()
            } else {
                self.handle_statement()
            }
        };

        match inner_parsing() {
            Ok(declaration) => Ok(declaration),
            Err(why) => {
                self.synchronize();
                // TODO: improve error reporting (line number, etc.)
                println!("{}: {:?}", why, why);
                Ok(LoxOperation::Invalid)
            }
        }
    }

    fn handle_class_declaration(&mut self) -> Result<LoxOperation> {
        let name = self.consume_identifier("Expect class name.")?.clone();
        let _ = self.consume_kind(&LoxTokenType::LeftBrace, "Expect '{' before class body.")?;
        let mut methods = vec![];
        while !self.check(&LoxTokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.handle_function_declaration("method")?.as_statement()?);
        }
        let _ = self.consume_kind(&LoxTokenType::RightBrace, "Expect '}' before class body.")?;
        Ok(LoxOperation::Statement(LoxStatement::Class {
            name,
            methods,
            super_class: LoxExpression::NoOp,
        }))
    }

    fn handle_function_declaration(&mut self, kind: &str) -> Result<LoxOperation> {
        let name = self
            .consume_identifier(format!("Expect {} name.", kind).as_str())?
            .clone();
        let _ = self.consume_kind(
            &LoxTokenType::LeftParenthesis,
            format!("Expect '(' after {} name.", kind).as_str(),
        )?;
        let mut parameters = vec![];
        if !self.check(&LoxTokenType::RightParenthesis) {
            parameters.push(self.consume_identifier("Expect parameter name.")?.clone());
            while self.match_kinds(&[LoxTokenType::Comma]) {
                if parameters.len() >= 255 {
                    // TODO: better error reporting
                    println!(
                        "{:?}",
                        Self::build_parse_error(
                            self.peek(),
                            "Can't have more than 255 parameters."
                        )
                    );
                }
                parameters.push(self.consume_identifier("Expect parameter name.")?.clone());
            }
        }
        let _ = self.consume_kind(
            &LoxTokenType::RightParenthesis,
            "Expect ')' after parameters.",
        )?;
        let _ = self.consume_kind(
            &LoxTokenType::LeftBrace,
            format!("Expect '{{' before {} body.", kind).as_str(),
        )?;
        let body = self.handle_statements_block()?;
        Ok(LoxOperation::Statement(LoxStatement::Function {
            name,
            parameters,
            body,
        }))
    }

    fn handle_variable_declaration(&mut self) -> Result<LoxOperation> {
        let name = self.consume_identifier("Expect variable name.")?.clone();
        let initializer = if self.match_kinds(&[LoxTokenType::Equal]) {
            self.handle_expression()?.as_expression()?
        } else {
            LoxExpression::NoOp
        };
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
        if self.match_kinds(&[LoxTokenType::For]) {
            self.handle_for_statement()
        } else if self.match_kinds(&[LoxTokenType::If]) {
            self.handle_if_statement()
        } else if self.match_kinds(&[LoxTokenType::Print]) {
            self.handle_print_statement()
        } else if self.match_kinds(&[LoxTokenType::Return]) {
            self.handle_return_statement()
        } else if self.match_kinds(&[LoxTokenType::While]) {
            self.handle_while_statement()
        } else if self.match_kinds(&[LoxTokenType::LeftBrace]) {
            Ok(LoxOperation::Statement(LoxStatement::Block {
                statements: self.handle_statements_block()?,
            }))
        } else {
            self.handle_expression_statement()
        }
    }

    fn handle_if_statement(&mut self) -> Result<LoxOperation> {
        let _ = self.consume_kind(&LoxTokenType::LeftParenthesis, "Expect '(' after 'if'.")?;
        let condition = self.handle_expression()?.as_expression()?;
        let _ = self.consume_kind(
            &LoxTokenType::RightParenthesis,
            "Expect ')' after if condition.",
        )?;
        let then_branch = self.handle_statement()?.as_statement()?;
        let else_branch = if self.match_kinds(&[LoxTokenType::Else]) {
            self.handle_statement()?.as_statement()?
        } else {
            LoxStatement::NoOp
        };
        Ok(LoxOperation::Statement(LoxStatement::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        }))
    }

    fn handle_while_statement(&mut self) -> Result<LoxOperation> {
        let _ = self.consume_kind(&LoxTokenType::LeftParenthesis, "Expect '(' after 'while'.")?;
        let condition = self.handle_expression()?.as_expression()?;
        let _ = self.consume_kind(
            &LoxTokenType::RightParenthesis,
            "Expect ')' after condition.",
        )?;
        let body = self.handle_statement()?.as_statement()?;
        Ok(LoxOperation::Statement(LoxStatement::While {
            condition,
            body: Box::new(body),
        }))
    }

    fn handle_for_statement(&mut self) -> Result<LoxOperation> {
        let _ = self.consume_kind(&LoxTokenType::LeftParenthesis, "Expect '(' after 'for'.")?;
        // initializer
        let initializer = if self.match_kinds(&[LoxTokenType::Semicolon]) {
            LoxStatement::NoOp
        } else if self.match_kinds(&[LoxTokenType::Var]) {
            self.handle_variable_declaration()?.as_statement()?
        } else {
            self.handle_expression_statement()?.as_statement()?
        };
        // condition
        let condition = if self.check(&LoxTokenType::Semicolon) {
            LoxExpression::NoOp
        } else {
            self.handle_expression()?.as_expression()?
        };
        let _ = self.consume_kind(&LoxTokenType::Semicolon, "Expect ';' after loop condition.")?;
        // increment
        let increment = if self.check(&LoxTokenType::RightParenthesis) {
            LoxExpression::NoOp
        } else {
            self.handle_expression()?.as_expression()?
        };
        let _ = self.consume_kind(
            &LoxTokenType::RightParenthesis,
            "Expect ')' after for clauses.",
        )?;
        // body
        let mut body = self.handle_statement()?;

        // 'for' statement syntax desugaring
        if !increment.is_noop() {
            body = LoxOperation::Statement(LoxStatement::Block {
                statements: vec![
                    body.as_statement()?,
                    LoxStatement::Expression {
                        expression: increment,
                    },
                ],
            })
        }
        body = LoxOperation::Statement(LoxStatement::While {
            condition: if condition.is_noop() {
                LoxExpression::Literal {
                    value: LoxLiteral::True,
                }
            } else {
                condition
            },
            body: Box::new(body.as_statement()?),
        });
        if !initializer.is_noop() {
            body = LoxOperation::Statement(LoxStatement::Block {
                statements: vec![initializer, body.as_statement()?],
            });
        }

        Ok(body)
    }

    fn handle_print_statement(&mut self) -> Result<LoxOperation> {
        let expression = self.handle_expression()?.as_expression()?;
        let _ = self.consume_kind(&LoxTokenType::Semicolon, "Expect ';' after value.")?;
        Ok(LoxOperation::Statement(LoxStatement::Print { expression }))
    }

    fn handle_return_statement(&mut self) -> Result<LoxOperation> {
        let keyword = self.peek_previous().clone();
        let value = if self.check(&LoxTokenType::Semicolon) {
            LoxExpression::NoOp
        } else {
            self.handle_expression()?.as_expression()?
        };
        let _ = self.consume_kind(&LoxTokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(LoxOperation::Statement(LoxStatement::Return {
            keyword,
            value,
        }))
    }

    fn handle_statements_block(&mut self) -> Result<Vec<LoxStatement>> {
        let mut statements = vec![];
        while !self.check(&LoxTokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.handle_declaration()?.as_statement()?);
        }
        let _ = self.consume_kind(&LoxTokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn handle_expression_statement(&mut self) -> Result<LoxOperation> {
        let expression = self.handle_expression()?.as_expression()?;
        let _ = self.consume_kind(&LoxTokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(LoxOperation::Statement(LoxStatement::Expression {
            expression,
        }))
    }

    fn handle_expression(&mut self) -> Result<LoxOperation> {
        Ok(LoxOperation::Expression(self.handle_assignment()?))
    }

    fn handle_assignment(&mut self) -> Result<LoxExpression> {
        let expression = self.handle_or()?;
        if self.match_kinds(&[LoxTokenType::Equal]) {
            let equals = self.peek_previous().clone();
            let value = self.handle_assignment()?;
            match &expression {
                LoxExpression::Variable { name } => Ok(LoxExpression::Assign {
                    name: name.clone(),
                    value: Box::new(value),
                }),
                LoxExpression::Get { name, object } => Ok(LoxExpression::Set {
                    name: name.clone(),
                    object: object.clone(),
                    value: Box::new(value),
                }),
                _ => Err(Self::build_parse_error(
                    &equals,
                    "Invalid assignment target.",
                )),
            }
        } else {
            Ok(expression)
        }
    }

    fn handle_or(&mut self) -> Result<LoxExpression> {
        let mut expression = self.handle_and()?;
        while self.match_kinds(&[LoxTokenType::Or]) {
            let operator = self.peek_previous().clone();
            let right = self.handle_and()?;
            expression = LoxExpression::Logical {
                operator,
                left: Box::new(expression),
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn handle_and(&mut self) -> Result<LoxExpression> {
        let mut expression = self.handle_equality()?;
        while self.match_kinds(&[LoxTokenType::And]) {
            let operator = self.peek_previous().clone();
            let right = self.handle_equality()?;
            expression = LoxExpression::Logical {
                operator,
                left: Box::new(expression),
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn handle_equality(&mut self) -> Result<LoxExpression> {
        let mut expression = self.handle_comparison()?;
        let kinds = [LoxTokenType::BangEqual, LoxTokenType::EqualEqual];
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
        if self.match_kinds(&[LoxTokenType::Bang, LoxTokenType::Minus]) {
            let operator = self.peek_previous().clone();
            let right = self.handle_unary()?;
            Ok(LoxExpression::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.handle_call()
        }
    }

    fn handle_call(&mut self) -> Result<LoxExpression> {
        let mut expression = self.handle_primary()?;
        loop {
            if self.match_kinds(&[LoxTokenType::LeftParenthesis]) {
                expression = self.finish_call(expression)?;
            } else if self.match_kinds(&[LoxTokenType::Dot]) {
                let name = self
                    .consume_identifier("Expect property name after '.'.")?
                    .clone();
                expression = LoxExpression::Get {
                    name,
                    object: Box::new(expression),
                };
            } else {
                break;
            }
        }
        Ok(expression)
    }

    fn finish_call(&mut self, callee: LoxExpression) -> Result<LoxExpression> {
        let mut arguments = vec![];
        if !self.check(&LoxTokenType::RightParenthesis) {
            arguments.push(self.handle_expression()?.as_expression()?);
            while self.match_kinds(&[LoxTokenType::Comma]) {
                if arguments.len() >= 255 {
                    // TODO: better error reporting
                    println!(
                        "{:?}",
                        Self::build_parse_error(self.peek(), "Can't have more than 255 arguments.")
                    );
                }
                arguments.push(self.handle_expression()?.as_expression()?);
            }
        }
        let parenthesis = self
            .consume_kind(
                &LoxTokenType::RightParenthesis,
                "Expect ')' after arguments.",
            )?
            .clone();
        Ok(LoxExpression::Call {
            callee: Box::new(callee),
            arguments,
            parenthesis,
        })
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
        } else if self.match_kinds(&[LoxTokenType::This]) {
            Ok(LoxExpression::This {
                keyword: self.peek_previous().clone(),
            })
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
