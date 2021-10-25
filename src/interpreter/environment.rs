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

    /// Retrieve a variable.
    pub fn get(&self, name: &str) -> Result<&LoxValue> {
        self.values
            .get(name)
            .ok_or_else(|| LoxInterpreterError::InterpreterUndefinedVariable(name.to_string()))
    }
}
