pub enum LoxAutoTestAssert {
    ExpectOutput(String),
}

pub struct LoxAutoTestDescription {
    code: String,
    asserts: Vec<LoxAutoTestAssert>,
}
