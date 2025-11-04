use lox_rs::{Chunk, Instruction, Value};

fn main() {
    println!("Hello, world!");
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value::Number(1.2));
    chunk.write(Instruction::Constant(constant), 1);
    chunk.write(Instruction::Return, 1);
    println!("{:?}", chunk);
}
