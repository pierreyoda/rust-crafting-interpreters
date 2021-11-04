use std::collections::HashMap;

use crate::{
    errors::{LoxInterpreterError, Result},
    interpreter::{
        environment::{environment_handle_get_at_depth, LoxEnvironment, LoxEnvironmentHandle},
        tree_walk::{LoxTreeWalkEvaluator, LoxTreeWalkEvaluatorLocals},
    },
    lexer::LoxToken,
    values::{LoxValue, LoxValueHandle},
};

pub trait LoxCallable {
    fn arity(&self) -> Option<usize>;

    fn call(
        &self,
        env: &mut LoxEnvironmentHandle,
        locals: &LoxTreeWalkEvaluatorLocals,
        arguments: &[LoxValueHandle],
        parenthesis: &LoxToken,
    ) -> Result<LoxValueHandle>;
}

impl LoxCallable for LoxValueHandle {
    fn arity(&self) -> Option<usize> {
        match &*self.borrow() {
            LoxValue::Function {
                arity,
                is_initializer: _,
                declaration: _,
                closure: _,
            } => Some(*arity),
            LoxValue::NativeFunction {
                label: _,
                arity,
                execute: _,
            } => Some(*arity),
            LoxValue::Class {
                name: _,
                methods: _,
            } => {
                if let Some(initializer) = self.borrow().class_find_method("init") {
                    initializer.arity()
                } else {
                    Some(0)
                }
            }
            _ => None,
        }
    }

    fn call(
        &self,
        env: &mut LoxEnvironmentHandle,
        locals: &LoxTreeWalkEvaluatorLocals,
        arguments: &[LoxValueHandle],
        parenthesis: &LoxToken,
    ) -> Result<LoxValueHandle> {
        match &*self.borrow() {
            // TODO: adapt to other evaluators implementations (bytecode)
            LoxValue::Function {
                arity,
                is_initializer,
                declaration,
                closure,
            } => {
                if *arity != arguments.len() {
                    Err(LoxInterpreterError::InterpreterCallableWrongArity(
                        *arity,
                        arguments.len(),
                    ))
                } else {
                    let mut function_env = LoxEnvironment::new(Some(closure.clone()));
                    let (_, parameters, body) =
                        declaration.deconstruct_function_declaration().unwrap();
                    for (i, parameter) in parameters.iter().enumerate() {
                        function_env
                            .borrow_mut()
                            .define(parameter.get_lexeme().clone(), arguments[i].clone());
                    }
                    // TODO: abstract over interpreter evaluator (bytecode)
                    match LoxTreeWalkEvaluator::execute_block_statement(
                        body,
                        &mut function_env,
                        locals,
                    ) {
                        Ok(_) => environment_handle_get_at_depth(closure, "this", 0),
                        Err(why) => match why {
                            LoxInterpreterError::InterpreterReturn(value) => {
                                if self.borrow().function_is_initializer() {
                                    environment_handle_get_at_depth(closure, "this", 0)
                                } else {
                                    Ok(value)
                                }
                            }
                            _ => Err(why),
                        },
                    }
                }
            }
            LoxValue::NativeFunction {
                label: _,
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
            LoxValue::Class {
                name: _,
                methods: _,
            } => {
                // class constructor (empty by default)
                let instance = LoxValue::new(LoxValue::ClassInstance {
                    class: self.clone(),
                    fields: HashMap::new(),
                });
                // initializer (optional)
                if let Some(initializer) = self.borrow().class_find_method("init") {
                    initializer
                        .borrow()
                        .class_method_bind_this(self)
                        .unwrap()
                        .call(env, locals, arguments, parenthesis)?;
                }
                Ok(instance)
            }
            _ => Err(LoxInterpreterError::InterpreterNonCallableValue(
                parenthesis.clone(),
            )),
        }
    }
}
