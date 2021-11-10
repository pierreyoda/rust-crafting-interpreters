use crate::bytecode::LoxBytecodeOpcode;

use super::{values::LoxValueNumber, LoxBytecodeChunk};

pub fn disassemble_instruction(chunk: &LoxBytecodeChunk, offset: usize) -> usize {
    print!("{:04}", offset);
    let line_number = chunk.get_line(offset);
    if offset > 0 && line_number == chunk.get_line(offset + 1) {
        print!("   | ");
    } else {
        print!("{:04}", line_number.unwrap());
    }
    if let Some(instruction) = chunk.get_instruction(offset) {
        match instruction {
            LoxBytecodeOpcode::Constant => constant_instruction("OP_CONSTANT", chunk, offset),
            LoxBytecodeOpcode::Add => simple_instruction("OP_ADD", offset),
            LoxBytecodeOpcode::Subtract => simple_instruction("OP_SUBTRACT", offset),
            LoxBytecodeOpcode::Multiply => simple_instruction("OP_MULTIPLY", offset),
            LoxBytecodeOpcode::Divide => simple_instruction("OP_DIVIDE", offset),
            LoxBytecodeOpcode::Negate => simple_instruction("OP_NEGATE", offset),
            LoxBytecodeOpcode::Return => simple_instruction("OP_RETURN", offset),
            _ => {
                print!("Unknown opcode {:?}", instruction);
                offset + 1
            }
        }
    } else {
        offset + 1
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{}", name);
    offset + 1
}

fn constant_instruction(name: &str, chunk: &LoxBytecodeChunk, offset: usize) -> usize {
    let constant_index = chunk
        .get_instruction(offset + 1)
        .unwrap()
        .as_value()
        .unwrap();
    print!("{} {:?}", name, constant_index); // TODO: check formatting
    print_value(chunk.get_constant(*constant_index).unwrap());
    println!();
    offset + 2
}

pub fn print_value(value: &LoxValueNumber) {
    print!("{}", value); // TODO: check equivalent to C-printf formatting "%g"
}
