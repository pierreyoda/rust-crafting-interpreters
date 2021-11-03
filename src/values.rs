use std::fmt;

use crate::{
    errors::Result, expressions::LoxStatement, interpreter::environment::LoxEnvironmentHandle,
    printer::LoxPrintable,
};

pub const LOX_NUMBER_VALUE_COMPARISON_EPSILON: f64 = f64::EPSILON;

pub type LoxNativeFunctionExecutor = fn(&mut LoxEnvironmentHandle, &[LoxValue]) -> Result<LoxValue>;

/// A runtime Lox value.
#[derive(Clone)]
pub enum LoxValue {
    Nil,
    Number(f64),
    Boolean(bool),
    String(String),
    Function {
        /// Number of input parameters.
        arity: usize,
        declaration: Box<LoxStatement>,
        closure: LoxEnvironmentHandle,
    },
    NativeFunction {
        label: String,
        /// Number of input parameters.
        arity: usize,
        execute: LoxNativeFunctionExecutor,
    },
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
            Self::Function {
                arity: _,
                declaration,
                closure: _,
            } => {
                let (name, _, _) = declaration.deconstruct_function_declaration().unwrap();
                format!("<fn {}>", name.get_lexeme())
            }
            Self::NativeFunction {
                label,
                arity: _,
                execute: _,
            } => format!("<native fn {}>", label),
        }
    }
}

impl fmt::Debug for LoxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.representation().as_str())
    }
}
