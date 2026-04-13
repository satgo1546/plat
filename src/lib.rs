use clap::Parser;
use lalrpop_util::lalrpop_mod;
use std::path::PathBuf;

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
    let output = match args.mode {
        Mode::Koopa => format!("{:#?}", ast),
        Mode::RISCV => todo!(),
    };
    match args.output {
        Some(output_path) => std::fs::write(output_path, output)?,
        None => print!("{}", output),
    }
    Ok(())
}
