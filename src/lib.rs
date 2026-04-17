use clap::Parser;
use koopa::{
    back::KoopaGenerator,
    ir::{
        self, FunctionData, Value,
        builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder},
    },
};
use lalrpop_util::lalrpop_mod;
use std::{io::Write, path::PathBuf};

lalrpop_mod!(grammar);
mod asm;
mod ast;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Mode {
    #[value(alias = "-koopa")]
    Koopa,
    #[value(alias = "-riscv")]
    RISCV,
}

#[derive(clap::Parser, Debug)]
struct Args {
    /// Output format.
    #[arg(allow_hyphen_values = true)]
    mode: Mode,
    /// Path to input file.
    input: PathBuf,
    /// Path to output file. Writes to stdout if omitted.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn lower_expression(
    insts: &mut Vec<Value>,
    func_data: &mut FunctionData,
    expression: &ast::Expression,
) -> Value {
    match expression {
        ast::Expression::Number(x) => func_data.dfg_mut().new_value().integer(*x),
        ast::Expression::Unary(operator, expression) => match operator {
            ast::UnaryOperator::Plus => lower_expression(insts, func_data, expression),
            ast::UnaryOperator::Minus => {
                let zero = func_data.dfg_mut().new_value().integer(0);
                let x = lower_expression(insts, func_data, expression);
                let value = func_data
                    .dfg_mut()
                    .new_value()
                    .binary(ir::BinaryOp::Sub, zero, x);
                insts.push(value);
                value
            }
            ast::UnaryOperator::BooleanNot => {
                let zero = func_data.dfg_mut().new_value().integer(0);
                let x = lower_expression(insts, func_data, expression);
                let value = func_data
                    .dfg_mut()
                    .new_value()
                    .binary(ir::BinaryOp::Eq, zero, x);
                insts.push(value);
                value
            }
            ast::UnaryOperator::BitNot => {
                let minus1 = func_data.dfg_mut().new_value().integer(-1);
                let x = lower_expression(insts, func_data, expression);
                let value = func_data
                    .dfg_mut()
                    .new_value()
                    .binary(ir::BinaryOp::Xor, minus1, x);
                insts.push(value);
                value
            }
        },
        ast::Expression::Binary(a, operator, b) => {
            let operator = match operator {
                ast::BinaryOperator::Plus => ir::BinaryOp::Add,
                ast::BinaryOperator::Minus => ir::BinaryOp::Sub,
                ast::BinaryOperator::Multiply => ir::BinaryOp::Mul,
                ast::BinaryOperator::Divide => ir::BinaryOp::Div,
                ast::BinaryOperator::Modulo => ir::BinaryOp::Mod,
            };
            let a = lower_expression(insts, func_data, a);
            let b = lower_expression(insts, func_data, b);
            let value = func_data.dfg_mut().new_value().binary(operator, a, b);
            insts.push(value);
            value
        }
    }
}

pub fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let input = std::fs::read_to_string(args.input)?;
    let ast = grammar::ProgramParser::new().parse(&input).unwrap();

    let mut program = ir::Program::new();
    let main = program.new_func_def_with_param_names(
        format!("@{}", ast.function_definition.name),
        vec![],
        ir::Type::get_i32(),
    );
    let main_data = program.func_mut(main);
    let entry = main_data
        .dfg_mut()
        .new_bb()
        .basic_block(Some("%entry".into()));
    main_data.layout_mut().bbs_mut().extend([entry]);
    let mut insts = vec![];
    let ret_value = lower_expression(
        &mut insts,
        main_data,
        &ast.function_definition.body.statements[0].value,
    );
    insts.push(main_data.dfg_mut().new_value().ret(Some(ret_value)));
    main_data
        .layout_mut()
        .bb_mut(entry)
        .insts_mut()
        .extend(insts);

    let mut output = Vec::<u8>::new();
    match args.mode {
        Mode::Koopa => KoopaGenerator::new(&mut output).generate_on(&program)?,
        Mode::RISCV => asm::emit_program(&mut output, &program)?,
    }
    match args.output {
        Some(output_path) => std::fs::write(output_path, output)?,
        None => std::io::stdout().write_all(&output)?,
    }
    Ok(())
}
