use crate::lexer::LoxToken;

#[derive(Clone)]
pub enum LoxExpression {
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
    Group { expression: Box<LoxExpression> },
    /// Literal value.
    Literal { value: LoxLiteral },
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
    Super { keyword: LoxToken, method: LoxToken },
    /// This expression.
    This { keyword: LoxToken },
    /// Unary operation.
    Unary {
        operator: LoxToken,
        right: Box<LoxExpression>,
    },
    /// Variable access.
    Variable { name: LoxToken },
}

#[derive(Clone, Debug)]
pub enum LoxLiteral {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
}

pub enum LoxStatement {
    /// Curly-braced block.
    Block { statements: Vec<LoxStatement> },
    /// Class declaration.
    Class {
        name: LoxToken,
        super_class: LoxExpression, // LoxExpression::Variable
        methods: Vec<LoxStatement>, // array of LoxStatement::Function
    },
    /// Expression.
    Expression { expression: LoxExpression },
    /// Function declaration.
    Function {
        name: LoxToken,
        parameters: Vec<LoxToken>,
        body: Vec<LoxStatement>,
    },
    /// If branching.
    If {
        condition: LoxExpression,
        then_ranch: Box<LoxStatement>,
        else_branch: Box<LoxStatement>,
    },
    /// Print.
    Print { expression: LoxExpression },
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
