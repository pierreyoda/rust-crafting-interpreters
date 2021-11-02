use crate::{
    callable::LoxCallable,
    errors::{LoxInterpreterError, Result},
    expressions::{LoxExpression, LoxLiteral, LoxOperation, LoxStatement},
    lexer::LoxTokenType,
    printer::LoxPrintable,
    values::LoxValue,
};

use super::{
    builtins::build_lox_clock_builtin,
    environment::{LoxEnvironment, LoxEnvironmentHandle},
};

pub struct LoxTreeWalkEvaluator {
    globals: LoxEnvironmentHandle,
}

impl LoxTreeWalkEvaluator {
    pub fn new() -> Self {
        let globals = LoxEnvironment::new(None);
        globals
            .borrow_mut()
            .define("clock".into(), build_lox_clock_builtin());
        Self { globals }
    }

    pub fn get_environment(&self) -> &LoxEnvironmentHandle {
        &self.globals
    }

    pub fn evaluate(&mut self, operation: &LoxOperation) -> Result<LoxValue> {
        match operation {
            LoxOperation::Invalid => Ok(LoxValue::Nil),
            LoxOperation::Expression(expression) => {
                Self::evaluate_expression(expression, &mut self.globals)
            }
            LoxOperation::Statement(statement) => {
                Self::evaluate_statement(statement, &mut self.globals)
            }
        }
    }

    fn evaluate_statement(
        statement: &LoxStatement,
        env: &mut LoxEnvironmentHandle,
    ) -> Result<LoxValue> {
        match statement {
            LoxStatement::NoOp => Ok(LoxValue::Nil),
            LoxStatement::Expression { expression } => {
                Self::evaluate_expression(expression, env)?;
                Ok(LoxValue::Nil)
            }
            LoxStatement::Print { expression } => {
                let value = Self::evaluate_expression(expression, env)?;
                println!("{}", value.representation());
                Ok(LoxValue::Nil)
            }
            LoxStatement::Variable { name, initializer } => {
                let value = Self::evaluate_expression(initializer, env)?;
                env.borrow_mut().define(name.get_lexeme().clone(), value);
                Ok(LoxValue::Nil)
            }
            LoxStatement::Block { statements } => {
                let mut block_env = LoxEnvironment::new(Some(env.clone()));
                Self::execute_block_statement(statements, &mut block_env)
            }
            LoxStatement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_value = Self::evaluate_expression(condition, env)?;
                if condition_value.is_truthy() {
                    Self::evaluate_statement(then_branch, env)?;
                } else if !else_branch.is_noop() {
                    Self::evaluate_statement(else_branch, env)?;
                }
                Ok(LoxValue::Nil)
            }
            LoxStatement::While { condition, body } => {
                while Self::evaluate_expression(condition, env)?.is_truthy() {
                    let _ = Self::evaluate_statement(body, env)?;
                }
                Ok(LoxValue::Nil)
            }
            LoxStatement::Function {
                name,
                parameters,
                body: _,
            } => {
                let function = LoxValue::Function {
                    arity: parameters.len(),
                    declaration: Box::new(statement.clone()),
                    closure: env.clone(),
                };
                env.borrow_mut().define(name.get_lexeme().clone(), function);
                Ok(LoxValue::Nil)
            }
            LoxStatement::Return { keyword: _, value } => {
                let returned_value = if value.is_noop() {
                    LoxValue::Nil
                } else {
                    Self::evaluate_expression(value, env)?
                };
                Err(LoxInterpreterError::InterpreterReturn(returned_value))
            }
            LoxStatement::Class {
                name,
                super_class,
                methods,
            } => todo!(),
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
    ) -> Result<LoxValue> {
        for statement in statements {
            Self::evaluate_statement(statement, env)?;
        }
        Ok(LoxValue::Nil)
    }

