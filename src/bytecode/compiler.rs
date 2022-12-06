use std::collections::HashMap;

use crate::{
    bytecode::lexer::LoxBytecodeTokenType,
    errors::{BResult, LoxBytecodeInterpreterError},
    lexer,
};

use super::{
    debug::disassemble_chunk,
    lexer::{LoxBytecodeLexer, LoxBytecodeToken},
    values::{LoxBytecodeObject, LoxBytecodeValue},
    LoxBytecodeChunk, LoxBytecodeOpcode,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoxBytecodeOperatorPrecedence {
    None = 0,
    Assignment = 1,
    Or = 2,
    And = 3,
    Equality = 4,
    Comparison = 5,
    Term = 6,
    Factor = 7,
    Unary = 8,
    Call = 9,
    Primary = 10,
}

impl LoxBytecodeOperatorPrecedence {
    pub fn from_usize(value: usize) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Assignment),
            2 => Some(Self::Or),
            3 => Some(Self::And),
            4 => Some(Self::Equality),
            5 => Some(Self::Comparison),
            6 => Some(Self::Term),
            7 => Some(Self::Factor),
            8 => Some(Self::Unary),
            9 => Some(Self::Call),
            10 => Some(Self::Primary),
            _ => None,
        }
    }
}

pub type LoxParseFunction = fn(
    &mut LoxBytecodeCompiler,
    source: &str,
    lexer: &mut LoxBytecodeLexer,
    chunk: &mut LoxBytecodeChunk,
) -> BResult<()>;

pub struct LoxParseRule {
    prefix: Option<LoxParseFunction>,
    infix: Option<LoxParseFunction>,
    precedence: LoxBytecodeOperatorPrecedence,
}

pub struct LoxBytecodeTokensParser {
    current: LoxBytecodeToken,
    previous: LoxBytecodeToken,
    had_error: bool,
    panic_mode: bool,
}

/// Takes tokens from the Lexer and transforms them into a chunk of bytecode.
pub struct LoxBytecodeCompiler {
    parser: LoxBytecodeTokensParser,
    parsing_rules: HashMap<LoxBytecodeTokenType, LoxParseRule>,
}

