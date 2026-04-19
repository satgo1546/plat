use clap::Parser;
use koopa::{
    back::KoopaGenerator,
    ir::{
        self, BasicBlock, FunctionData, Value,
        builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder},
    },
};
use lalrpop_util::lalrpop_mod;
use std::{collections::HashMap, io::Write, path::PathBuf};

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

fn push_inst(func_data: &mut FunctionData, bb: BasicBlock, inst: Value) {
    func_data
        .layout_mut()
        .bb_mut(bb)
        .insts_mut()
        .push_key_back(inst)
        .unwrap();
}

fn evaluate_expression(scope: &HashMap<String, ScopeItem>, expression: &ast::Expression) -> i32 {
    match expression {
        ast::Expression::Variable(name) => match scope[name] {
            ScopeItem::Constant(value) => value,
            ScopeItem::Variable(_) => panic!("`{}` is a variable but used in const", name),
        },
        ast::Expression::Number(x) => *x,
        ast::Expression::Unary(operator, expression) => {
            let x = evaluate_expression(scope, expression);
            match operator {
                ast::UnaryOperator::Plus => x,
                ast::UnaryOperator::Minus => -x,
                ast::UnaryOperator::BooleanNot => (x == 0).into(),
                ast::UnaryOperator::BitNot => !x,
            }
        }
        ast::Expression::Binary(a, operator, b) => {
            let a = evaluate_expression(scope, a);
            let b = evaluate_expression(scope, b);
            match operator {
                ast::BinaryOperator::Plus => a + b,
                ast::BinaryOperator::Minus => a - b,
                ast::BinaryOperator::Multiply => a * b,
                ast::BinaryOperator::Divide => a / b,
                ast::BinaryOperator::Modulo => a % b,
                ast::BinaryOperator::Less => (a < b).into(),
                ast::BinaryOperator::LessEqual => (a <= b).into(),
                ast::BinaryOperator::Greater => (a > b).into(),
                ast::BinaryOperator::GreaterEqual => (a >= b).into(),
                ast::BinaryOperator::Equal => (a == b).into(),
                ast::BinaryOperator::NotEqual => (a != b).into(),
                ast::BinaryOperator::BooleanAnd => (a != 0 && b != 0).into(),
                ast::BinaryOperator::BooleanOr => (a != 0 || b != 0).into(),
            }
        }
    }
}

