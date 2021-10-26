use std::collections::HashMap;

use crate::{
    errors::{LoxInterpreterError, Result},
    values::LoxValue,
};

/// A Lox environment stores variables within a certain scope.
#[derive(Clone)]
pub struct LoxEnvironment {
    values: HashMap<String, LoxValue>,
    /// The enclosing environment, if any.
    outer: Option<Box<LoxEnvironment>>,
}

impl LoxEnvironment {
    pub fn new(outer: Option<Box<LoxEnvironment>>) -> Self {
        Self {
            values: HashMap::new(),
            outer,
        }
    }

    /// Define a variable.
    pub fn define(&mut self, name: String, value: LoxValue) {
        self.values.insert(name, value);
    }

    /// Assign to an existing variable.
    pub fn assign(&mut self, name: &str, value: LoxValue) -> Result<()> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(outer) = &mut self.outer {
            outer.assign(name, value)
        } else {
            Err(LoxInterpreterError::InterpreterUndefinedVariable(
                name.to_string(),
            ))
        }
    }

    /// Retrieve a variable.
    pub fn get(&self, name: &str) -> Result<&LoxValue> {
        let local_value = self.values.get(name);
        if let Some(value) = local_value {
            Ok(value)
        } else if let Some(outer) = &self.outer {
            outer.get(name)
        } else {
            Err(LoxInterpreterError::InterpreterUndefinedVariable(
                name.to_string(),
            ))
        }
    }
}
