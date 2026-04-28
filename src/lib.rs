use clap::Parser;
use koopa::{
    back::KoopaGenerator,
    ir::{
        self, BasicBlock, FunctionData, Value,
        builder::{
            BasicBlockBuilder, EntityInfoQuerier, GlobalInstBuilder, LocalInstBuilder, ValueBuilder,
        },
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

fn new_bb(func_data: &mut FunctionData, params: usize) -> BasicBlock {
    let name = format!(
        "%{}__bb{}",
        &func_data.name()[1..],
        func_data.dfg().bbs().len()
    );
    let bb = func_data
        .dfg_mut()
        .new_bb()
        .basic_block_with_params(Some(name), vec![koopa::ir::Type::get_i32(); params]);
    func_data.layout_mut().bbs_mut().push_key_back(bb).unwrap();
    bb
}

fn push_inst(func_data: &mut FunctionData, bb: BasicBlock, inst: Value) {
    if let Some(&last_value) = func_data.layout_mut().bb_mut(bb).insts().back_key()
        && let ir::ValueKind::Branch(_) | ir::ValueKind::Jump(_) | ir::ValueKind::Return(_) =
            func_data.dfg().value(last_value).kind()
    {
        return;
    }
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
            _ => panic!("`{}` is not a constant but used in const", name),
        },
        ast::Expression::Element(_, _) => panic!("cannot manipulate array in constant"),
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
        ast::Expression::Call { .. } => panic!("cannot call functions in const"),
    }
}

fn lower_type(scope: &HashMap<String, ScopeItem>, ast_type: &ast::Type) -> (Vec<usize>, ir::Type) {
    let dimensions: Vec<_> = ast_type
        .dimensions
        .iter()
        .map(|expression| evaluate_expression(scope, expression) as usize)
        .collect();
    let mut ty = ir::Type::get_i32();
    for &dimension in dimensions.iter().rev() {
        if dimension == 0 {
            ty = ir::Type::get_pointer(ty);
        } else {
            ty = ir::Type::get_array(ty, dimension);
        }
    }
    (dimensions, ty)
}

fn pad_initializer(
    item: ast::InitializerListItem,
    dimensions: &[usize],
) -> ast::InitializerListItem {
    match (item, dimensions) {
        (item @ ast::InitializerListItem::Value(_), []) => item,
        (ast::InitializerListItem::Value(_), _) => panic!("array initialized with scalar"),
        (ast::InitializerListItem::List(list), [_, ..]) => {
            let mut result = flat_pad_initializer(list, dimensions);
            for &d in dimensions[1..].iter().rev() {
                result = result
                    .chunks_exact(d)
                    .map(|chunk| ast::InitializerListItem::List(chunk.to_vec()))
                    .collect();
            }
            ast::InitializerListItem::List(result)
        }
        (ast::InitializerListItem::List(_), []) => panic!("scalar initialized with list"),
    }
}

fn flat_pad_initializer(
    items: Vec<ast::InitializerListItem>,
    dimensions: &[usize],
) -> Vec<ast::InitializerListItem> {
    let mut result = Vec::new();
    let target_size = dimensions.iter().product();
    for item in items {
        match item {
            ast::InitializerListItem::Value(_) => result.push(item),
            ast::InitializerListItem::List(list) => {
                let mut s = result.len();
                let mut i = 1;
                for (j, &len) in dimensions.iter().enumerate().rev() {
                    if s % len == 0 {
                        s /= len;
                    } else {
                        i = j + 1;
                        break;
                    }
                }
                if i >= dimensions.len() {
                    panic!("invalid initializer")
                }
                result.extend(flat_pad_initializer(list, &dimensions[i..]));
            }
        }
        if result.len() > target_size {
            panic!("too many elements")
        }
    }
    while result.len() < target_size {
        result.push(ast::InitializerListItem::Value(Box::new(
            ast::Expression::Number(0),
        )));
    }
    result
}

