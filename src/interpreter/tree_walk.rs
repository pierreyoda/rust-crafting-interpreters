use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use crate::{
    callable::LoxCallable,
    errors::{LoxInterpreterError, Result},
    expressions::{LoxExpression, LoxLiteral, LoxOperation, LoxStatement},
    interpreter::environment::environment_handle_assign_at_depth,
    lexer::{LoxToken, LoxTokenType},
    printer::LoxPrintable,
    values::{
        lox_value_handle_instance_get_field, lox_value_handle_instance_set_field, LoxValue,
        LoxValueHandle,
    },
};

use super::{
    builtins::build_lox_clock_builtin,
    environment::{environment_handle_get_at_depth, LoxEnvironment, LoxEnvironmentHandle},
};

pub type LoxTreeWalkEvaluatorLocals = HashMap<u64, usize>;

pub struct LoxTreeWalkEvaluator {
    globals: LoxEnvironmentHandle,
    locals: LoxTreeWalkEvaluatorLocals,
}

impl LoxTreeWalkEvaluator {
    pub fn new() -> Self {
        let globals = LoxEnvironment::new(None);
        globals
            .borrow_mut()
            .define("clock".into(), build_lox_clock_builtin());
        Self {
            globals,
            locals: HashMap::new(),
        }
    }

    pub fn get_environment(&self) -> &LoxEnvironmentHandle {
        &self.globals
    }

    pub fn evaluate(&mut self, operation: &LoxOperation) -> Result<LoxValueHandle> {
        match operation {
            LoxOperation::Invalid => Ok(LoxValue::new(LoxValue::Nil)),
            LoxOperation::Expression(expression) => {
                Self::evaluate_expression(expression, &mut self.globals, &self.locals)
            }
            LoxOperation::Statement(statement) => {
                Self::evaluate_statement(statement, &mut self.globals, &self.locals)
            }
        }
    }

    pub fn resolve_variable(&mut self, expression: &LoxExpression, depth: usize) {
        let key = Self::compute_locals_key_from_expression(expression);
        self.locals.insert(key, depth);
    }

    pub fn lookup_variable(
        expression: &LoxExpression,
        name: &LoxToken,
        env: &LoxEnvironmentHandle,
        locals: &LoxTreeWalkEvaluatorLocals,
    ) -> Result<LoxValueHandle> {
        if let Some(distance) = locals.get(&Self::compute_locals_key_from_expression(expression)) {
            environment_handle_get_at_depth(env, name.get_lexeme().as_str(), *distance)
        } else {
            env.borrow().get(name.get_lexeme().as_str())
        }
    }

    fn compute_locals_key_from_expression(expression: &LoxExpression) -> u64 {
        let mut hasher = DefaultHasher::new();
        expression.hash(&mut hasher);
        hasher.finish()
    }

