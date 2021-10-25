use crate::expressions::{LoxExpression, LoxLiteral};

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
            Self::Assign { name, value } => debug_parenthesize("=", &[value.as_ref()]),
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
