use std::io::Write;

pub fn emit_program<W: Write>(f: &mut W, program: &koopa::ir::Program) -> std::io::Result<()> {
    writeln!(f, ".text\n.globl main")?;
    for &func in program.func_layout() {
        emit_function(f, program.func(func))?;
    }
    Ok(())
}

fn emit_function<W: Write>(f: &mut W, func_data: &koopa::ir::FunctionData) -> std::io::Result<()> {
    writeln!(f, "{}:", &func_data.name()[1..])?;
    for (i, (_, node)) in func_data.layout().bbs().iter().enumerate() {
        writeln!(f, "{}__{}:", &func_data.name()[1..], i)?;
        for &inst in node.insts().keys() {
            emit_value(f, func_data.dfg(), inst)?;
        }
    }
    Ok(())
}

fn emit_value<W: Write>(
    f: &mut W,
    dfg: &koopa::ir::dfg::DataFlowGraph,
    value: koopa::ir::Value,
) -> std::io::Result<()> {
    match dfg.value(value).kind() {
        koopa::ir::ValueKind::Integer(integer) => {
            writeln!(f, "li t1, {}", integer.value())
        }
        koopa::ir::ValueKind::ZeroInit(_) => todo!(),
        koopa::ir::ValueKind::Undef(_) => todo!(),
        koopa::ir::ValueKind::Aggregate(_) => todo!(),
        koopa::ir::ValueKind::FuncArgRef(_) => todo!(),
        koopa::ir::ValueKind::BlockArgRef(_) => todo!(),
        koopa::ir::ValueKind::Alloc(_) => todo!(),
        koopa::ir::ValueKind::GlobalAlloc(_) => todo!(),
        koopa::ir::ValueKind::Load(_) => todo!(),
        koopa::ir::ValueKind::Store(_) => todo!(),
        koopa::ir::ValueKind::GetPtr(_) => todo!(),
        koopa::ir::ValueKind::GetElemPtr(_) => todo!(),
        koopa::ir::ValueKind::Binary(_) => todo!(),
        koopa::ir::ValueKind::Branch(_) => todo!(),
        koopa::ir::ValueKind::Jump(_) => todo!(),
        koopa::ir::ValueKind::Call(_) => todo!(),
        koopa::ir::ValueKind::Return(ret) => {
            if let Some(value) = ret.value() {
                emit_value(f, dfg, value)?;
                writeln!(f, "mv a0, t1")?;
            }
            writeln!(f, "ret")
        }
    }
}