    fn evaluate_statement(
        statement: &LoxStatement,
        env: &mut LoxEnvironmentHandle,
        locals: &LoxTreeWalkEvaluatorLocals,
    ) -> Result<LoxValueHandle> {
        match statement {
            LoxStatement::NoOp => Ok(LoxValue::new(LoxValue::Nil)),
            LoxStatement::Expression { expression } => {
                Self::evaluate_expression(expression, env, locals)?;
                Ok(LoxValue::new(LoxValue::Nil))
            }
            LoxStatement::Print { expression } => {
                let value = Self::evaluate_expression(expression, env, locals)?;
                println!("{}", value.borrow().representation());
                Ok(LoxValue::new(LoxValue::Nil))
            }
            LoxStatement::Variable { name, initializer } => {
                let value = Self::evaluate_expression(initializer, env, locals)?;
                env.borrow_mut().define(name.get_lexeme().clone(), value);
                Ok(LoxValue::new(LoxValue::Nil))
            }
            LoxStatement::Block { statements } => {
                let mut block_env = LoxEnvironment::new(Some(env.clone()));
                Self::execute_block_statement(statements, &mut block_env, locals)
            }
            LoxStatement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_value = Self::evaluate_expression(condition, env, locals)?;
                if condition_value.borrow().is_truthy() {
                    Self::evaluate_statement(then_branch, env, locals)?;
                } else if !else_branch.is_noop() {
                    Self::evaluate_statement(else_branch, env, locals)?;
                }
                Ok(LoxValue::new(LoxValue::Nil))
            }
            LoxStatement::While { condition, body } => {
                while Self::evaluate_expression(condition, env, locals)?.borrow().is_truthy() {
                    let _ = Self::evaluate_statement(body, env, locals)?;
                }
                Ok(LoxValue::new(LoxValue::Nil))
            }
            LoxStatement::Function {
                name,
                parameters,
                body: _,
            } => {
                let function = LoxValue::new(LoxValue::Function {
                    is_initializer: false,
                    arity: parameters.len(),
                    declaration: Box::new(statement.clone()),
                    closure: env.clone(),
                });
                env.borrow_mut().define(name.get_lexeme().clone(), function);
                Ok(LoxValue::new(LoxValue::Nil))
            }
            LoxStatement::Return { keyword: _, value } => {
                let returned_value = if value.is_noop() {
                    LoxValue::new(LoxValue::Nil)
                } else {
                    Self::evaluate_expression(value, env, locals)?
                };
                Err(LoxInterpreterError::InterpreterReturn(returned_value))
            }
            LoxStatement::Class {
                name,
                super_class,
                methods,
            } => {
                // super-class handling
                let super_class_value = if super_class.is_noop() {
                    LoxValue::new(LoxValue::Nil)
                } else {
                    let super_class_value = Self::evaluate_expression(super_class, env, locals)?;
                    if super_class_value.borrow().is_class() {
                        super_class_value
                    } else {
                        return Err(LoxInterpreterError::InterpreterSuperClassNotAClass(super_class.representation()));
                    }
                };
                // allows references to the class inside its own methods
                env.borrow_mut()
                    .define(name.get_lexeme().clone(), LoxValue::new(LoxValue::Nil));
                // "super" handling
                let class_env = if super_class.is_noop() {
                    env.clone()
                } else {
                    let class_env = env.clone();
                    class_env.borrow_mut().define("super".into(), super_class_value.clone());
                    class_env
                };
                // methods
                let mut evaluated_methods: HashMap<String, LoxValueHandle> = HashMap::new();
                for method in methods {
                    if let LoxStatement::Function { name: method_name, parameters, body: _ } = method {
                            let borrowed_method: &LoxStatement = method;
                            let declaration = borrowed_method.clone();
                            let function = LoxValue::new(LoxValue::Function {
                                arity: parameters.len(),
                                is_initializer: method_name.get_lexeme() == "init",
                                declaration: Box::new(declaration),
                                closure: class_env.clone(),
                            });
                            evaluated_methods.insert(method_name.get_lexeme().clone(), function);
                        } else {
                            panic!("interpreter: expected a function statement in class methods");
                        }
                }
                // class value
                let class = LoxValue::new(LoxValue::Class { name: name.get_lexeme().clone(), super_class: super_class_value.clone(), methods: evaluated_methods });
                env.borrow_mut()
                    .define(name.get_lexeme().clone(), class);
                Ok(LoxValue::new(LoxValue::Nil))
            }
            // _ => panic!(
            //     "treewalk.evaluate_statement: not implemented for: {}\n{}",
            //     statement.get_type_representation(),
            //     statement.representation()
            // ),
        }
    }

    pub fn execute_block_statement(
        statements: &[LoxStatement],
        env: &mut LoxEnvironmentHandle,
        locals: &LoxTreeWalkEvaluatorLocals,
    ) -> Result<LoxValueHandle> {
        for statement in statements {
            Self::evaluate_statement(statement, env, locals)?;
        }
        Ok(LoxValue::new(LoxValue::Nil))
    }

    fn evaluate_expression(
        expression: &LoxExpression,
        env: &mut LoxEnvironmentHandle,
        locals: &LoxTreeWalkEvaluatorLocals,
    ) -> Result<LoxValueHandle> {
        match expression {
            LoxExpression::NoOp => Ok(LoxValue::new(LoxValue::Nil)),
            LoxExpression::Literal { value } => Ok(Self::evaluate_literal(value)),
            LoxExpression::Group { expression: expr } => {
                Self::evaluate_expression(expr, env, locals)
            }
            LoxExpression::Unary { operator, right } => {
                let right_value = Self::evaluate_expression(right, env, locals)?;
                match operator.get_kind() {
                    // number inversion
                    LoxTokenType::Minus => Ok(LoxValue::new(LoxValue::Number(
                        -Self::extract_number(&right_value)?,
                    ))),
                    // logical not
                    LoxTokenType::Bang => Ok(LoxValue::new(LoxValue::Boolean(
                        !right_value.borrow().is_truthy(),
                    ))),
                    // unexpected
                    _ => Err(LoxInterpreterError::InterpreterUnexpectedOperation(
                        operator.get_lexeme().clone(),
                    )),
                }
            }
            LoxExpression::Binary {
                left,
                operator,
                right,
            } => {
                let (left_value, right_value) = (
                    Self::evaluate_expression(left, env, locals)?,
                    Self::evaluate_expression(right, env, locals)?,
                );
                match operator.get_kind() {
                    // subtraction
                    LoxTokenType::Minus => Ok(LoxValue::new(LoxValue::Number(
                        Self::extract_number(&left_value)? - Self::extract_number(&right_value)?,
                    ))),
                    // division
                    LoxTokenType::Slash => Ok(LoxValue::new(LoxValue::Number(
                        Self::extract_number(&left_value)? / Self::extract_number(&right_value)?,
                    ))),
                    // multiplication
                    LoxTokenType::Star => Ok(LoxValue::new(LoxValue::Number(
                        Self::extract_number(&left_value)? * Self::extract_number(&right_value)?,
                    ))),
                    // addition and string concatenation
                    LoxTokenType::Plus => match (&*left_value.borrow(), &*right_value.borrow()) {
                        (LoxValue::Number(left), LoxValue::Number(right)) => {
                            Ok(LoxValue::new(LoxValue::Number(left + right)))
                        }
                        (LoxValue::String(left), LoxValue::String(right)) => Ok(LoxValue::new(
                            LoxValue::String(format!("{}{}", left, right)),
                        )),
                        _ => Err(LoxInterpreterError::InterpreterUnexpectedOperation(
                            operator.get_lexeme().clone(),
                        )),
                    },
                    // greater than
                    LoxTokenType::Greater => Ok(LoxValue::new(LoxValue::Boolean(
                        Self::extract_number(&left_value)? > Self::extract_number(&right_value)?,
                    ))),
                    // greater or equal
                    LoxTokenType::GreaterEqual => Ok(LoxValue::new(LoxValue::Boolean(
                        Self::extract_number(&left_value)? >= Self::extract_number(&right_value)?,
                    ))),
                    // less than
                    LoxTokenType::Less => Ok(LoxValue::new(LoxValue::Boolean(
                        Self::extract_number(&left_value)? < Self::extract_number(&right_value)?,
                    ))),
                    // less or equal
                    LoxTokenType::LessEqual => Ok(LoxValue::new(LoxValue::Boolean(
                        Self::extract_number(&left_value)? <= Self::extract_number(&right_value)?,
                    ))),
                    // equality
                    LoxTokenType::EqualEqual => Ok(LoxValue::new(LoxValue::Boolean(
                        left_value.borrow().equals(&right_value.borrow()),
                    ))),
                    // non-equality
                    LoxTokenType::BangEqual => Ok(LoxValue::new(LoxValue::Boolean(
                        !left_value.borrow().equals(&right_value.borrow()),
                    ))),
                    // unexpected
                    _ => Err(LoxInterpreterError::InterpreterUnexpectedOperation(
                        operator.get_lexeme().clone(),
                    )),
                }
            }
            LoxExpression::Logical {
                left,
                operator,
                right,
            } => {
                let left_value = Self::evaluate_expression(left, env, locals)?;
                match operator.get_kind() {
                    LoxTokenType::Or => {
                        if left_value.borrow().is_truthy() {
                            Ok(left_value)
                        } else {
                            Self::evaluate_expression(right, env, locals)
                        }
                    }
                    LoxTokenType::And => {
                        if !left_value.borrow().is_truthy() {
                            Ok(left_value)
                        } else {
                            Self::evaluate_expression(right, env, locals)
                        }
                    }
                    _ => Err(LoxInterpreterError::InterpreterUnexpectedOperation(
                        operator.get_lexeme().to_string(),
                    )),
                }
            }
            LoxExpression::Variable { name } => {
                let value = env.borrow().get(name.get_lexeme().as_str())?;
                Ok(value)
            }
            LoxExpression::Assign { name, value } => {
                let evaluated_value = Self::evaluate_expression(value, env, locals)?;
                if let Some(distance) =
                    locals.get(&Self::compute_locals_key_from_expression(expression))
                {
                    environment_handle_assign_at_depth(
                        env,
                        name.get_lexeme(),
                        *distance,
                        evaluated_value.clone(),
                    );
                } else {
                    env.borrow_mut()
                        .assign(name.get_lexeme(), evaluated_value.clone())?;
                }
                Ok(evaluated_value)
            }
            LoxExpression::Get { name, object } => {
                let object_value = Self::evaluate_expression(object, env, locals)?;
                lox_value_handle_instance_get_field(&object_value, name)
            }
            LoxExpression::Set {
                name,
                object,
                value,
            } => {
                let mut object_value = Self::evaluate_expression(object, env, locals)?;
                let evaluated_value = Self::evaluate_expression(value, env, locals)?;
                lox_value_handle_instance_set_field(&mut object_value, name, evaluated_value)
            }
            LoxExpression::Call {
                callee,
                arguments,
                parenthesis,
            } => {
                let callee_value = Self::evaluate_expression(callee, env, locals)?;
                let mut arguments_values = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    arguments_values.push(Self::evaluate_expression(argument, env, locals)?);
                }
                callee_value.call(env, locals, &arguments_values, parenthesis)
            }
            LoxExpression::This { keyword } => {
                Self::lookup_variable(expression, keyword, env, locals)
            }
            LoxExpression::Super { keyword: _, method } => {
                let distance = locals.get(&Self::compute_locals_key_from_expression(expression)).expect("interpreter evaluating LoxExpression::Super expects a defined superclass method.");
                let super_class = environment_handle_get_at_depth(env, "super", *distance)?;
                let super_class_method = super_class.borrow().class_find_method(method.get_lexeme()).expect("interpreter evaluating LoxExpression::Super expects a defined superclass method.");
                let this_instance = environment_handle_get_at_depth(env, "this", distance - 1)?;
                Ok(super_class_method
                    .clone() // TODO: can we avoid this?
                    .borrow()
                    .class_method_bind_this(&this_instance)
                    .expect("superclass method value is a function"))
            }
        }
    }

    fn evaluate_literal(literal: &LoxLiteral) -> LoxValueHandle {
        match literal {
            LoxLiteral::Number(number) => LoxValue::new(LoxValue::Number(*number)),
            LoxLiteral::String(string) => LoxValue::new(LoxValue::String(string.clone())),
            LoxLiteral::True => LoxValue::new(LoxValue::Boolean(true)),
            LoxLiteral::False => LoxValue::new(LoxValue::Boolean(false)),
            LoxLiteral::Nil => LoxValue::new(LoxValue::Nil),
        }
    }

    fn extract_number(value: &LoxValueHandle) -> Result<f64> {
        value.borrow().as_number().ok_or_else(|| {
            LoxInterpreterError::InterpreterNotANumber(value.borrow().representation())
        })
    }
}