fn lower_global_initializer(
    program: &mut ir::Program,
    scope: &mut HashMap<String, ScopeItem>,
    item: &ast::InitializerListItem,
) -> ir::Value {
    match item {
        ast::InitializerListItem::Value(expression) => program
            .new_value()
            .integer(evaluate_expression(scope, expression)),
        ast::InitializerListItem::List(items) => {
            let values = items
                .iter()
                .map(|item| lower_global_initializer(program, scope, item))
                .collect();
            program.new_value().aggregate(values)
        }
    }
}

fn lower_initializer(
    program: &mut ir::Program,
    func: ir::Function,
    bb: &mut BasicBlock,
    scope: &mut HashMap<String, ScopeItem>,
    item: &ast::InitializerListItem,
) -> ir::Value {
    match item {
        ast::InitializerListItem::Value(expression) => {
            lower_expression(program, func, bb, scope, expression)
        }
        ast::InitializerListItem::List(items) => {
            let values = items
                .iter()
                .map(|item| lower_initializer(program, func, bb, scope, item))
                .collect();
            let func_data = program.func_mut(func);
            func_data.dfg_mut().new_value().aggregate(values)
        }
    }
}

fn lower_lvalue(
    program: &mut ir::Program,
    func: ir::Function,
    bb: &mut BasicBlock,
    scope: &HashMap<String, ScopeItem>,
    expression: &ast::Expression,
) -> Value {
    match expression {
        ast::Expression::Variable(name) => {
            let ScopeItem::Variable(value) = scope[name] else {
                panic!("`{}` is not a variable", name)
            };
            value
        }
        ast::Expression::Element(array, index) => {
            let array = match &**array {
                ast::Expression::Variable(name) => {
                    let ScopeItem::Variable(value) = scope[name] else {
                        panic!("`{}` is not an array", name)
                    };
                    value
                }
                _ => lower_lvalue(program, func, bb, scope, array),
            };
            let index = lower_expression(program, func, bb, scope, index);
            let func_data = program.func_mut(func);
            let ptr = func_data.dfg_mut().new_value().get_elem_ptr(array, index);
            push_inst(func_data, *bb, ptr);
            ptr
        }
        _ => panic!("invalid lvalue"),
    }
}

