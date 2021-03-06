use crate::{bytecode::LoxBytecodeOpcode, printer::LoxPrintable};

use super::{values::LoxBytecodeValue, LoxBytecodeChunk};

pub fn disassemble_chunk(chunk: &LoxBytecodeChunk, name: &str) {
    println!("== {} ==", name);
    let mut offset = 0;
    while offset < chunk.get_size() {
        offset = disassemble_instruction(chunk, offset);
    }
}

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
            LoxBytecodeOpcode::Nil => simple_instruction("OP_NIL", offset),
            LoxBytecodeOpcode::True => simple_instruction("OP_TRUE", offset),
            LoxBytecodeOpcode::False => simple_instruction("OP_FALSE", offset),
            LoxBytecodeOpcode::Equal => simple_instruction("OP_EQUAL", offset),
            LoxBytecodeOpcode::Greater => simple_instruction("OP_GREATER", offset),
            LoxBytecodeOpcode::Less => simple_instruction("OP_LESS", offset),
            LoxBytecodeOpcode::Add => simple_instruction("OP_ADD", offset),
            LoxBytecodeOpcode::Subtract => simple_instruction("OP_SUBTRACT", offset),
            LoxBytecodeOpcode::Multiply => simple_instruction("OP_MULTIPLY", offset),
            LoxBytecodeOpcode::Divide => simple_instruction("OP_DIVIDE", offset),
            LoxBytecodeOpcode::Not => simple_instruction("OP_NOT", offset),
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

pub fn print_value(value: &LoxBytecodeValue) {
    print!("{}", value.representation()); // TODO: check equivalent to C-printf formatting "%g"
}
