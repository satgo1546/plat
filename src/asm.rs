use std::{collections::HashMap, io::Write};

pub fn emit_program<W: Write>(f: &mut W, program: &koopa::ir::Program) -> std::io::Result<()> {
    let mut globals = StackFrame {
        map: HashMap::with_capacity(program.inst_layout().len()),
        offset: 0,
    };
    for &var in program.inst_layout() {
        globals.map.insert(var, globals.offset);
        let var = program.borrow_value(var);
        let koopa::ir::TypeKind::Pointer(ty) = var.ty().kind() else {
            panic!("global alloc should return a pointer")
        };
        globals.offset += ty.size();
    }
    writeln!(f, ".text\n.globl main")?;
    for &func in program.func_layout() {
        emit_function(f, program, &globals, func)?;
    }
    Ok(())
}

#[derive(Debug, Default)]
struct StackFrame {
    map: HashMap<koopa::ir::Value, usize>,
    offset: usize,
}

impl StackFrame {
    fn round(size: usize) -> usize {
        (size + 15) & !15
    }

    fn load<W: Write>(
        &self,
        f: &mut W,
        dfg: &koopa::ir::dfg::DataFlowGraph,
        register: &str,
        value: koopa::ir::Value,
    ) -> std::io::Result<()> {
        if value.is_global() {
            return writeln!(f, "lw {}, {}(fp)", register, self.map[&value]);
        }
        match dfg.value(value).kind() {
            koopa::ir::ValueKind::Integer(integer) => {
                writeln!(f, "li {}, {}", register, integer.value())
            }
            koopa::ir::ValueKind::FuncArgRef(arg) => {
                let i = arg.index();
                if i < 8 {
                    writeln!(f, "mv {}, a{}", register, i)
                } else {
                    writeln!(f, "lw {}, {}(sp)", register, self.offset + (i - 8) * 4)
                }
            }
            koopa::ir::ValueKind::GlobalAlloc(_) => panic!("a local global?"),
            _ => writeln!(f, "lw {}, {}(sp)", register, self.offset - self.map[&value]),
        }
    }

    fn store<W: Write>(
        &self,
        f: &mut W,
        register: &str,
        value: koopa::ir::Value,
    ) -> std::io::Result<()> {
        if value.is_global() {
            writeln!(f, "sw {}, {}(fp)", register, self.map[&value])
        } else {
            writeln!(f, "sw {}, {}(sp)", register, self.offset - self.map[&value])
        }
    }
}

fn emit_function<W: Write>(
    f: &mut W,
    program: &koopa::ir::Program,
    globals: &StackFrame,
    func: koopa::ir::Function,
) -> std::io::Result<()> {
    let func_data = program.func(func);
    if let None = func_data.layout().entry_bb() {
        return Ok(());
    }
    let is_main = func_data.name() == "@main";
    let mut stack_frame = StackFrame {
        offset: 4,
        map: globals.map.clone(),
    };
    if is_main {
        stack_frame.offset += globals.offset;
    }
    let mut max_args = 0;
    for (&bb, node) in func_data.layout().bbs() {
        for &param in func_data.dfg().bb(bb).params() {
            stack_frame.offset += 4;
            stack_frame.map.insert(param, stack_frame.offset);
        }
        for &inst in node.insts().keys() {
            if !func_data.dfg().value(inst).ty().is_unit() {
                stack_frame.offset += 4;
                stack_frame.map.insert(inst, stack_frame.offset);
            }
            if let koopa::ir::ValueKind::Call(call) = func_data.dfg().value(inst).kind() {
                max_args = max_args.max(call.args().len());
            }
        }
    }
    stack_frame.offset = StackFrame::round(stack_frame.offset + max_args * 4);
    writeln!(
        f,
        "\n{}:\naddi sp, sp, -{}\nsw ra, {}(sp)",
        &func_data.name()[1..],
        stack_frame.offset,
        stack_frame.offset - 4,
    )?;
    if is_main {
        writeln!(
            f,
            "addi fp, sp, {}",
            stack_frame.offset - globals.offset - 4,
        )?;
        for &var in program.inst_layout() {
            let alloc = program.borrow_value(var);
            let koopa::ir::ValueKind::GlobalAlloc(alloc) = alloc.kind() else {
                panic!("top-level computation")
            };
            match program.borrow_value(alloc.init()).kind() {
                koopa::ir::ValueKind::Integer(integer) => {
                    writeln!(f, "li t1, {}", integer.value())?;
                    stack_frame.store(f, "t1", var)
                }
                koopa::ir::ValueKind::ZeroInit(_) | koopa::ir::ValueKind::Undef(_) => {
                    writeln!(f, "li t1, 0")?;
                    stack_frame.store(f, "t1", var)
                }
                koopa::ir::ValueKind::Aggregate(_) => todo!(),
                _ => panic!("invalid initializer for global variable"),
            }?;
        }
    }
    for (&bb, node) in func_data.layout().bbs() {
        writeln!(
            f,
            "{}:",
            &func_data.dfg().bb(bb).name().as_ref().unwrap()[1..]
        )?;
        for &inst in node.insts().keys() {
            emit_value(f, program, func_data.dfg(), &stack_frame, inst)?;
        }
    }
    Ok(())
}

fn emit_value<W: Write>(
    f: &mut W,
    program: &koopa::ir::Program,
    dfg: &koopa::ir::dfg::DataFlowGraph,
    stack_frame: &StackFrame,
    value: koopa::ir::Value,
) -> std::io::Result<()> {
    match dfg.value(value).kind() {
        koopa::ir::ValueKind::Integer(_) => panic!("how did you do that?"),
        koopa::ir::ValueKind::ZeroInit(_) => todo!(),
        koopa::ir::ValueKind::Undef(_) => todo!(),
        koopa::ir::ValueKind::Aggregate(_) => todo!(),
        koopa::ir::ValueKind::FuncArgRef(_) => panic!("how did you do that?"),
        koopa::ir::ValueKind::BlockArgRef(_) => panic!("how did you do that?"),
        koopa::ir::ValueKind::Alloc(_) => Ok(()),
        koopa::ir::ValueKind::GlobalAlloc(_) => panic!("how? that's not a local value"),
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
            for (&arg, &param) in jump.args().iter().zip(dfg.bb(jump.target()).params()) {
                stack_frame.load(f, dfg, "t1", arg)?;
                stack_frame.store(f, "t1", param)?;
            }
            writeln!(
                f,
                "j {}",
                &dfg.bb(jump.target()).name().as_ref().unwrap()[1..]
            )
        }
        koopa::ir::ValueKind::Call(call) => {
            let mut args = call.args().iter();
            for (i, &arg) in args.by_ref().take(8).enumerate() {
                stack_frame.load(f, dfg, &format!("a{}", i), arg)?;
            }
            for (i, &arg) in args.enumerate() {
                stack_frame.load(f, dfg, "t1", arg)?;
                writeln!(f, "sw t1, {}(sp)", i * 4)?;
            }
            writeln!(f, "call {}", &program.func(call.callee()).name()[1..])?;
            stack_frame.store(f, "a0", value)
        }
        koopa::ir::ValueKind::Return(ret) => {
            if let Some(value) = ret.value() {
                stack_frame.load(f, dfg, "a0", value)?;
            }
            writeln!(
                f,
                "lw ra, {}(sp)\naddi sp, sp, {}\nret",
                stack_frame.offset - 4,
                stack_frame.offset,
            )
        }
    }
}
