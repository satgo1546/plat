use lox_rs::{Chunk, Instruction, InterpretResult, VM, Value};

fn main() -> InterpretResult {
    println!("Hello, world!");
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value::Number(1.2));
    chunk.write(Instruction::Constant(constant), 1);
    let constant = chunk.add_constant(Value::Number(3.4));
    chunk.write(Instruction::Constant(constant), 1);
    chunk.write(Instruction::Add, 1);
    let constant = chunk.add_constant(Value::Number(5.6));
    chunk.write(Instruction::Constant(constant), 1);
    chunk.write(Instruction::Divide, 1);
    chunk.write(Instruction::Negate, 1);
    chunk.write(Instruction::Return, 1);
    println!("{:?}", chunk);
    let mut vm = VM::new();
    vm.interpret(&chunk)
}
