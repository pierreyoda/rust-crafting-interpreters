use std::collections::HashMap;

use crate::{
    errors::{LoxInterpreterError, Result},
    values::LoxValue,
};

/// A Lox environment stores variables within a certain scope.
pub struct LoxEnvironment {
    values: HashMap<String, LoxValue>,
}

impl LoxEnvironment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
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
        } else {
            Err(LoxInterpreterError::InterpreterUndefinedVariable(
                name.to_string(),
            ))
        }
    }

    /// Retrieve a variable.
    pub fn get(&self, name: &str) -> Result<&LoxValue> {
        self.values
            .get(name)
            .ok_or_else(|| LoxInterpreterError::InterpreterUndefinedVariable(name.to_string()))
    }
}