fn lower_expression(
    program: &mut ir::Program,
    func: ir::Function,
    bb: &mut BasicBlock,
    scope: &HashMap<String, ScopeItem>,
    expression: &ast::Expression,
) -> Value {
    let func_data = program.func_mut(func);
    match expression {
        ast::Expression::Variable(name) => match scope[name] {
            ScopeItem::Constant(value) => func_data.dfg_mut().new_value().integer(value),
            ScopeItem::Variable(value) => {
                let value = func_data.dfg_mut().new_value().load(value);
                push_inst(func_data, *bb, value);
                value
            }
            ScopeItem::Function(_) => panic!("second-class function"),
        },
        ast::Expression::Element(_, _) => {
            let ptr = lower_lvalue(program, func, bb, scope, expression);
            let func_data = program.func_mut(func);
            let value = func_data.dfg_mut().new_value().load(ptr);
            push_inst(func_data, *bb, value);
            value
        }
        ast::Expression::Number(x) => func_data.dfg_mut().new_value().integer(*x),
        ast::Expression::Unary(operator, expression) => match operator {
            ast::UnaryOperator::Plus => lower_expression(program, func, bb, scope, expression),
            ast::UnaryOperator::Minus => {
                let zero = func_data.dfg_mut().new_value().integer(0);
                let x = lower_expression(program, func, bb, scope, expression);
                let func_data = program.func_mut(func);
                let value = func_data
                    .dfg_mut()
                    .new_value()
                    .binary(ir::BinaryOp::Sub, zero, x);
                push_inst(func_data, *bb, value);
                value
            }
            ast::UnaryOperator::BooleanNot => {
                let zero = func_data.dfg_mut().new_value().integer(0);
                let x = lower_expression(program, func, bb, scope, expression);
                let func_data = program.func_mut(func);
                let value = func_data
                    .dfg_mut()
                    .new_value()
                    .binary(ir::BinaryOp::Eq, zero, x);
                push_inst(func_data, *bb, value);
                value
            }
            ast::UnaryOperator::BitNot => {
                let minus1 = func_data.dfg_mut().new_value().integer(-1);
                let x = lower_expression(program, func, bb, scope, expression);
                let func_data = program.func_mut(func);
                let value = func_data
                    .dfg_mut()
                    .new_value()
                    .binary(ir::BinaryOp::Xor, minus1, x);
                push_inst(func_data, *bb, value);
                value
            }
        },
        ast::Expression::Binary(a, operator, b) => {
            let a = lower_expression(program, func, bb, scope, a);
            let func_data = program.func_mut(func);
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
                ast::BinaryOperator::BooleanAnd => {
                    let then_bb = new_bb(func_data, 0);
                    let else_bb = new_bb(func_data, 0);
                    let end_bb = new_bb(func_data, 1);
                    let branch = func_data.dfg_mut().new_value().branch(a, then_bb, else_bb);
                    push_inst(func_data, *bb, branch);

                    *bb = then_bb;
                    let zero = func_data.dfg_mut().new_value().integer(0);
                    let b = lower_expression(program, func, bb, scope, b);
                    let func_data = program.func_mut(func);
                    let b = func_data
                        .dfg_mut()
                        .new_value()
                        .binary(ir::BinaryOp::NotEq, zero, b);
                    push_inst(func_data, *bb, b);
                    let jump = func_data
                        .dfg_mut()
                        .new_value()
                        .jump_with_args(end_bb, vec![b]);
                    push_inst(func_data, *bb, jump);

                    *bb = else_bb;
                    let jump = func_data
                        .dfg_mut()
                        .new_value()
                        .jump_with_args(end_bb, vec![zero]);
                    push_inst(func_data, *bb, jump);

                    *bb = end_bb;
                    return func_data.dfg_mut().new_value().bb_params(*bb)[0];
                }
                ast::BinaryOperator::BooleanOr => {
                    let then_bb = new_bb(func_data, 0);
                    let else_bb = new_bb(func_data, 0);
                    let end_bb = new_bb(func_data, 1);
                    let branch = func_data.dfg_mut().new_value().branch(a, then_bb, else_bb);
                    push_inst(func_data, *bb, branch);

                    *bb = then_bb;
                    let one = func_data.dfg_mut().new_value().integer(1);
                    let jump = func_data
                        .dfg_mut()
                        .new_value()
                        .jump_with_args(end_bb, vec![one]);
                    push_inst(func_data, *bb, jump);

                    *bb = else_bb;
                    let zero = func_data.dfg_mut().new_value().integer(0);
                    let b = lower_expression(program, func, bb, scope, b);
                    let func_data = program.func_mut(func);
                    let b = func_data
                        .dfg_mut()
                        .new_value()
                        .binary(ir::BinaryOp::NotEq, zero, b);
                    push_inst(func_data, *bb, b);
                    let jump = func_data
                        .dfg_mut()
                        .new_value()
                        .jump_with_args(end_bb, vec![b]);
                    push_inst(func_data, *bb, jump);

                    *bb = end_bb;
                    return func_data.dfg_mut().new_value().bb_params(*bb)[0];
                }
            };
            let b = lower_expression(program, func, bb, scope, b);
            let func_data = program.func_mut(func);
            let value = func_data.dfg_mut().new_value().binary(operator, a, b);
            push_inst(func_data, *bb, value);
            value
        }
        ast::Expression::Call {
            function_name,
            arguments,
        } => match scope[function_name] {
            ScopeItem::Function(callee) => {
                let args = arguments
                    .iter()
                    .map(|arg| lower_expression(program, func, bb, scope, arg))
                    .collect();
                let func_data = program.func_mut(func);
                let value = func_data.dfg_mut().new_value().call(callee, args);
                push_inst(func_data, *bb, value);
                value
            }
            _ => panic!("call non-function"),
        },
    }
}

#[derive(Debug, Clone, Copy)]
enum ScopeItem {
    Constant(i32),
    Variable(koopa::ir::Value),
    Function(koopa::ir::Function),
}

struct LoopInfo {
    entry: BasicBlock,
    end: BasicBlock,
}

