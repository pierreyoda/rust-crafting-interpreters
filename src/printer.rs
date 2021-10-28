use crate::{
    expressions::{LoxExpression, LoxLiteral, LoxOperation, LoxStatement},
    lexer::LoxToken,
};

pub trait LoxPrintable {
    fn representation(&self) -> String;
}

impl LoxPrintable for LoxLiteral {
    fn representation(&self) -> String {
        match self {
            Self::Number(number) => format!("{}", number),
            Self::String(string) => string.clone(),
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::Nil => "nil".to_string(),
        }
    }
}

impl LoxPrintable for LoxExpression {
    fn representation(&self) -> String {
        match self {
            Self::NoOp => "".to_string(),
            Self::Assign { name, value } => debug_parenthesize_fragments(&[
                LoxPrintableFragment::Arbitrary("=".into()),
                LoxPrintableFragment::Token(name),
                LoxPrintableFragment::Expression(value),
            ]),
            Self::Binary {
                left,
                operator,
                right,
            } => debug_parenthesize(
                operator.get_lexeme().as_str(),
                &[left.as_ref(), right.as_ref()],
            ),
            Self::Call {
                callee,
                parenthesis,
                arguments,
            } => todo!(),
            Self::Get { object, name } => todo!(),
            Self::Group { expression } => debug_parenthesize("group", &[expression.as_ref()]),
            Self::Literal { value } => value.representation(),
            Self::Logical {
                left,
                operator,
                right,
            } => todo!(),
            Self::Set {
                object,
                name,
                value,
            } => todo!(),
            Self::Super { keyword, method } => todo!(),
            Self::This { keyword } => "this".to_string(),
            Self::Unary { operator, right } => {
                debug_parenthesize(operator.get_lexeme().as_str(), &[right.as_ref()])
            }
            Self::Variable { name } => name.get_lexeme().clone(),
        }
    }
}

impl LoxPrintable for LoxStatement {
    fn representation(&self) -> String {
        match self {
            Self::NoOp => "".to_string(),
            Self::Block { statements } => {
                let mut output = "(block ".to_string();
                for statement in statements {
                    output += statement.representation().as_str();
                }
                output += ")";
                output
            }
            Self::Class {
                name,
                super_class,
                methods,
            } => todo!(),
            Self::Expression { expression } => debug_parenthesize_fragments(&[
                LoxPrintableFragment::Arbitrary(";".into()),
                LoxPrintableFragment::Expression(expression),
            ]),
            Self::Function {
                name,
                parameters,
                body,
            } => todo!(),
            Self::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if else_branch.is_noop() {
                    debug_parenthesize_fragments(&[
                        LoxPrintableFragment::Arbitrary("if".into()),
                        LoxPrintableFragment::Expression(condition),
                        LoxPrintableFragment::Statement(then_branch),
                    ])
                } else {
                    debug_parenthesize_fragments(&[
                        LoxPrintableFragment::Arbitrary("if-else".into()),
                        LoxPrintableFragment::Expression(condition),
                        LoxPrintableFragment::Statement(then_branch),
                        LoxPrintableFragment::Statement(else_branch),
                    ])
                }
            }
            Self::Print { expression } => debug_parenthesize_fragments(&[
                LoxPrintableFragment::Arbitrary("print".into()),
                LoxPrintableFragment::Expression(expression),
            ]),
            Self::Return { value, keyword: _ } => {
                if value.is_noop() {
                    "(return)".to_string()
                } else {
                    debug_parenthesize_fragments(&[
                        LoxPrintableFragment::Arbitrary("return".into()),
                        LoxPrintableFragment::Expression(value),
                    ])
                }
            }
            Self::Variable { name, initializer } => {
                if initializer.is_noop() {
                    debug_parenthesize_fragments(&[
                        LoxPrintableFragment::Arbitrary("var".into()),
                        LoxPrintableFragment::Token(name),
                    ])
                } else {
                    debug_parenthesize_fragments(&[
                        LoxPrintableFragment::Arbitrary("var".into()),
                        LoxPrintableFragment::Token(name),
                        LoxPrintableFragment::Arbitrary("=".into()),
                        LoxPrintableFragment::Expression(initializer),
                    ])
                }
            }
            Self::While { condition, body } => debug_parenthesize_fragments(&[
                LoxPrintableFragment::Arbitrary("while".into()),
                LoxPrintableFragment::Expression(condition),
                LoxPrintableFragment::Statement(body),
            ]),
        }
    }
}

impl LoxPrintable for LoxOperation {
    fn representation(&self) -> String {
        match self {
            Self::Invalid => "".to_string(),
            Self::Statement(statement) => statement.representation(),
            Self::Expression(expression) => expression.representation(),
        }
    }
}

pub fn operations_representation(operations: &[LoxOperation]) -> String {
    let mut output = String::new();
    for (i, operation) in operations.iter().enumerate() {
        output += operation.representation().as_str();
        if i != operations.len() - 1 {
            output += "\n";
        }
    }
    output
}

fn debug_parenthesize(name: &str, expressions: &[&LoxExpression]) -> String {
    let mut output = String::new();
    output += "(";
    output += name;
    for expression in expressions {
        output += " ";
        output += expression.representation().as_str();
    }
    output += ")";
    output
}

enum LoxPrintableFragment<'a> {
    Arbitrary(String),
    Token(&'a LoxToken),
    Statement(&'a LoxStatement),
    Expression(&'a LoxExpression),
}

impl<'a> LoxPrintable for LoxPrintableFragment<'a> {
    fn representation(&self) -> String {
        match self {
            Self::Arbitrary(string) => string.clone(),
            Self::Token(token) => token.get_lexeme().clone(),
            Self::Statement(statement) => statement.representation(),
            Self::Expression(expression) => expression.representation(),
        }
    }
}

fn debug_parenthesize_fragments(fragments: &[LoxPrintableFragment]) -> String {
    let mut output = String::new();
    output += "(";
    for (i, fragment) in fragments.iter().enumerate() {
        if i != 0 {
            output += " ";
        }
        output += fragment.representation().as_str();
    }
    output += ")";
    output
}
