use crate::{
    errors::{LoxInterpreterError, Result},
    lexer::LoxToken,
};

#[derive(Clone)]
pub enum LoxOperation {
    Invalid,
    Expression(LoxExpression),
    Statement(LoxStatement),
}

impl LoxOperation {
    pub fn as_expression(self) -> Result<LoxExpression> {
        match self {
            Self::Expression(expression) => Ok(expression),
            _ => Err(LoxInterpreterError::ParserUnexpectedOperation(
                "operation is not an expression".into(),
            )),
        }
    }

    pub fn as_statement(self) -> Result<LoxStatement> {
        match self {
            Self::Statement(statement) => Ok(statement),
            _ => Err(LoxInterpreterError::ParserUnexpectedOperation(
                "operation is not a statement".into(),
            )),
        }
    }
}

#[derive(Clone)]
pub enum LoxExpression {
    NoOp,
    /// Variable assignment.
    Assign {
        name: LoxToken,
        value: Box<LoxExpression>,
    },
    /// Binary operation.
    Binary {
        left: Box<LoxExpression>,
        operator: LoxToken,
        right: Box<LoxExpression>,
    },
    /// Function call.
    Call {
        callee: Box<LoxExpression>,
        parenthesis: LoxToken,
        arguments: Vec<LoxExpression>,
    },
    /// Property access.
    Get {
        object: Box<LoxExpression>,
        name: LoxToken,
    },
    /// Using parentheses to group expressions.
    Group {
        expression: Box<LoxExpression>,
    },
    /// Literal value.
    Literal {
        value: LoxLiteral,
    },
    /// Logical (and/or) branching.
    Logical {
        left: Box<LoxExpression>,
        operator: LoxToken,
        right: Box<LoxExpression>,
    },
    /// Property assignment.
    Set {
        object: Box<LoxExpression>,
        name: LoxToken,
        value: Box<LoxExpression>,
    },
    /// Super expression.
    Super {
        keyword: LoxToken,
        method: LoxToken,
    },
    /// This expression.
    This {
        keyword: LoxToken,
    },
    /// Unary operation.
    Unary {
        operator: LoxToken,
        right: Box<LoxExpression>,
    },
    /// Variable access.
    Variable {
        name: LoxToken,
    },
}

impl LoxExpression {
    pub fn is_noop(&self) -> bool {
        matches!(self, Self::NoOp)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoxLiteral {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
}

#[derive(Clone)]
pub enum LoxStatement {
    NoOp,
    /// Curly-braced block.
    Block {
        statements: Vec<LoxStatement>,
    },
    /// Class declaration.
    Class {
        name: LoxToken,
        super_class: LoxExpression, // LoxExpression::Variable
        methods: Vec<LoxStatement>, // array of LoxStatement::Function
    },
    /// Expression.
    Expression {
        expression: LoxExpression,
    },
    /// Function declaration.
    Function {
        name: LoxToken,
        parameters: Vec<LoxToken>,
        body: Vec<LoxStatement>,
    },
    /// If branching.
    If {
        condition: LoxExpression,
        then_branch: Box<LoxStatement>,
        else_branch: Box<LoxStatement>,
    },
    /// Print.
    Print {
        expression: LoxExpression,
    },
    /// Return.
    Return {
        keyword: LoxToken,
        value: LoxExpression,
    },
    /// Variable declaration.
    Variable {
        name: LoxToken,
        initializer: LoxExpression,
    },
    /// While loop.
    While {
        condition: LoxExpression,
        body: Box<LoxStatement>,
    },
}

impl LoxStatement {
    pub fn is_noop(&self) -> bool {
        matches!(self, Self::NoOp)
    }

    pub fn deconstruct_function_declaration(
        &self,
    ) -> Option<(&LoxToken, &[LoxToken], &[LoxStatement])> {
        match self {
            Self::Function {
                name,
                parameters,
                body,
            } => Some((name, parameters.as_ref(), body.as_ref())),
            _ => None,
        }
    }

    pub fn get_type_representation(&self) -> &str {
        match self {
            Self::NoOp => "noop",
            Self::Block { statements: _ } => "block",
            Self::Class {
                name: _,
                super_class: _,
                methods: _,
            } => "class",
            Self::Expression { expression: _ } => "expression",
            Self::Function {
                name: _,
                parameters: _,
                body: _,
            } => "function",
            Self::If {
                condition: _,
                then_branch: _,
                else_branch: _,
            } => "if",
            Self::Print { expression: _ } => "print",
            Self::Return {
                keyword: _,
                value: _,
            } => "return",
            Self::Variable {
                name: _,
                initializer: _,
            } => "variable",
            Self::While {
                condition: _,
                body: _,
            } => "while",
        }
    }
}
