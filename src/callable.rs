use crate::{
    errors::Result, interpreter::environment::LoxEnvironmentHandle, lexer::LoxToken,
    printer::LoxPrintable, values::LoxValue,
};

pub trait LoxCallable: LoxPrintable {
    fn arity(&self) -> Option<usize>;

    fn call(
        &self,
        env: &mut LoxEnvironmentHandle,
        arguments: &[LoxValue],
        parenthesis: &LoxToken,
    ) -> Result<LoxValue>;
}
