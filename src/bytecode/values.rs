use crate::printer::LoxPrintable;

pub const LOX_NUMBER_VALUE_COMPARISON_EPSILON: f64 = f64::EPSILON;

#[derive(Clone, Debug)]
pub enum LoxBytecodeObject {
    String(String),
}

impl LoxBytecodeObject {
    fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(left), Self::String(right)) => left == right,
            _ => false,
        }
    }
}

impl LoxPrintable for LoxBytecodeObject {
    fn representation(&self) -> String {
        match self {
            Self::String(string) => string.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LoxBytecodeValue {
    Nil,
    Number(f64),
    Boolean(bool),
    Object(LoxBytecodeObject),
}

impl LoxBytecodeValue {
    pub fn is_falsy(&self) -> bool {
        match self {
            Self::Nil => true,
            Self::Boolean(value) => !value,
            _ => false,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let Self::Number(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Self::Object(object) = self {
            if let LoxBytecodeObject::String(string) = object {
                Some(string)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn is_string(&self) -> bool {
        if let Self::Object(object) = self {
            matches!(object, LoxBytecodeObject::String(_))
        } else {
            false
        }
    }

    pub fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Nil, _) => false,
            (Self::Boolean(left), Self::Boolean(right)) => *left == *right,
            (Self::Number(left), Self::Number(right)) => {
                (left - right).abs() < LOX_NUMBER_VALUE_COMPARISON_EPSILON
            }
            (Self::Object(left), Self::Object(right)) => left.equals(right),
            _ => false,
        }
    }
}

impl LoxPrintable for LoxBytecodeValue {
    fn representation(&self) -> String {
        match self {
            Self::Nil => "nil".to_string(),
            Self::Number(value) => format!("{}", value),
            Self::Boolean(boolean) => (if *boolean { "true" } else { "false" }).to_string(),
            Self::Object(object) => object.representation(),
        }
    }
}

/// Constants pool.
#[derive(Clone, Debug, Default)]
pub struct LoxValueArray {
    values: Vec<LoxBytecodeValue>,
}

impl LoxValueArray {
    pub fn read(&self, index: usize) -> Option<&LoxBytecodeValue> {
        self.values.get(index)
    }

    pub fn write(&mut self, value: LoxBytecodeValue) {
        self.values.push(value);
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}
