use crate::{
    errors::Result, expressions::LoxOperation, lexer::Lexer, parser::Parser, values::LoxValue,
};

use self::{environment::LoxEnvironment, tree_walk::LoxTreeWalkEvaluator};

pub mod builtins;
pub mod environment;
pub mod tree_walk;

pub trait LoxInterpreter {
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

impl LoxTreeWalkInterpreter {
    pub fn new() -> Self {
        Self {
            evaluator: LoxTreeWalkEvaluator::new(),
        }
    }
}

impl LoxInterpreter for LoxTreeWalkInterpreter {
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
    use crate::{
        printer::{operations_representation, LoxPrintable},
        values::LoxValue,
    };

    use super::{LoxInterpreter, LoxTreeWalkInterpreter};

    #[test]
    fn test_interpreter_parsing_and_ast_printing() {
        let test_data = vec![
            (
                "(5 - (3 - 1)) + -1",
                "(+ (group (- 5 (group (- 3 1)))) (- 1))",
            ),
            (
                r#"
            {
                var a = "outer";
                {
                    print a;
                }
            }
                        "#,
                "(block (var a = outer)(block (print a)))",
            ),
            (
                r#"
            var a = 10;
            if (a > 5) {
                print a - 5;
            } else {
                print a;
            }
                            "#,
                "(var a = 10)\n(if-else (> a 5) (block (print (- a 5))) (block (print a)))",
            ),
            // (
            //     r#"
            // var counter = 0;
            // while (counter < 5) {
            //     counter = 10;
            //     print counter;
            // }"#,
            //     "",
            // ),
            //             (
            //                 r#"
            // var a = 0;
            // var temp = 0;
            // for (var b = 1; a < 10000; b = temp + b) {
            //     print a;
            //     temp = a;
            //     a = b;
            // }
            // "#,
            //                 "",
            //             ),
        ];

        let interpreter = LoxTreeWalkInterpreter::new();
        for (source, expected) in test_data {
            let parsed = interpreter.parse(source.to_string()).unwrap();
            assert_eq!(operations_representation(&parsed), expected);
        }
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
