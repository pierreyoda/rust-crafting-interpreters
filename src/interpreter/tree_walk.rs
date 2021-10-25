use crate::{
    errors::{LoxInterpreterError, Result},
    expressions::{LoxExpression, LoxLiteral},
    lexer::LoxTokenType,
    printer::LoxPrintable,
    values::LoxValue,
};

pub struct LoxTreeWalkEvaluator {}

impl LoxTreeWalkEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(&self, expression: &LoxExpression) -> Result<LoxValue> {
        match expression {
            LoxExpression::Literal { value } => Ok(Self::evaluate_literal(value)),
            LoxExpression::Group { expression: expr } => self.evaluate(expr),
            LoxExpression::Unary { operator, right } => {
                let right_value = self.evaluate(right)?;
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
                let (left_value, right_value) = (self.evaluate(left)?, self.evaluate(right)?);
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
            .ok_or(LoxInterpreterError::InterpreterNotANumber(
                value.representation(),
            ))
    }
}