    fn evaluate_expression(
        expression: &LoxExpression,
        env: &mut LoxEnvironmentHandle,
    ) -> Result<LoxValue> {
        match expression {
            LoxExpression::NoOp => Ok(LoxValue::Nil),
            LoxExpression::Literal { value } => Ok(Self::evaluate_literal(value)),
            LoxExpression::Group { expression: expr } => Self::evaluate_expression(expr, env),
            LoxExpression::Unary { operator, right } => {
                let right_value = Self::evaluate_expression(right, env)?;
                match operator.get_kind() {
                    // number inversion
                    LoxTokenType::Minus => {
                        Ok(LoxValue::Number(-Self::extract_number(&right_value)?))
                    }
                    // logical not
                    LoxTokenType::Bang => Ok(LoxValue::Boolean(!right_value.is_truthy())),
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
                    Self::evaluate_expression(left, env)?,
                    Self::evaluate_expression(right, env)?,
                );
                match operator.get_kind() {
                    // subtraction
                    LoxTokenType::Minus => Ok(LoxValue::Number(
                        Self::extract_number(&left_value)? - Self::extract_number(&right_value)?,
                    )),
                    // division
                    LoxTokenType::Slash => Ok(LoxValue::Number(
                        Self::extract_number(&left_value)? / Self::extract_number(&right_value)?,
                    )),
                    // multiplication
                    LoxTokenType::Star => Ok(LoxValue::Number(
                        Self::extract_number(&left_value)? * Self::extract_number(&right_value)?,
                    )),
                    // addition and string concatenation
                    LoxTokenType::Plus => match (left_value, right_value) {
                        (LoxValue::Number(left), LoxValue::Number(right)) => {
                            Ok(LoxValue::Number(left + right))
                        }
                        (LoxValue::String(left), LoxValue::String(right)) => {
                            Ok(LoxValue::String(format!("{}{}", left, right)))
                        }
                        _ => Err(LoxInterpreterError::InterpreterUnexpectedOperation(
                            operator.get_lexeme().clone(),
                        )),
                    },
                    // greater than
                    LoxTokenType::Greater => Ok(LoxValue::Boolean(
                        Self::extract_number(&left_value)? > Self::extract_number(&right_value)?,
                    )),
                    // greater or equal
                    LoxTokenType::GreaterEqual => Ok(LoxValue::Boolean(
                        Self::extract_number(&left_value)? >= Self::extract_number(&right_value)?,
                    )),
                    // less than
                    LoxTokenType::Less => Ok(LoxValue::Boolean(
                        Self::extract_number(&left_value)? < Self::extract_number(&right_value)?,
                    )),
                    // less or equal
                    LoxTokenType::LessEqual => Ok(LoxValue::Boolean(
                        Self::extract_number(&left_value)? <= Self::extract_number(&right_value)?,
                    )),
                    // equality
                    LoxTokenType::EqualEqual => {
                        Ok(LoxValue::Boolean(left_value.equals(&right_value)))
                    }
                    // non-equality
                    LoxTokenType::BangEqual => {
                        Ok(LoxValue::Boolean(!left_value.equals(&right_value)))
                    }
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
                let left_value = Self::evaluate_expression(left, env)?;
                match operator.get_kind() {
                    LoxTokenType::Or => {
                        if left_value.is_truthy() {
                            Ok(left_value)
                        } else {
                            Self::evaluate_expression(right, env)
                        }
                    }
                    LoxTokenType::And => {
                        if !left_value.is_truthy() {
                            Ok(left_value)
                        } else {
                            Self::evaluate_expression(right, env)
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
                let evaluated_value = Self::evaluate_expression(value, env)?;
                env.borrow_mut()
                    .assign(name.get_lexeme(), evaluated_value.clone())?;
                Ok(evaluated_value)
            }
            LoxExpression::Call {
                callee,
                arguments,
                parenthesis,
            } => {
                let callee_value = Self::evaluate_expression(callee, env)?;
                let mut arguments_values = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    arguments_values.push(Self::evaluate_expression(argument, env)?);
                }
                callee_value.call(env, &arguments_values, parenthesis)
            }
            _ => todo!(),
        }
    }

    fn evaluate_literal(literal: &LoxLiteral) -> LoxValue {
        match literal {
            LoxLiteral::Number(number) => LoxValue::Number(*number),
            LoxLiteral::String(string) => LoxValue::String(string.clone()),
            LoxLiteral::True => LoxValue::Boolean(true),
            LoxLiteral::False => LoxValue::Boolean(false),
            LoxLiteral::Nil => LoxValue::Nil,
        }
    }

    fn extract_number(value: &LoxValue) -> Result<f64> {
        value
            .as_number()
            .ok_or_else(|| LoxInterpreterError::InterpreterNotANumber(value.representation()))
    }
}
