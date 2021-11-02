use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    errors::{LoxInterpreterError, Result},
    values::LoxValue,
};

pub type LoxEnvironmentHandle = Rc<RefCell<LoxEnvironment>>;

/// A Lox environment stores variables within a certain scope.
#[derive(Clone)]
pub struct LoxEnvironment {
    values: HashMap<String, LoxValue>,
    /// The enclosing environment, if any.
    outer: Option<LoxEnvironmentHandle>,
}

impl LoxEnvironment {
    pub fn new(outer: Option<LoxEnvironmentHandle>) -> LoxEnvironmentHandle {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            outer,
        }))
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
            outer.borrow_mut().assign(name, value)
        } else {
            Err(LoxInterpreterError::InterpreterUndefinedVariable(
                name.to_string(),
            ))
        }
    }

    /// Retrieve a variable.
    pub fn get(&self, name: &str) -> Result<LoxValue> {
        let local_value = self.values.get(name);
        if let Some(value) = local_value {
            Ok(value.clone())
        } else if let Some(outer) = &self.outer {
            Self::get_deeply(name, outer)
        } else {
            Err(LoxInterpreterError::InterpreterUndefinedVariable(
                name.to_string(),
            ))
        }
    }

    fn get_deeply(name: &str, env: &LoxEnvironmentHandle) -> Result<LoxValue> {
        let mut current = env.clone();
        loop {
            if let Some(value) = current.borrow().values.get(name).cloned() {
                return Ok(value);
            }
            let new = if let Some(outer) = &current.borrow().outer {
                outer.clone()
            } else {
                break;
            };
            current = new;
        }
        Err(LoxInterpreterError::InterpreterUndefinedVariable(
            name.to_string(),
        ))
    }
}
