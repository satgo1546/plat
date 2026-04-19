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
    let mut stack_frame = StackFrame::default();
    for (_, node) in func_data.layout().bbs() {
        for &inst in node.insts().keys() {
            if !func_data.dfg().value(inst).ty().is_unit() {
                stack_frame.offset += 4;
                stack_frame.map.insert(inst, -stack_frame.offset);
            }
        }
    }
    writeln!(f, "{}:", &func_data.name()[1..])?;
    writeln!(f, "addi sp, sp, {}", -stack_frame.offset)?;
    for (&bb, node) in func_data.layout().bbs() {
        writeln!(
            f,
            "{}:",
            &func_data.dfg().bb(bb).name().as_ref().unwrap()[1..]
        )?;
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
        koopa::ir::ValueKind::Alloc(_) => Ok(()),
        koopa::ir::ValueKind::GlobalAlloc(_) => todo!(),
        koopa::ir::ValueKind::Load(load) => {
            stack_frame.load(f, dfg, "t1", load.src())?;
            stack_frame.store(f, "t1", value)
        }
        koopa::ir::ValueKind::Store(store) => {
            stack_frame.load(f, dfg, "t1", store.value())?;
            stack_frame.store(f, "t1", store.dest())
        }
        koopa::ir::ValueKind::GetPtr(_) => todo!(),
        koopa::ir::ValueKind::GetElemPtr(_) => todo!(),
        koopa::ir::ValueKind::Binary(binary) => {
            stack_frame.load(f, dfg, "t2", binary.lhs())?;
            stack_frame.load(f, dfg, "t1", binary.rhs())?;
            match binary.op() {
                koopa::ir::BinaryOp::NotEq => {
                    writeln!(f, "xor t1, t2, t1\nsnez t1, t1")?;
                }
                koopa::ir::BinaryOp::Eq => {
                    writeln!(f, "xor t1, t2, t1\nseqz t1, t1")?;
                }
                koopa::ir::BinaryOp::Gt => {
                    writeln!(f, "sgt t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Lt => {
                    writeln!(f, "slt t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Ge => {
                    writeln!(f, "slt t1, t2, t1\nseqz t1, t1")?;
                }
                koopa::ir::BinaryOp::Le => {
                    writeln!(f, "sgt t1, t2, t1\nseqz t1, t1")?;
                }
                koopa::ir::BinaryOp::Add => {
                    writeln!(f, "add t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Sub => {
                    writeln!(f, "sub t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Mul => {
                    writeln!(f, "mul t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Div => {
                    writeln!(f, "div t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Mod => {
                    writeln!(f, "rem t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::And => {
                    writeln!(f, "and t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Or => {
                    writeln!(f, "or t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Xor => {
                    writeln!(f, "xor t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Shl => {
                    writeln!(f, "sll t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Shr => {
                    writeln!(f, "srl t1, t2, t1")?;
                }
                koopa::ir::BinaryOp::Sar => {
                    writeln!(f, "sra t1, t2, t1")?;
                }
            }
            stack_frame.store(f, "t1", value)
        }
        koopa::ir::ValueKind::Branch(branch) => {
            stack_frame.load(f, dfg, "t1", branch.cond())?;
            writeln!(
                f,
                "bnez t1, {}\nj {}",
                &dfg.bb(branch.true_bb()).name().as_ref().unwrap()[1..],
                &dfg.bb(branch.false_bb()).name().as_ref().unwrap()[1..],
            )
        }
        koopa::ir::ValueKind::Jump(jump) => {
            writeln!(
                f,
                "j {}",
                &dfg.bb(jump.target()).name().as_ref().unwrap()[1..]
            )
        }
        koopa::ir::ValueKind::Call(_) => todo!(),
        koopa::ir::ValueKind::Return(ret) => {
            if let Some(value) = ret.value() {
                stack_frame.load(f, dfg, "a0", value)?;
            }
            writeln!(f, "addi sp, sp, {}\nret", stack_frame.offset)
        }
    }
}
