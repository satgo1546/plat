use std::fmt::{Debug, Write};

#[derive(Debug, Clone)]
pub enum Instruction {
    Constant(u8),
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
}

pub struct Chunk {
    code: Vec<Instruction>,
    lines: Vec<i32>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn write(&mut self, instruction: Instruction, line: i32) {
        self.code.push(instruction);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, instruction) in self.code.iter().enumerate() {
            if i > 0 {
                f.write_char('\n')?;
            }
            write!(f, "{:04} ", i)?;
            if i > 0 && self.lines[i - 1] == self.lines[i] {
                f.write_str("   | ")?;
            } else {
                write!(f, "{:4} ", self.lines[i])?;
            }
            write!(f, "{:?}", instruction)?;
        }
        Ok(())
    }
}

pub struct VM {}

#[derive(Debug)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
}
pub type InterpretResult = Result<(), InterpretError>;

impl VM {
    pub fn new() -> VM {
        VM {}
    }

    fn binary_op(stack: &mut Vec<Value>, op: fn(f64, f64) -> f64) -> InterpretResult {
        let b = stack.pop().unwrap();
        let a = stack.pop().unwrap();
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                stack.push(Value::Number(op(a, b)));
                Ok(())
            }
            _ => Err(InterpretError::RuntimeError),
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> InterpretResult {
        let mut ip = 0;
        let mut stack = Vec::<Value>::new();
        loop {
            match chunk.code[ip] {
                Instruction::Constant(constant) => {
                    let constant = chunk.constants[constant as usize].clone();
                    println!("{:?}", constant);
                    stack.push(constant);
                }
                Instruction::Add => {
                    Self::binary_op(&mut stack, std::ops::Add::add)?;
                }
                Instruction::Subtract => {
                    Self::binary_op(&mut stack, std::ops::Sub::sub)?;
                }
                Instruction::Multiply => {
                    Self::binary_op(&mut stack, std::ops::Mul::mul)?;
                }
                Instruction::Divide => {
                    Self::binary_op(&mut stack, std::ops::Div::div)?;
                }
                Instruction::Negate => {
                    let value = stack.last_mut().unwrap();
                    match value {
                        Value::Number(x) => *x = -*x,
                    }
                }
                Instruction::Return => {
                    let value = stack.pop().unwrap();
                    println!("ret {:?}", value);
                    return Ok(());
                }
            }
            ip += 1;
        }
    }
}