fn lower_expression(
    func_data: &mut FunctionData,
    bb: &mut BasicBlock,
    scope: &HashMap<String, ScopeItem>,
    expression: &ast::Expression,
) -> Value {
    match expression {
        ast::Expression::Variable(name) => match scope[name] {
            ScopeItem::Constant(value) => func_data.dfg_mut().new_value().integer(value),
            ScopeItem::Variable(value) => {
                let value = func_data.dfg_mut().new_value().load(value);
                push_inst(func_data, *bb, value);
                value
            }
        },
        ast::Expression::Number(x) => func_data.dfg_mut().new_value().integer(*x),
        ast::Expression::Unary(operator, expression) => match operator {
            ast::UnaryOperator::Plus => lower_expression(func_data, bb, scope, expression),
            ast::UnaryOperator::Minus => {
                let zero = func_data.dfg_mut().new_value().integer(0);
                let x = lower_expression(func_data, bb, scope, expression);
                let value = func_data
                    .dfg_mut()
                    .new_value()
                    .binary(ir::BinaryOp::Sub, zero, x);
                push_inst(func_data, *bb, value);
                value
            }
            ast::UnaryOperator::BooleanNot => {
                let zero = func_data.dfg_mut().new_value().integer(0);
                let x = lower_expression(func_data, bb, scope, expression);
                let value = func_data
                    .dfg_mut()
                    .new_value()
                    .binary(ir::BinaryOp::Eq, zero, x);
                push_inst(func_data, *bb, value);
                value
            }
            ast::UnaryOperator::BitNot => {
                let minus1 = func_data.dfg_mut().new_value().integer(-1);
                let x = lower_expression(func_data, bb, scope, expression);
                let value = func_data
                    .dfg_mut()
                    .new_value()
                    .binary(ir::BinaryOp::Xor, minus1, x);
                push_inst(func_data, *bb, value);
                value
            }
        },
        ast::Expression::Binary(a, operator, b) => {
            let mut a = lower_expression(func_data, bb, scope, a);
            let mut b = lower_expression(func_data, bb, scope, b);
            if let ast::BinaryOperator::BooleanAnd | ast::BinaryOperator::BooleanOr = operator {
                let zero = func_data.dfg_mut().new_value().integer(0);
                a = func_data
                    .dfg_mut()
                    .new_value()
                    .binary(ir::BinaryOp::NotEq, zero, a);
                b = func_data
                    .dfg_mut()
                    .new_value()
                    .binary(ir::BinaryOp::NotEq, zero, b);
                push_inst(func_data, *bb, a);
                push_inst(func_data, *bb, b);
            }
            let operator = match operator {
                ast::BinaryOperator::Plus => ir::BinaryOp::Add,
                ast::BinaryOperator::Minus => ir::BinaryOp::Sub,
                ast::BinaryOperator::Multiply => ir::BinaryOp::Mul,
                ast::BinaryOperator::Divide => ir::BinaryOp::Div,
                ast::BinaryOperator::Modulo => ir::BinaryOp::Mod,
                ast::BinaryOperator::Less => ir::BinaryOp::Lt,
                ast::BinaryOperator::LessEqual => ir::BinaryOp::Le,
                ast::BinaryOperator::Greater => ir::BinaryOp::Gt,
                ast::BinaryOperator::GreaterEqual => ir::BinaryOp::Ge,
                ast::BinaryOperator::Equal => ir::BinaryOp::Eq,
                ast::BinaryOperator::NotEqual => ir::BinaryOp::NotEq,
                ast::BinaryOperator::BooleanAnd => ir::BinaryOp::And,
                ast::BinaryOperator::BooleanOr => ir::BinaryOp::Or,
            };
            let value = func_data.dfg_mut().new_value().binary(operator, a, b);
            push_inst(func_data, *bb, value);
            value
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ScopeItem {
    Constant(i32),
    Variable(koopa::ir::Value),
}

fn lower_statement(
    func_data: &mut FunctionData,
    bb: &mut BasicBlock,
    scope: &mut HashMap<String, ScopeItem>,
    statement: &ast::Statement,
) -> () {
    match statement {
        ast::Statement::Constant {
            constant_type: ast::BasicType {},
            name,
            value,
        } => {
            scope.insert(
                name.clone(),
                ScopeItem::Constant(evaluate_expression(&scope, &value)),
            );
        }
        ast::Statement::Variable {
            variable_type: ast::BasicType {},
            name,
            value,
        } => {
            let alloc = func_data
                .dfg_mut()
                .new_value()
                .alloc(koopa::ir::Type::get_i32());
            push_inst(func_data, *bb, alloc);
            if let Some(value) = value {
                let value = lower_expression(func_data, bb, &scope, &value);
                let store = func_data.dfg_mut().new_value().store(value, alloc);
                push_inst(func_data, *bb, store);
            }
            scope.insert(name.clone(), ScopeItem::Variable(alloc));
        }
        ast::Statement::Assign { target, value } => {
            let value = lower_expression(func_data, bb, &scope, &value);
            match &**target {
                ast::Expression::Variable(name) => match scope[name] {
                    ScopeItem::Constant(_) => panic!("assign to constant"),
                    ScopeItem::Variable(alloc) => {
                        let store = func_data.dfg_mut().new_value().store(value, alloc);
                        push_inst(func_data, *bb, store);
                    }
                },
                _ => panic!("invalid lvalue"),
            }
        }
        ast::Statement::Expression(expression) => {
            lower_expression(func_data, bb, scope, expression);
        }
        ast::Statement::Block(statements) => {
            let mut scope = scope.clone();
            for statement in statements {
                lower_statement(func_data, bb, &mut scope, statement);
            }
        }
        ast::Statement::Return(expression) => {
            let ret_value = lower_expression(func_data, bb, &scope, &expression);
            let ret = func_data.dfg_mut().new_value().ret(Some(ret_value));
            push_inst(func_data, *bb, ret);
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
    let mut bb = main_data
        .dfg_mut()
        .new_bb()
        .basic_block(Some("%entry".into()));
    main_data.layout_mut().bbs_mut().push_key_back(bb).unwrap();
    let mut scope = HashMap::new();
    for statement in &ast.function_definition.body {
        lower_statement(main_data, &mut bb, &mut scope, statement);
    }

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
