use crate::{
    errors::Result, expressions::LoxExpression, lexer::Lexer, parser::Parser, values::LoxValue,
};

use self::tree_walk::LoxTreeWalkEvaluator;

mod tree_walk;

pub trait LoxInterpreter {
    fn new() -> Self;

    fn parse(&self, source: String) -> Result<LoxExpression> {
        let lexer = Lexer::from_source(source)?;
        Parser::from_tokens(lexer.get_tokens().clone()).parse()
    }

    fn evaluate(&mut self, expression: &LoxExpression) -> Result<LoxValue>;
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

    fn evaluate(&mut self, expression: &LoxExpression) -> Result<LoxValue> {
        self.evaluator.evaluate(&expression)
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
        let ast = interpreter.parse(source.to_string()).unwrap();
        let result = interpreter.evaluate(&ast).unwrap();
        assert!(result.equals(&LoxValue::Number(2.0)));
        assert_eq!(result.representation(), "2".to_string());
    }
}
