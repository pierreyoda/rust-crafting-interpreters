use crate::{
    errors::Result, expressions::LoxOperation, lexer::Lexer, parser::Parser, values::LoxValue,
};

use self::{environment::LoxEnvironment, tree_walk::LoxTreeWalkEvaluator};

mod environment;
mod tree_walk;

pub trait LoxInterpreter {
    fn new() -> Self;

    fn parse(&self, source: String) -> Result<Vec<LoxOperation>> {
        let lexer = Lexer::from_source(source)?;
        Parser::from_tokens(lexer.get_tokens().clone()).parse()
    }

    fn interpret(&mut self, operations: &[LoxOperation]) -> Result<LoxValue>;

    fn get_environment(&self) -> &LoxEnvironment;
}

pub struct LoxTreeWalkInterpreter {
    evaluator: LoxTreeWalkEvaluator,
}

impl LoxInterpreter for LoxTreeWalkInterpreter {
    fn new() -> Self {
        Self {
            evaluator: LoxTreeWalkEvaluator::new(),
        }
    }

    fn interpret(&mut self, operations: &[LoxOperation]) -> Result<LoxValue> {
        let mut last_value = LoxValue::Nil;
        for operation in operations {
            last_value = self.evaluator.evaluate(operation)?;
        }
        Ok(last_value)
    }

    fn get_environment(&self) -> &LoxEnvironment {
        self.evaluator.get_environment()
    }
}

#[cfg(test)]
mod tests {
    use crate::{printer::LoxPrintable, values::LoxValue};

    use super::{LoxInterpreter, LoxTreeWalkInterpreter};

    #[test]
    fn test_interpreter_parsing_and_ast_printing() {
        let source = "(5 - (3 - 1)) + -1";
        let ast = LoxTreeWalkInterpreter::new()
            .parse(source.to_string())
            .unwrap()[0]
            .clone()
            .as_expression()
            .unwrap();
        let ast_representation = ast.representation();
        assert_eq!(
            ast_representation,
            "(+ (group (- 5 (group (- 3 1)))) (- 1))".to_string()
        );
    }

    #[test]
    fn test_tree_walk_interpreter_basic_evaluation() {
        let source = "(5 - (3 - 1)) + -1";
        let mut interpreter = LoxTreeWalkInterpreter::new();
        let operations = interpreter.parse(source.to_string()).unwrap();
        let result = interpreter.interpret(&operations).unwrap();
        assert!(result.equals(&LoxValue::Number(2.0)));
        assert_eq!(result.representation(), "2".to_string());
    }

    #[test]
    fn test_tree_walk_interpreter_basic_variables() {
        let source = r#"
var variable = "before";
variable = "after";
        "#;
        let mut interpreter = LoxTreeWalkInterpreter::new();
        let operations = interpreter.parse(source.to_string()).unwrap();
        let _ = interpreter.interpret(&operations).unwrap();
        assert!(interpreter
            .get_environment()
            .get("variable")
            .unwrap()
            .equals(&LoxValue::String("after".into())));
    }
}
