use crate::values::LoxValue;

pub type LoxValueNumber = f64;

/// Constants pool.
#[derive(Clone, Debug, Default)]
pub struct LoxValueArray {
    values: Vec<LoxValueNumber>,
}

impl LoxValueArray {
    pub fn read(&self, index: usize) -> Option<&LoxValueNumber> {
        self.values.get(index)
    }

    pub fn write(&mut self, value: LoxValueNumber) {
        self.values.push(value);
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}
