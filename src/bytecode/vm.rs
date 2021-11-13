use crate::errors::BResult;

use super::{
    debug::{disassemble_instruction, print_value},
    lexer::LoxBytecodeLexer,
    values::LoxBytecodeValue,
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
    stack: [LoxBytecodeValue; LOX_STACK_MAX],
    stack_index: usize,
}

fn stack_init<const N: usize>() -> [LoxBytecodeValue; N] {
    let mut vec = Vec::with_capacity(N);
    for _ in 0..N {
        vec.push(LoxBytecodeValue::Nil);
    }
    vec.try_into().unwrap()
}

macro_rules! vm_binary_operation {
    ($self: ident, $operator: tt, $value_type: path) => {{
        // type checking
        if !$self.peek(0).is_number() || !$self.peek(1).is_number() {
            $self.runtime_error("Operands must be a numbers.");
            return Ok(LoxInterpreterResult::RuntimeError);
        }
        // watch out for the pop order
        let b = $self.stack_pop().as_number().expect("vm.binary_operation expects a number value");
        let a = $self.stack_pop().as_number().expect("vm.binary_operation expects a number value");
        $self.stack_push($value_type(a $operator b));
    }};
}

impl Default for LoxBytecodeVirtualMachine {
    fn default() -> Self {
        Self {
            chunk: LoxBytecodeChunk::default(),
            instruction_pointer: 0,
            stack: stack_init(),
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
                    let constant = self
                        .chunk
                        .get_constant(constant_index)
                        .expect("the constant must exist")
                        .clone();
                    self.stack_push(constant);
                }
                LoxBytecodeOpcode::Nil => self.stack_push(LoxBytecodeValue::Nil),
                LoxBytecodeOpcode::True => self.stack_push(LoxBytecodeValue::Boolean(true)),
                LoxBytecodeOpcode::False => self.stack_push(LoxBytecodeValue::Boolean(false)),
                LoxBytecodeOpcode::Equal => {
                    let b = self.stack_pop().clone(); // TODO: can we avoid this?
                    let a = self.stack_pop();
                    let value = a.equals(&b);
                    self.stack_push(LoxBytecodeValue::Boolean(value));
                }
                LoxBytecodeOpcode::Greater => {
                    vm_binary_operation!(self, >, LoxBytecodeValue::Boolean)
                }
                LoxBytecodeOpcode::Less => {
                    vm_binary_operation!(self, <, LoxBytecodeValue::Boolean)
                }
                LoxBytecodeOpcode::Add => vm_binary_operation!(self, +, LoxBytecodeValue::Number),
                LoxBytecodeOpcode::Subtract => {
                    vm_binary_operation!(self, -, LoxBytecodeValue::Number)
                }
                LoxBytecodeOpcode::Multiply => {
                    vm_binary_operation!(self, *, LoxBytecodeValue::Number)
                }
                LoxBytecodeOpcode::Divide => {
                    vm_binary_operation!(self, /, LoxBytecodeValue::Number)
                }
                LoxBytecodeOpcode::Not => {
                    let value = self.stack_pop().is_falsy();
                    self.stack_push(LoxBytecodeValue::Boolean(value));
                }
                LoxBytecodeOpcode::Negate => {
                    if let LoxBytecodeValue::Number(value) = self.peek(0).clone() {
                        self.stack_pop();
                        self.stack_push(LoxBytecodeValue::Number(-value));
                    } else {
                        self.runtime_error("Operand must be a number.");
                        return Ok(LoxInterpreterResult::RuntimeError);
                    }
                }
                LoxBytecodeOpcode::Return => {
                    print_value(self.stack_pop());
                    println!();
                }
                _ => panic!(
                    "vm.interpret instruction not implemented: {:?}",
                    instruction
                ),
            }
        }
        Ok(LoxInterpreterResult::Ok)
    }

    fn stack_push(&mut self, value: LoxBytecodeValue) {
        self.stack[self.stack_index] = value;
        self.stack_index += 1;
    }

    fn stack_pop(&mut self) -> &LoxBytecodeValue {
        self.stack_index -= 1;
        self.stack
            .last()
            .expect("the stack should not be empty when popped")
    }

    fn stack_reset(&mut self) {
        self.stack_index = 0; // TODO: check that this is the correct behavior
    }

    fn peek(&self, distance: usize) -> &LoxBytecodeValue {
        self.stack
            .get(LOX_STACK_MAX - 1 - distance)
            .unwrap_or_else(|| panic!("vm.peek({}) expects a valid stack value", distance))
    }

    fn runtime_error<S: AsRef<str> + std::fmt::Display>(&mut self, message: S) {
        println!("{}", message);
        let instruction_offset = self.instruction_pointer - self.chunk.get_size() - 1; // TODO: check formula
        let line_number = self
            .chunk
            .get_line(instruction_offset)
            .expect("vm.runtime_error should be able to get the line number");
        println!("[line {}] in script", line_number);
        self.stack_reset();
    }
}

// TODO: unit tests
