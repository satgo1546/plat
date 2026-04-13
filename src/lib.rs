use clap::Parser;
use koopa::{
    back::KoopaGenerator,
    ir::{
        self,
        builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder},
    },
};
use lalrpop_util::lalrpop_mod;
use std::{io::Write, path::PathBuf};

lalrpop_mod!(grammar);
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
    let forty_two = main_data
        .dfg_mut()
        .new_value()
        .integer(ast.function_definition.body.statements[0].value);
    let ret = main_data.dfg_mut().new_value().ret(Some(forty_two));
    main_data
        .layout_mut()
        .bb_mut(entry)
        .insts_mut()
        .extend([ret]);

    let mut output = Vec::<u8>::new();
    match args.mode {
        Mode::Koopa => KoopaGenerator::new(&mut output).generate_on(&program)?,
        Mode::RISCV => todo!(),
    }
    match args.output {
        Some(output_path) => std::fs::write(output_path, output)?,
        None => std::io::stdout().write_all(&output)?,
    }
    Ok(())
}
