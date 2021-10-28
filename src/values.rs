use crate::printer::LoxPrintable;

pub const LOX_NUMBER_VALUE_COMPARISON_EPSILON: f64 = f64::EPSILON;

/// A runtime Lox value.
#[derive(Clone, Debug)]
pub enum LoxValue {
    Nil,
    Number(f64),
    Boolean(bool),
    String(String),
}

impl LoxValue {
    /// Lox follows Ruby’s simple rule: false and nil are falsy,
    /// and everything else is truthy.
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Boolean(boolean) => *boolean,
            _ => true,
        }
    }

    pub fn equals(&self, other: &LoxValue) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Nil, _) => false,
            (Self::Boolean(left), Self::Boolean(right)) => *left == *right,
            (Self::String(left), Self::String(right)) => *left == *right,
            (Self::Number(left), Self::Number(right)) => {
                (left - right).abs() < LOX_NUMBER_VALUE_COMPARISON_EPSILON
            }
            _ => false,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(number) => Some(*number),
            _ => None,
        }
    }
}

impl LoxPrintable for LoxValue {
    fn representation(&self) -> String {
        match self {
            Self::Nil => "nil".to_string(),
            Self::Number(number) => format!("{}", number),
            Self::Boolean(boolean) => (if *boolean { "true" } else { "false" }).to_string(),
            Self::String(string) => string.clone(),
        }
    }
}
