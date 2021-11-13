use self::values::{LoxBytecodeValue, LoxValueArray};

pub mod compiler;
pub mod debug;
pub mod lexer;
pub mod values;
pub mod vm;

#[derive(Clone, Debug)]
pub enum LoxBytecodeOpcode {
    Value(usize),
    Constant,
    Nil,
    True,
    False,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Return,
}

impl LoxBytecodeOpcode {
    pub fn as_value(&self) -> Option<&usize> {
        match self {
            Self::Value(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoxBytecodeChunk {
    lines: Vec<usize>,
    constants: LoxValueArray,
    code: Vec<LoxBytecodeOpcode>,
}

impl Default for LoxBytecodeChunk {
    fn default() -> Self {
        Self {
            code: vec![],
            lines: vec![],
            constants: LoxValueArray::default(),
        }
    }
}

impl LoxBytecodeChunk {
    pub fn append(&mut self, bytecode: LoxBytecodeOpcode, line_number: usize) {
        self.code.push(bytecode);
        self.lines.push(line_number);
    }

    pub fn reallocate(&mut self, new_size: usize) {
        todo!()
    }

    pub fn add_constant(&mut self, value: LoxBytecodeValue) -> usize {
        self.constants.write(value);
        self.constants.count() - 1
    }

    pub fn get_constant(&self, index: usize) -> Option<&LoxBytecodeValue> {
        self.constants.read(index)
    }

    pub fn get_instruction(&self, offset: usize) -> Option<&LoxBytecodeOpcode> {
        self.code.get(offset)
    }

    pub fn get_instructions(&self) -> &[LoxBytecodeOpcode] {
        &self.code
    }

    pub fn get_line(&self, offset: usize) -> Option<usize> {
        self.lines.get(offset).cloned()
    }

    pub fn get_size(&self) -> usize {
        self.code.len()
    }
}