impl LoxBytecodeCompiler {
    pub fn new(source: &str, lexer: &mut LoxBytecodeLexer) -> BResult<Self> {
        // parsing rules
        // TODO: use a macro here for terseness
        let mut parsing_rules = HashMap::new();
        parsing_rules.insert(
            LoxBytecodeTokenType::LeftParenthesis,
            LoxParseRule {
                prefix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_grouping(source, lexer, chunk)
                }),
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::RightParenthesis,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::LeftBrace,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::RightBrace,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Comma,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Dot,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Minus,
            LoxParseRule {
                prefix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_unary(source, lexer, chunk)
                }),
                infix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_binary(source, lexer, chunk)
                }),
                precedence: LoxBytecodeOperatorPrecedence::Term,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Plus,
            LoxParseRule {
                prefix: None,
                infix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_binary(source, lexer, chunk)
                }),
                precedence: LoxBytecodeOperatorPrecedence::Term,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Semicolon,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Slash,
            LoxParseRule {
                prefix: None,
                infix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_binary(source, lexer, chunk)
                }),
                precedence: LoxBytecodeOperatorPrecedence::Factor,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Star,
            LoxParseRule {
                prefix: None,
                infix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_binary(source, lexer, chunk)
                }),
                precedence: LoxBytecodeOperatorPrecedence::Factor,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Bang,
            LoxParseRule {
                prefix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_unary(source, lexer, chunk)
                }),
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::BangEqual,
            LoxParseRule {
                prefix: None,
                infix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_binary(source, lexer, chunk)
                }),
                precedence: LoxBytecodeOperatorPrecedence::Equality,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Equal,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::EqualEqual,
            LoxParseRule {
                prefix: None,
                infix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_binary(source, lexer, chunk)
                }),
                precedence: LoxBytecodeOperatorPrecedence::Equality,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Greater,
            LoxParseRule {
                prefix: None,
                infix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_binary(source, lexer, chunk)
                }),
                precedence: LoxBytecodeOperatorPrecedence::Comparison,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::GreaterEqual,
            LoxParseRule {
                prefix: None,
                infix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_binary(source, lexer, chunk)
                }),
                precedence: LoxBytecodeOperatorPrecedence::Comparison,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Less,
            LoxParseRule {
                prefix: None,
                infix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_binary(source, lexer, chunk)
                }),
                precedence: LoxBytecodeOperatorPrecedence::Comparison,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::LessEqual,
            LoxParseRule {
                prefix: None,
                infix: Some(|compiler, source, lexer, chunk| {
                    compiler.handle_binary(source, lexer, chunk)
                }),
                precedence: LoxBytecodeOperatorPrecedence::Comparison,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Identifier,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::String,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Number,
            LoxParseRule {
                prefix: Some(|compiler, source, _, chunk| compiler.handle_number(source, chunk)),
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::And,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Class,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Else,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::False,
            LoxParseRule {
                prefix: Some(|compiler, source, _, chunk| compiler.handle_literal(source, chunk)),
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::For,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Fun,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::If,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Nil,
            LoxParseRule {
                prefix: Some(|compiler, source, _, chunk| compiler.handle_literal(source, chunk)),
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Or,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Print,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Return,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Super,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::This,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::True,
            LoxParseRule {
                prefix: Some(|compiler, source, _, chunk| compiler.handle_literal(source, chunk)),
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Var,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::While,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::Error,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );
        parsing_rules.insert(
            LoxBytecodeTokenType::EndOfFile,
            LoxParseRule {
                prefix: None,
                infix: None,
                precedence: LoxBytecodeOperatorPrecedence::None,
            },
        );

        let first_token = lexer.scan_token(source)?;
        Ok(Self {
            parser: LoxBytecodeTokensParser {
                current: first_token.clone(), // TODO: check init
                previous: first_token,        // TODO: check init
                had_error: false,
                panic_mode: false,
            },
            parsing_rules,
        })
    }

    pub fn compile(
        &mut self,
        source: &str,
        chunk: &mut LoxBytecodeChunk,
        lexer: &mut LoxBytecodeLexer,
    ) -> BResult<bool> {
        self.init(source, lexer, chunk)?;
        self.parser.had_error = false;
        let mut line_number = usize::MAX;
        loop {
            let token = lexer.scan_token(source)?;
            let token_line_number = token.get_line_number();
            if token_line_number != line_number {
                print!("{:04}", token_line_number);
                line_number = token_line_number;
            } else {
                print!("   | ");
            }
            println!("{:?} '{}'", token.get_kind(), token.get_lexeme(source));
            if token.get_kind() == &LoxBytecodeTokenType::EndOfFile {
                break;
            }
        }
        self.end_compilation(chunk);
        Ok(!self.parser.had_error)
    }

    fn init(
        &mut self,
        source: &str,
        lexer: &mut LoxBytecodeLexer,
        chunk: &mut LoxBytecodeChunk,
    ) -> BResult<()> {
        self.advance(source, lexer)?;
        self.handle_expression(source, lexer, chunk)?;
        self.consume_kind(
            &LoxBytecodeTokenType::EndOfFile,
            source,
            lexer,
            "Expect end of expression.",
        )?;
        Ok(())
    }

    fn end_compilation(&self, chunk: &mut LoxBytecodeChunk) {
        self.emit_return(chunk);
        #[cfg(feature = "code-printing")]
        {
            if !self.parser.had_error {
                disassemble_chunk(chunk, "code");
            }
        }
    }

    fn emit_constant(
        &mut self,
        source: &str,
        chunk: &mut LoxBytecodeChunk,
        value: LoxBytecodeValue,
    ) {
        let constant_value = self.build_constant(source, chunk, value);
        self.emit_bytes(chunk, LoxBytecodeOpcode::Constant, constant_value);
    }

    fn build_constant(
        &mut self,
        source: &str,
        chunk: &mut LoxBytecodeChunk,
        value: LoxBytecodeValue,
    ) -> LoxBytecodeOpcode {
        let constant = chunk.add_constant(value);
        if constant > u8::MAX as usize {
            self.error("Too many constants in one chunk", source);
            LoxBytecodeOpcode::Value(0)
        } else {
            LoxBytecodeOpcode::Value(constant)
        }
    }

    fn emit_return(&self, chunk: &mut LoxBytecodeChunk) {
        self.emit_byte(chunk, LoxBytecodeOpcode::Return);
    }

    fn emit_bytes(
        &self,
        chunk: &mut LoxBytecodeChunk,
        first_byte: LoxBytecodeOpcode,
        second_byte: LoxBytecodeOpcode,
    ) {
        self.emit_byte(chunk, first_byte);
        self.emit_byte(chunk, second_byte);
    }

    fn emit_byte(&self, chunk: &mut LoxBytecodeChunk, opcode: LoxBytecodeOpcode) {
        chunk.append(opcode, self.parser.previous.get_line_number());
    }

    fn handle_string(&mut self, source: &str, chunk: &mut LoxBytecodeChunk) -> BResult<()> {
        let start = self.parser.previous.get_start() + 1; // avoid the leading quotation mark
        let slice = &source[start..start + self.parser.previous.get_length() - 2]; // TODO: check slicing
        self.emit_constant(
            source,
            chunk,
            LoxBytecodeValue::Object(LoxBytecodeObject::String(slice.into())),
        );
        Ok(())
    }

    fn handle_binary(
        &mut self,
        source: &str,
        lexer: &mut LoxBytecodeLexer,
        chunk: &mut LoxBytecodeChunk,
    ) -> BResult<()> {
        let operator_kind = self.parser.previous.get_kind().clone();
        let rule = self.get_rule(&operator_kind)?;
        let precedence =
            LoxBytecodeOperatorPrecedence::from_usize(rule.precedence.clone() as usize + 1)
                .expect("compiler expects a valid value for LoxBytecodeOperatorPrecedence");
        self.parse_precedence(source, precedence, lexer, chunk)?;
        match operator_kind {
            LoxBytecodeTokenType::BangEqual => {
                self.emit_bytes(chunk, LoxBytecodeOpcode::Equal, LoxBytecodeOpcode::Not)
            }
            LoxBytecodeTokenType::EqualEqual => self.emit_byte(chunk, LoxBytecodeOpcode::Equal),
            LoxBytecodeTokenType::Greater => self.emit_byte(chunk, LoxBytecodeOpcode::Greater),
            LoxBytecodeTokenType::GreaterEqual => {
                self.emit_bytes(chunk, LoxBytecodeOpcode::Less, LoxBytecodeOpcode::Not)
            }
            LoxBytecodeTokenType::Less => self.emit_byte(chunk, LoxBytecodeOpcode::Less),
            LoxBytecodeTokenType::LessEqual => {
                self.emit_bytes(chunk, LoxBytecodeOpcode::Greater, LoxBytecodeOpcode::Not)
            }
            LoxBytecodeTokenType::Plus => self.emit_byte(chunk, LoxBytecodeOpcode::Add),
            LoxBytecodeTokenType::Minus => self.emit_byte(chunk, LoxBytecodeOpcode::Subtract),
            LoxBytecodeTokenType::Star => self.emit_byte(chunk, LoxBytecodeOpcode::Multiply),
            LoxBytecodeTokenType::Slash => self.emit_byte(chunk, LoxBytecodeOpcode::Divide),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn handle_unary(
        &mut self,
        source: &str,
        lexer: &mut LoxBytecodeLexer,
        chunk: &mut LoxBytecodeChunk,
    ) -> BResult<()> {
        let operator_kind = self.parser.previous.get_kind().clone();
        // compile the operand
        self.parse_precedence(source, LoxBytecodeOperatorPrecedence::Unary, lexer, chunk)?;
        // emit the operator instruction
        match operator_kind {
            LoxBytecodeTokenType::Bang => self.emit_byte(chunk, LoxBytecodeOpcode::Not),
            LoxBytecodeTokenType::Minus => self.emit_byte(chunk, LoxBytecodeOpcode::Negate),
            _ => unreachable!(),
        };
        Ok(())
    }

    fn handle_grouping(
        &mut self,
        source: &str,
        lexer: &mut LoxBytecodeLexer,
        chunk: &mut LoxBytecodeChunk,
    ) -> BResult<()> {
        self.handle_expression(source, lexer, chunk)?;
        self.consume_kind(
            &LoxBytecodeTokenType::RightParenthesis,
            source,
            lexer,
            "Expect ')' after expression.",
        )?;
        Ok(())
    }

    fn handle_expression(
        &mut self,
        source: &str,
        lexer: &mut LoxBytecodeLexer,
        chunk: &mut LoxBytecodeChunk,
    ) -> BResult<()> {
        self.parse_precedence(
            source,
            LoxBytecodeOperatorPrecedence::Assignment,
            lexer,
            chunk,
        )?;
        Ok(())
    }

    fn handle_number(&mut self, source: &str, chunk: &mut LoxBytecodeChunk) -> BResult<()> {
        let lexeme = self.parser.previous.get_lexeme(source);
        let value: f64 = lexeme
            .parse()
            .map_err(|_| LoxBytecodeInterpreterError::ParserInvalidNumber(lexeme.into()))?;
        self.emit_constant(source, chunk, LoxBytecodeValue::Number(value));
        Ok(())
    }

    fn handle_literal(&mut self, source: &str, chunk: &mut LoxBytecodeChunk) -> BResult<()> {
        match self.parser.previous.get_kind() {
            LoxBytecodeTokenType::False => self.emit_byte(chunk, LoxBytecodeOpcode::False),
            LoxBytecodeTokenType::Nil => self.emit_byte(chunk, LoxBytecodeOpcode::Nil),
            LoxBytecodeTokenType::True => self.emit_byte(chunk, LoxBytecodeOpcode::True),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn parse_precedence(
        &mut self,
        source: &str,
        precedence: LoxBytecodeOperatorPrecedence,
        lexer: &mut LoxBytecodeLexer,
        chunk: &mut LoxBytecodeChunk,
    ) -> BResult<()> {
        self.advance(source, lexer)?;
        if let Some(prefix_rule) = self.get_rule(self.parser.previous.get_kind())?.prefix {
            prefix_rule(self, source, lexer, chunk)?;
        } else {
            self.error("Expect expression.", source);
            return Ok(());
        }

        while precedence.clone() as usize
            <= self
                .get_rule(self.parser.current.get_kind())?
                .precedence
                .clone() as usize
        {
            self.advance(source, lexer)?;
            if let Some(infix_rule) = self.get_rule(self.parser.previous.get_kind())?.infix {
                infix_rule(self, source, lexer, chunk)?;
            } else {
                panic!("Compiler: infix rule expected");
            }
        }

        Ok(())
    }

    fn advance(&mut self, source: &str, lexer: &mut LoxBytecodeLexer) -> BResult<()> {
        self.parser.previous = self.parser.current.clone();
        loop {
            self.parser.current = lexer.scan_token(source)?;
            if self.parser.current.get_kind() != &LoxBytecodeTokenType::Error {
                break;
            }
            self.error_at_current(self.parser.current.get_lexeme(source), source);
        }
        Ok(())
    }

    fn consume_kind(
        &mut self,
        kind: &LoxBytecodeTokenType,
        source: &str,
        lexer: &mut LoxBytecodeLexer,
        message: &str,
    ) -> BResult<()> {
        if self.parser.current.get_kind() == kind {
            self.advance(source, lexer)?;
        } else {
            self.error_at_current(message, source);
        }
        Ok(())
    }

    fn get_rule(&self, kind: &LoxBytecodeTokenType) -> BResult<&LoxParseRule> {
        self.parsing_rules
            .get(kind)
            .ok_or_else(|| LoxBytecodeInterpreterError::CompilerUnknownRule(format!("{:?}", kind)))
    }

    fn error(&mut self, message: &str, source: &str) {
        let token = self.parser.previous.clone();
        self.error_at(&token, source, message);
    }

    fn error_at_current(&mut self, message: &str, source: &str) {
        let token = self.parser.current.clone();
        self.error_at(&token, source, message);
    }

    fn error_at(&mut self, token: &LoxBytecodeToken, source: &str, message: &str) {
        if self.parser.panic_mode {
            return; // suppress any other errors in panic mode
        }

        self.parser.panic_mode = true;
        let mut error = format!("[line {}] Error", token.get_line_number());
        match token.get_kind() {
            LoxBytecodeTokenType::EndOfFile => error += " at end",
            LoxBytecodeTokenType::Error => (),
            _ => error += format!(" at '{}'", token.get_lexeme(source)).as_str(), // TODO: check formatting
        }
        error += format!(": {}\n", message).as_str();
        println!("{}", error);
        self.parser.had_error = true;
    }
}
