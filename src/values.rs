use std::fmt;

use crate::{
    callable::LoxCallable,
    errors::{LoxInterpreterError, Result},
    expressions::LoxStatement,
    interpreter::{environment::LoxEnvironment, tree_walk::LoxTreeWalkEvaluator},
    lexer::LoxToken,
    printer::LoxPrintable,
};

pub const LOX_NUMBER_VALUE_COMPARISON_EPSILON: f64 = f64::EPSILON;

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
    },
    NativeFunction {
        label: String,
        /// Number of input parameters.
        arity: usize,
        execute: fn(&mut LoxEnvironment, &[LoxValue]) -> Result<LoxValue>,
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

impl LoxCallable for LoxValue {
    fn arity(&self) -> Option<usize> {
        match self {
            Self::Function {
                arity,
                declaration: _,
            } => Some(*arity),
            Self::NativeFunction {
                label: _,
                arity,
                execute: _,
            } => Some(*arity),
            _ => None,
        }
    }

    fn call(
        &self,
        env: &mut LoxEnvironment,
        arguments: &[LoxValue],
        parenthesis: &LoxToken,
    ) -> Result<LoxValue> {
        match self {
            // TODO: adapt to other evaluators implementations (bytecode)
            Self::Function { arity, declaration } => {
                if *arity != arguments.len() {
                    Err(LoxInterpreterError::InterpreterCallableWrongArity(
                        *arity,
                        arguments.len(),
                    ))
                } else {
                    let mut function_env = LoxEnvironment::new(Some(Box::new(env.clone())));
                    let (_, parameters, body) =
                        declaration.deconstruct_function_declaration().unwrap();
                    for (i, parameter) in parameters.iter().enumerate() {
                        function_env.define(parameter.get_lexeme().clone(), arguments[i].clone());
                        // TODO: abstract over interpreter evaluator (bytecode)
                    }
                    match LoxTreeWalkEvaluator::execute_block_statement(body, &mut function_env) {
                        Ok(_) => Ok(LoxValue::Nil),
                        Err(why) => match why {
                            LoxInterpreterError::InterpreterReturn(value) => Ok(value),
                            _ => Err(why),
                        },
                    }
                }
            }
            Self::NativeFunction {
                label: _label,
                arity,
                execute,
            } => {
                if *arity != arguments.len() {
                    Err(LoxInterpreterError::InterpreterCallableWrongArity(
                        *arity,
                        arguments.len(),
                    ))
                } else {
                    execute(env, arguments)
                }
            }
            _ => Err(LoxInterpreterError::InterpreterNonCallableValue(
                parenthesis.clone(),
            )),
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
