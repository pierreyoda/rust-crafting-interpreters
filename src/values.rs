use std::{borrow::Borrow, collections::HashMap, fmt, hash::Hash};

use crate::{
    errors::{LoxInterpreterError, Result},
    expressions::LoxStatement,
    interpreter::environment::{LoxEnvironment, LoxEnvironmentHandle},
    lexer::LoxToken,
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
        is_initializer: bool,
        declaration: Box<LoxStatement>,
        closure: LoxEnvironmentHandle,
    },
    NativeFunction {
        label: String,
        /// Number of input parameters.
        arity: usize,
        execute: LoxNativeFunctionExecutor,
    },
    Class {
        name: String,
        methods: HashMap<String, LoxValue>,
    },
    ClassInstance {
        class: Box<LoxValue>,
        fields: HashMap<String, LoxValue>,
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

    pub fn equals(&self, other: &Self) -> bool {
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

    pub fn function_is_initializer(&self) -> bool {
        if let Self::Function {
            arity: _,
            closure: _,
            declaration: _,
            is_initializer,
        } = self
        {
            *is_initializer
        } else {
            false
        }
    }

    pub fn class_name(&self) -> Option<&String> {
        match self {
            Self::Class { name, methods: _ } => Some(name),
            _ => None,
        }
    }

    pub fn class_find_method(&self, name: &str) -> Option<&LoxValue> {
        match self {
            Self::Class { name: _, methods } => methods.get(name),
            _ => None,
        }
    }

    pub fn class_method_bind_this(&self, instance: &Self) -> Option<Self> {
        match self {
            Self::Function {
                arity,
                is_initializer,
                closure,
                declaration,
            } => {
                let environment = LoxEnvironment::new(Some(closure.clone()));
                environment
                    .borrow_mut()
                    .define("this".into(), instance.clone());
                Some(LoxValue::Function {
                    arity: *arity,
                    closure: environment,
                    is_initializer: *is_initializer,
                    declaration: declaration.clone(),
                })
            }
            _ => None,
        }
    }

    pub fn instance_get_field(&self, name: &LoxToken) -> Result<Self> {
        if let LoxValue::ClassInstance { class, fields } = self {
            // find method
            if let Some(method) = class.class_find_method(name.get_lexeme()) {
                return Ok(method
                    .class_method_bind_this(self)
                    .expect("method value is a function"));
            }
            // find field
            Ok(fields
                .get(name.get_lexeme())
                .map(|v| {
                    let borrowed: &LoxValue = v.borrow();
                    borrowed.clone()
                })
                .ok_or_else(|| {
                    LoxInterpreterError::InterpreterUndefinedClassProperty(
                        name.get_lexeme().clone(),
                    )
                })?)
        } else {
            Err(LoxInterpreterError::InterpreterCannotGetOrSetField(
                name.clone(),
            ))
        }
    }

    pub fn instance_set_field(&mut self, name: &LoxToken, value: LoxValue) -> Result<Self> {
        if let LoxValue::ClassInstance { class: _, fields } = self {
            fields.insert(name.get_lexeme().clone(), value.clone());
            Ok(value)
        } else {
            Err(LoxInterpreterError::InterpreterCannotGetOrSetField(
                name.clone(),
            ))
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
                is_initializer: _,
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
            Self::Class { name, methods: _ } => name.clone(),
            Self::ClassInstance { class, fields: _ } => {
                format!("{} instance", class.class_name().unwrap())
            }
        }
    }
}

impl fmt::Debug for LoxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.representation().as_str())
    }
}