fn lower_statement(
    program: &mut ir::Program,
    func: ir::Function,
    bb: &mut BasicBlock,
    scope: &mut HashMap<String, ScopeItem>,
    loop_info: Option<&LoopInfo>,
    statement: &ast::Statement,
) -> () {
    let func_data = program.func_mut(func);
    match statement {
        ast::Statement::Declaration(ast::Declaration::Constant { name, value }) => {
            scope.insert(
                name.clone(),
                ScopeItem::Constant(evaluate_expression(&scope, &value)),
            );
        }
        ast::Statement::Declaration(ast::Declaration::Variable {
            variable_type,
            name,
            initial_value,
        }) => {
            let (dimensions, ty) = lower_type(&scope, variable_type);
            let alloc = func_data.dfg_mut().new_value().alloc(ty);
            push_inst(func_data, *bb, alloc);
            if let Some(value) = initial_value {
                let value = pad_initializer(value.clone(), &dimensions);
                let value = lower_initializer(program, func, bb, scope, &value);
                let func_data = program.func_mut(func);
                let store = func_data.dfg_mut().new_value().store(value, alloc);
                push_inst(func_data, *bb, store);
            }
            scope.insert(name.clone(), ScopeItem::Variable(alloc));
        }
        ast::Statement::Assign { target, value } => {
            let ptr = lower_lvalue(program, func, bb, &scope, target);
            let value = lower_expression(program, func, bb, &scope, &value);
            let func_data = program.func_mut(func);
            let store = func_data.dfg_mut().new_value().store(value, ptr);
            push_inst(func_data, *bb, store);
        }
        ast::Statement::Expression(expression) => {
            lower_expression(program, func, bb, scope, expression);
        }
        ast::Statement::Block(statements) => {
            let mut scope = scope.clone();
            for statement in statements {
                lower_statement(program, func, bb, &mut scope, loop_info, statement);
            }
        }
        ast::Statement::If {
            condition,
            then,
            otherwise,
        } => {
            let mut then_bb = new_bb(func_data, 0);
            let mut else_bb = new_bb(func_data, 0);
            let end_bb = new_bb(func_data, 0);
            let condition = lower_expression(program, func, bb, scope, condition);
            let func_data = program.func_mut(func);
            let branch = func_data
                .dfg_mut()
                .new_value()
                .branch(condition, then_bb, else_bb);
            push_inst(func_data, *bb, branch);
            lower_statement(program, func, &mut then_bb, scope, loop_info, then);
            if let Some(otherwise) = otherwise {
                lower_statement(program, func, &mut else_bb, scope, loop_info, otherwise);
            }
            let func_data = program.func_mut(func);
            if let Some(&x) = func_data
                .layout_mut()
                .bb_mut(then_bb)
                .insts_mut()
                .back_key()
            {
                func_data.dfg().value(x).ty();
            }
            let jump = func_data.dfg_mut().new_value().jump(end_bb);
            push_inst(func_data, then_bb, jump);
            push_inst(func_data, else_bb, jump);
            *bb = end_bb;
        }
        ast::Statement::While { condition, body } => {
            let condition_bb = new_bb(func_data, 0);
            let body_bb = new_bb(func_data, 0);
            let end_bb = new_bb(func_data, 0);
            let jump = func_data.dfg_mut().new_value().jump(condition_bb);
            push_inst(func_data, *bb, jump);

            *bb = condition_bb;
            let condition = lower_expression(program, func, bb, scope, condition);
            let func_data = program.func_mut(func);
            let branch = func_data
                .dfg_mut()
                .new_value()
                .branch(condition, body_bb, end_bb);
            push_inst(func_data, *bb, branch);

            *bb = body_bb;
            lower_statement(
                program,
                func,
                bb,
                scope,
                Some(&LoopInfo {
                    entry: condition_bb,
                    end: end_bb,
                }),
                body,
            );
            let func_data = program.func_mut(func);
            push_inst(func_data, *bb, jump);

            *bb = end_bb;
        }
        ast::Statement::Break => {
            let Some(loop_info) = loop_info else {
                panic!("stray break")
            };
            let jump = func_data.dfg_mut().new_value().jump(loop_info.end);
            push_inst(func_data, *bb, jump);
        }
        ast::Statement::Continue => {
            let Some(loop_info) = loop_info else {
                panic!("stray continue")
            };
            let jump = func_data.dfg_mut().new_value().jump(loop_info.entry);
            push_inst(func_data, *bb, jump);
        }
        ast::Statement::Return(expression) => {
            let ret_value = lower_expression(program, func, bb, &scope, &expression);
            let func_data = program.func_mut(func);
            let ret = func_data.dfg_mut().new_value().ret(Some(ret_value));
            push_inst(func_data, *bb, ret);
        }
    }
}

