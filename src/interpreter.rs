use crate::{
    errors::Result, expressions::LoxOperation, lexer::Lexer, parser::Parser, values::LoxValue,
};

use self::{environment::LoxEnvironmentHandle, tree_walk::LoxTreeWalkEvaluator};

pub mod builtins;
pub mod environment;
pub mod tree_walk;

pub trait LoxInterpreter {
    fn parse(&self, source: String) -> Result<Vec<LoxOperation>> {
        let lexer = Lexer::from_source(source)?;
        Parser::from_tokens(lexer.get_tokens().clone()).parse()
    }

    fn interpret(&mut self, operations: &[LoxOperation]) -> Result<LoxValue>;

    fn get_environment(&self) -> &LoxEnvironmentHandle;
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

    fn get_environment(&self) -> &LoxEnvironmentHandle {
        self.evaluator.get_environment()
    }
}

#[cfg(test)]
mod tests {
    use crate::{printer::operations_representation, values::LoxValue};

    use super::{LoxInterpreter, LoxTreeWalkInterpreter};

    #[test]
    fn test_interpreter_parsing_and_ast_printing() {
        let test_data = vec![
            (
                "var computed = (5 - (3 - 1)) + -1;",
                "(var computed = (+ (group (- 5 (group (- 3 1)))) (- 1)))",
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
            (
                r#"
            var counter = 0;
            while (counter < 5) {
                counter = 10;
                print counter;
            }"#,
                "(var counter = 0)\n(while (< counter 5) (block (; (= counter 10))(print counter)))",
            ),
                        (
                            r#"
            var a = 0;
            var temp = 0;
            for (var b = 1; a < 10000; b = temp + b) {
                print a;
                temp = a;
                a = b;
            }
            "#,
                            r#"(var a = 0)
(var temp = 0)
(block (var b = 1)(while (< a 10000) (block (block (print a)(; (= temp a))(; (= a b)))(; (= b (+ temp b))))))"#,
                        ),
                        (
                            r#"
            fun add(a, b) {
                return a + b;
            }
            print add(1, 2);
                        "#,
                            "(fun add (a b) (return (+ a b)))\n(print (call add 1 2))",
                        ),
            (
                r#"
fun makeCounter() {
    var i = 0;
    fun count() {
        i = i + 1;
        print i;
    }

    return count;
}

var counter = makeCounter();
counter(); // 1
counter(); // 2
"#,
                r#"(fun makeCounter () (var i = 0)(fun count () (; (= i (+ i 1)))(print i))(return count))
(var counter = (call makeCounter))
(; (call counter))
(; (call counter))"#,
            ),
        ];

        let interpreter = LoxTreeWalkInterpreter::new();
        for (source, expected) in test_data {
            let parsed = interpreter.parse(source.to_string()).unwrap();
            assert_eq!(operations_representation(&parsed), expected);
        }
    }

    #[test]
    fn test_tree_walk_interpreter_basic_variables() {
        let source = r#"
var variable = "before";
variable = "after";
        "#;
        let mut interpreter = LoxTreeWalkInterpreter::new();
        let operations = interpreter.parse(source.to_string()).unwrap();
        assert_eq!(
            operations_representation(&operations),
            "(var variable = before)\n(; (= variable after))"
        );
        let _ = interpreter.interpret(&operations).unwrap();
        let variable = interpreter
            .get_environment()
            .borrow()
            .get("variable")
            .unwrap();
        assert!(variable.equals(&LoxValue::String("after".into())));
    }
}
