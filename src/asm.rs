use std::{collections::HashMap, io::Write};

pub fn emit_program<W: Write>(f: &mut W, program: &koopa::ir::Program) -> std::io::Result<()> {
    writeln!(f, ".text\n.globl main")?;
    for &func in program.func_layout() {
        emit_function(f, program.func(func))?;
    }
    Ok(())
}

#[derive(Debug, Default)]
struct StackFrame {
    map: HashMap<koopa::ir::Value, i32>,
    offset: i32,
}

impl StackFrame {
    fn load<W: Write>(
        &self,
        f: &mut W,
        dfg: &koopa::ir::dfg::DataFlowGraph,
        register: &str,
        value: koopa::ir::Value,
    ) -> std::io::Result<()> {
        if let koopa::ir::ValueKind::Integer(integer) = dfg.value(value).kind() {
            writeln!(f, "li {}, {}", register, integer.value())
        } else {
            writeln!(f, "lw {}, {}(sp)", register, self.map[&value] + self.offset)
        }
    }

    fn store<W: Write>(
        &self,
        f: &mut W,
        register: &str,
        value: koopa::ir::Value,
    ) -> std::io::Result<()> {
        writeln!(f, "sw {}, {}(sp)", register, self.map[&value] + self.offset)
    }
}

fn emit_function<W: Write>(f: &mut W, func_data: &koopa::ir::FunctionData) -> std::io::Result<()> {
    writeln!(f, "{}:", &func_data.name()[1..])?;
    for (i, (_, node)) in func_data.layout().bbs().iter().enumerate() {
        writeln!(f, "{}__{}:", &func_data.name()[1..], i)?;
        let mut stack_frame = StackFrame::default();
        for &inst in node.insts().keys() {
            if !func_data.dfg().value(inst).ty().is_unit() {
                stack_frame.offset += 4;
                stack_frame.map.insert(inst, -stack_frame.offset);
            }
        }
        writeln!(f, "addi sp, sp, {}", -stack_frame.offset)?;
        for &inst in node.insts().keys() {
            emit_value(f, func_data.dfg(), &stack_frame, inst)?;
        }
    }
    Ok(())
}

fn emit_value<W: Write>(
    f: &mut W,
    dfg: &koopa::ir::dfg::DataFlowGraph,
    stack_frame: &StackFrame,
    value: koopa::ir::Value,
) -> std::io::Result<()> {
    match dfg.value(value).kind() {
        koopa::ir::ValueKind::Integer(_) => panic!("how did you do that?"),
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
        koopa::ir::ValueKind::Binary(binary) => {
            stack_frame.load(f, dfg, "t2", binary.lhs())?;
            stack_frame.load(f, dfg, "t1", binary.rhs())?;
            match binary.op() {
                koopa::ir::BinaryOp::NotEq => todo!(),
                koopa::ir::BinaryOp::Eq => {
                    writeln!(f, "xor t1, t2, t1\nseqz t1, t1")?;
                }
                koopa::ir::BinaryOp::Gt => todo!(),
                koopa::ir::BinaryOp::Lt => todo!(),
                koopa::ir::BinaryOp::Ge => todo!(),
                koopa::ir::BinaryOp::Le => todo!(),
                koopa::ir::BinaryOp::Add => todo!(),
                koopa::ir::BinaryOp::Sub => {
                    writeln!(f, "sub t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Mul => todo!(),
                koopa::ir::BinaryOp::Div => todo!(),
                koopa::ir::BinaryOp::Mod => todo!(),
                koopa::ir::BinaryOp::And => todo!(),
                koopa::ir::BinaryOp::Or => todo!(),
                koopa::ir::BinaryOp::Xor => {
                    writeln!(f, "xor t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Shl => todo!(),
                koopa::ir::BinaryOp::Shr => todo!(),
                koopa::ir::BinaryOp::Sar => todo!(),
            }
            stack_frame.store(f, "t1", value)
        }
        koopa::ir::ValueKind::Branch(_) => todo!(),
        koopa::ir::ValueKind::Jump(_) => todo!(),
        koopa::ir::ValueKind::Call(_) => todo!(),
        koopa::ir::ValueKind::Return(ret) => {
            if let Some(value) = ret.value() {
                stack_frame.load(f, dfg, "a0", value)?;
            }
            writeln!(f, "addi sp, sp, {}\nret", stack_frame.offset)
        }
    }
}