fn lower_program(ast: &ast::Program) -> ir::Program {
    let mut program = ir::Program::new();
    let mut scope = HashMap::with_capacity(ast.functions.len() + 8);
    for (name, params) in [
        ("@getint", vec![]),
        ("@getch", vec![]),
        (
            "@getarray",
            vec![ir::Type::get_pointer(ir::Type::get_i32())],
        ),
        ("@putint", vec![ir::Type::get_i32()]),
        ("@putch", vec![ir::Type::get_i32()]),
        (
            "@putarray",
            vec![
                ir::Type::get_i32(),
                ir::Type::get_pointer(ir::Type::get_i32()),
            ],
        ),
        ("@starttime", vec![]),
        ("@stoptime", vec![]),
    ] {
        let func = program.new_func_def_with_param_names(
            name.to_string(),
            params.into_iter().map(|t| (None, t)).collect(),
            ir::Type::get_i32(),
        );
        scope.insert(name[1..].to_string(), ScopeItem::Function(func));
    }
    for declaration in &ast.declarations {
        match declaration {
            ast::Declaration::Constant { name, value } => {
                scope.insert(
                    name.clone(),
                    ScopeItem::Constant(evaluate_expression(&scope, &value)),
                );
            }
            ast::Declaration::Variable {
                variable_type,
                name,
                initial_value,
            } => {
                let (dimensions, ty) = lower_type(&scope, variable_type);
                let value = match initial_value {
                    Some(value) => {
                        let value = pad_initializer(value.clone(), &dimensions);
                        lower_global_initializer(&mut program, &mut scope, &value)
                    }
                    None => program.new_value().zero_init(ty),
                };
                let alloc = program.new_value().global_alloc(value);
                scope.insert(name.clone(), ScopeItem::Variable(alloc));
            }
        }
    }
    let mut funcs = Vec::with_capacity(ast.functions.len());
    for function in &ast.functions {
        let func = program.new_func_def_with_param_names(
            format!("@{}", function.name),
            function
                .parameters
                .iter()
                .map(|p| (Some(format!("@{}", p.name)), ir::Type::get_i32()))
                .collect(),
            ir::Type::get_i32(),
        );
        scope.insert(function.name.clone(), ScopeItem::Function(func));
        funcs.push(func);
    }
    for (function, func) in ast.functions.iter().zip(funcs) {
        let func_data = program.func_mut(func);
        let mut bb = new_bb(func_data, 0);
        let mut scope = scope.clone();
        for (parameter, param) in function
            .parameters
            .iter()
            .zip(Vec::from(func_data.params()))
        // clone forced by borrow checker 😾
        {
            let (_, ty) = lower_type(&scope, &parameter.parameter_type);
            let alloc = func_data.dfg_mut().new_value().alloc(ty);
            push_inst(func_data, bb, alloc);
            let store = func_data.dfg_mut().new_value().store(param, alloc);
            push_inst(func_data, bb, store);
            scope.insert(parameter.name.clone(), ScopeItem::Variable(alloc));
        }
        for statement in &function.body {
            lower_statement(&mut program, func, &mut bb, &mut scope, None, statement);
        }
        let func_data = program.func_mut(func);
        let zero = func_data.dfg_mut().new_value().integer(0);
        let ret = func_data.dfg_mut().new_value().ret(Some(zero));
        push_inst(func_data, bb, ret);
    }
    program
}

pub fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let input = std::fs::read_to_string(args.input)?;
    let ast = grammar::ProgramParser::new().parse(&input).unwrap();
    koopa::ir::Type::set_ptr_size(4);
    let program = lower_program(&ast);
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
