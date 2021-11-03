use std::collections::HashMap;

use crate::{
    errors::{LoxInterpreterError, Result},
    expressions::{LoxExpression, LoxOperation, LoxStatement},
    lexer::LoxToken,
};

use super::tree_walk::LoxTreeWalkEvaluator;

#[derive(Clone, PartialEq, Eq)]
enum LoxClassType {
    None,
    Class,
}

#[derive(Clone, PartialEq, Eq)]
enum LoxFunctionType {
    None,
    Function,
    ClassMethod,
    ClassInitializer,
}

type LoxLexicalScope = HashMap<String, bool>;

pub struct LoxResolver {
    evaluator: LoxTreeWalkEvaluator,
    /// LIFO stack of block scopes.
    scopes: Vec<LoxLexicalScope>,
    current_class_kind: LoxClassType,
    current_function_kind: LoxFunctionType,
}

impl LoxResolver {
    pub fn new(evaluator: LoxTreeWalkEvaluator) -> Self {
        Self {
            evaluator,
            scopes: vec![],
            current_class_kind: LoxClassType::None,
            current_function_kind: LoxFunctionType::None,
        }
    }

    pub fn get_evaluator(&self) -> &LoxTreeWalkEvaluator {
        &self.evaluator
    }
    pub fn get_evaluator_mut(&mut self) -> &mut LoxTreeWalkEvaluator {
        &mut self.evaluator
    }

    pub fn resolve(&mut self, operation: &LoxOperation) -> Result<()> {
        match operation {
            LoxOperation::Invalid => Ok(()),
            LoxOperation::Statement(statement) => self.resolve_statement(statement),
            LoxOperation::Expression(expression) => self.resolve_expression(expression),
        }
    }

    fn resolve_statements(&mut self, statements: &[LoxStatement]) -> Result<()> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    pub fn resolve_statement(&mut self, statement: &LoxStatement) -> Result<()> {
        match statement {
            LoxStatement::NoOp => (),
            LoxStatement::Block { statements } => {
                self.begin_scope();
                self.resolve_statements(statements)?;
                self.end_scope();
            }
            LoxStatement::Expression { expression } => self.resolve_expression(expression)?,
            LoxStatement::Variable { name, initializer } => {
                self.declare(name)?;
                if !initializer.is_noop() {
                    self.resolve_expression(initializer)?;
                }
                self.define(name);
            }
            LoxStatement::Function {
                name,
                parameters: _,
                body: _,
            } => {
                self.declare(name)?;
                self.define(name);
                self.resolve_function(statement, LoxFunctionType::Function)?;
            }
            LoxStatement::Return { keyword, value } => {
                if self.current_function_kind == LoxFunctionType::None {
                    return Err(LoxInterpreterError::ResolverImpossibleTopLevelReturn(
                        keyword.clone(),
                    ));
                }
                if !value.is_noop() {
                    if self.current_function_kind == LoxFunctionType::ClassInitializer {
                        return Err(LoxInterpreterError::ResolverImpossibleInitializerReturn(
                            keyword.clone(),
                        ));
                    }
                    self.resolve_expression(value)?;
                }
            }
            LoxStatement::Class {
                name,
                super_class,
                methods,
            } => {
                let enclosing_class_kind = self.current_class_kind.clone();
                self.current_class_kind = LoxClassType::Class;
                self.declare(name)?;
                self.define(name);
                self.begin_scope();
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert("this".into(), true);
                }
                for method in methods {
                    self.resolve_function(
                        method,
                        if method
                            .deconstruct_function_declaration()
                            .unwrap()
                            .0
                            .get_lexeme()
                            == "init"
                        {
                            LoxFunctionType::ClassInitializer
                        } else {
                            LoxFunctionType::ClassMethod
                        },
                    )?;
                }
                self.end_scope();
                self.current_class_kind = enclosing_class_kind;
            }
            LoxStatement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch)?;
                if !else_branch.is_noop() {
                    self.resolve_statement(else_branch)?;
                }
            }
            LoxStatement::While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)?;
            }
            LoxStatement::Print { expression } => self.resolve_expression(expression)?,
        }
        Ok(())
    }

    fn resolve_expression(&mut self, expression: &LoxExpression) -> Result<()> {
        match expression {
            LoxExpression::NoOp => (),
            LoxExpression::This { keyword } => {
                if self.current_class_kind == LoxClassType::None {
                    return Err(LoxInterpreterError::ResolverImpossibleThisUsage(
                        keyword.clone(),
                    ));
                }
                self.resolve_local_variable(expression, keyword)?;
            }
            LoxExpression::Super { keyword: _, method } => todo!(),
            LoxExpression::Variable { name } => {
                if let Some(scope) = self.scopes.last() {
                    if scope.get(name.get_lexeme()) == Some(&false) {
                        return Err(LoxInterpreterError::ResolverRecursiveLocalAssignment(
                            name.clone(),
                        ));
                    }
                    self.resolve_local_variable(expression, name)?;
                }
            }
            LoxExpression::Assign { name, value } => {
                self.resolve_expression(value)?;
                self.resolve_local_variable(expression, name)?;
            }
            LoxExpression::Get { name: _, object } => {
                self.resolve_expression(object)?;
            }
            LoxExpression::Set {
                name: _,
                object,
                value,
            } => {
                self.resolve_expression(value)?;
                self.resolve_expression(object)?;
            }
            LoxExpression::Call {
                callee,
                arguments,
                parenthesis: _,
            } => {
                self.resolve_expression(callee)?;
                for argument in arguments {
                    self.resolve_expression(argument)?;
                }
            }
            LoxExpression::Unary { right, operator: _ } => self.resolve_expression(right)?,
            LoxExpression::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            LoxExpression::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            LoxExpression::Literal { value: _ } => (),
            LoxExpression::Group { expression } => self.resolve_expression(expression)?,
        }
        Ok(())
    }

    fn resolve_function(&mut self, function: &LoxStatement, kind: LoxFunctionType) -> Result<()> {
        match function {
            LoxStatement::Function {
                name: _,
                parameters,
                body,
            } => {
                let enclosing_function_kind = self.current_function_kind.clone();
                self.current_function_kind = kind;
                self.begin_scope();
                for parameter in parameters {
                    self.declare(parameter)?;
                    self.define(parameter);
                }
                self.resolve_statements(body)?;
                self.end_scope();
                self.current_function_kind = enclosing_function_kind;
                Ok(())
            }
            _ => Err(LoxInterpreterError::ResolverUnexpectedOperation(
                "resolve_function expected a function".into(),
            )),
        }
    }

    fn resolve_local_variable(
        &mut self,
        expression: &LoxExpression,
        name: &LoxToken,
    ) -> Result<()> {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name.get_lexeme()) {
                // TODO:
                // self.interpreter.resolve(expression, self.scopes.len() - 1 - i)?;
            }
        }
        Ok(())
    }

    /// Declares a variable in the innermost scope in order to shadow any outer one.
    fn declare(&mut self, name: &LoxToken) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name.get_lexeme()) {
                return Err(LoxInterpreterError::ResolverDuplicateVariableDeclaration(
                    name.clone(),
                ));
            }
            scope.insert(name.get_lexeme().clone(), false);
        }
        Ok(())
    }

    /// Marks a variable as defined in the innermost scope.
    fn define(&mut self, name: &LoxToken) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.get_lexeme().clone(), true);
        }
    }

    /// Create a new block scope.
    fn begin_scope(&mut self) {
        self.scopes.push(LoxLexicalScope::new());
    }

    /// Exit the current block scope, if any.
    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
