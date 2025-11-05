use std::io::{self, Write};
use std::process::ExitCode;

use lox_rs::{InterpretError, VM};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let mut vm = VM::new();
    match args.len() {
        1 => {
            loop {
                print!("> ");
                io::stdout().flush().unwrap();
                let mut line = String::new();
                if io::stdin().read_line(&mut line).unwrap() == 0 {
                    break;
                }
                let _ = vm.interpret(&line);
            }
            ExitCode::SUCCESS
        }
        2 => {
            let path = &args[1];
            let source = match std::fs::read_to_string(path) {
                Ok(source) => source,
                Err(_) => {
                    eprintln!("Could not open file \"{}\".", path);
                    return ExitCode::from(74);
                }
            };
            match vm.interpret(&source) {
                Ok(()) => ExitCode::SUCCESS,
                Err(InterpretError::CompileError) => ExitCode::from(65),
                Err(InterpretError::RuntimeError) => ExitCode::from(70),
            }
        }
        _ => {
            eprintln!("Usage: lox-rs [script.lox]");
            ExitCode::from(64)
        }
    }
}
