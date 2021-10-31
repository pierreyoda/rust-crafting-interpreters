use crate::{
    errors::Result, interpreter::environment::LoxEnvironment, lexer::LoxToken,
    printer::LoxPrintable, values::LoxValue,
};

pub trait LoxCallable: LoxPrintable {
    fn arity(&self) -> Option<usize>;

    fn call(
        &self,
        env: &mut LoxEnvironment,
        arguments: &[LoxValue],
        parenthesis: &LoxToken,
    ) -> Result<LoxValue>;
}
