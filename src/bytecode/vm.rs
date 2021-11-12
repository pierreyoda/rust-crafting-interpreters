use crate::errors::BResult;

use super::{
    debug::{disassemble_instruction, print_value},
    lexer::LoxBytecodeLexer,
    values::LoxValueNumber,
    LoxBytecodeChunk, LoxBytecodeOpcode,
};

const LOX_STACK_MAX: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoxInterpreterResult {
    Ok,
    CompilationError,
    RuntimeError,
}

pub struct LoxBytecodeVirtualMachine {
    chunk: LoxBytecodeChunk,
    instruction_pointer: usize,
    stack: [LoxValueNumber; LOX_STACK_MAX],
    stack_index: usize,
}

macro_rules! vm_binary_operation {
    ($self: ident, $operator: tt) => {
        {
            // watch out for the pop order
            let b = *$self.stack_pop();
            let a = *$self.stack_pop();
            $self.stack_push(a $operator b);
        }
    };
}

impl Default for LoxBytecodeVirtualMachine {
    fn default() -> Self {
        Self {
            chunk: LoxBytecodeChunk::default(),
            instruction_pointer: 0,
            stack: [0.0; LOX_STACK_MAX],
            stack_index: 0,
        }
    }
}

impl LoxBytecodeVirtualMachine {
    pub fn run_code(&mut self, code: &String) -> BResult<LoxInterpreterResult> {
        let mut lexer = LoxBytecodeLexer::default();
        let parsed = lexer.compile(code)?;
        self.interpret()
    }

    pub fn interpret(&mut self) -> BResult<LoxInterpreterResult> {
        let instructions = self.chunk.get_instructions().to_vec();
        while let Some(instruction) = instructions.get(self.instruction_pointer) {
            #[cfg(feature = "bytecode-tracing")]
            {
                print!("          ");
                for index in 0..self.stack_index {
                    print!("[ ");
                    print_value(self.stack[index]);
                    print!(" ]");
                }
                println!();
                disassemble_instruction(&self.chunk, self.instruction_pointer); // TODO: check offset
            }

            match instruction {
                LoxBytecodeOpcode::Constant => {
                    let constant_index = *instructions
                        .get(self.instruction_pointer + 1)
                        .expect("constant opcode is followed by value")
                        .as_value()
                        .expect("next opcode after constant opcode must be a value");
                    let constant = *self
                        .chunk
                        .get_constant(constant_index)
                        .expect("the constant must exist");
                    self.stack_push(constant);
                }
                LoxBytecodeOpcode::Add => vm_binary_operation!(self, +),
                LoxBytecodeOpcode::Subtract => vm_binary_operation!(self, -),
                LoxBytecodeOpcode::Multiply => vm_binary_operation!(self, *),
                LoxBytecodeOpcode::Divide => vm_binary_operation!(self, /),
                LoxBytecodeOpcode::Negate => {
                    let negated = -self.stack_pop();
                    self.stack_push(negated);
                }
                LoxBytecodeOpcode::Return => {
                    print_value(self.stack_pop());
                    println!();
                    return Ok(LoxInterpreterResult::Ok);
                }
                _ => (),
            }
        }
        Ok(LoxInterpreterResult::Ok)
    }

    fn stack_push(&mut self, value: LoxValueNumber) {
        self.stack[self.stack_index] = value;
        self.stack_index += 1;
    }

    fn stack_pop(&mut self) -> &LoxValueNumber {
        self.stack_index -= 1;
        self.stack
            .last()
            .expect("the stack should not be empty when popped")
    }
}
