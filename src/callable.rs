use std::collections::HashMap;

use crate::{
    errors::{LoxInterpreterError, Result},
    interpreter::{
        environment::{environment_handle_get_at_depth, LoxEnvironment, LoxEnvironmentHandle},
        tree_walk::{LoxTreeWalkEvaluator, LoxTreeWalkEvaluatorLocals},
    },
    lexer::LoxToken,
    printer::LoxPrintable,
    values::LoxValue,
};

pub trait LoxCallable: LoxPrintable {
    fn arity(&self) -> Option<usize>;

    fn call(
        &self,
        env: &mut LoxEnvironmentHandle,
        locals: &LoxTreeWalkEvaluatorLocals,
        arguments: &[LoxValue],
        parenthesis: &LoxToken,
    ) -> Result<LoxValue>;
}

impl LoxCallable for LoxValue {
    fn arity(&self) -> Option<usize> {
        match self {
            Self::Function {
                arity,
                is_initializer: _,
                declaration: _,
                closure: _,
            } => Some(*arity),
            Self::NativeFunction {
                label: _,
                arity,
                execute: _,
            } => Some(*arity),
            Self::Class {
                name: _,
                methods: _,
            } => {
                if let Some(initializer) = self.class_find_method("init") {
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
        arguments: &[LoxValue],
        parenthesis: &LoxToken,
    ) -> Result<LoxValue> {
        match self {
            // TODO: adapt to other evaluators implementations (bytecode)
            Self::Function {
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
                        // TODO: abstract over interpreter evaluator (bytecode)
                    }
                    match LoxTreeWalkEvaluator::execute_block_statement(
                        body,
                        &mut function_env,
                        locals,
                    ) {
                        Ok(_) => environment_handle_get_at_depth(closure, "this", 0),
                        Err(why) => match why {
                            LoxInterpreterError::InterpreterReturn(value) => {
                                if self.function_is_initializer() {
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
            Self::NativeFunction {
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
            Self::Class {
                name: _,
                methods: _,
            } => {
                // class constructor (empty by default)
                let instance = LoxValue::ClassInstance {
                    class: Box::new(self.clone()),
                    fields: HashMap::new(),
                };
                // initializer (optional)
                if let Some(initializer) = self.class_find_method("init") {
                    initializer.class_method_bind_this(self).unwrap().call(
                        env,
                        locals,
                        arguments,
                        parenthesis,
                    )?;
                }
                Ok(instance)
            }
            _ => Err(LoxInterpreterError::InterpreterNonCallableValue(
                parenthesis.clone(),
            )),
        }
    }
}
