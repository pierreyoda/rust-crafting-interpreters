use crate::{
    errors::{LoxInterpreterError, Result},
    interpreter::{
        environment::{LoxEnvironment, LoxEnvironmentHandle},
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
                declaration: _,
                closure: _,
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
        env: &mut LoxEnvironmentHandle,
        locals: &LoxTreeWalkEvaluatorLocals,
        arguments: &[LoxValue],
        parenthesis: &LoxToken,
    ) -> Result<LoxValue> {
        match self {
            // TODO: adapt to other evaluators implementations (bytecode)
            Self::Function {
                arity,
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
