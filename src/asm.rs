use std::{collections::HashMap, io::Write};

fn flatten_initializer<GetValue: Copy + Fn(koopa::ir::Value) -> koopa::ir::entities::ValueData>(
    value: koopa::ir::Value,
    get_value: GetValue,
) -> Vec<Option<koopa::ir::Value>> {
    match get_value(value).kind() {
        koopa::ir::ValueKind::ZeroInit(_) | koopa::ir::ValueKind::Undef(_) => {
            let size = get_value(value).ty().size();
            vec![None; size / 4]
        }
        koopa::ir::ValueKind::Aggregate(aggregate) => {
            let mut result = Vec::new();
            for &elem in aggregate.elems() {
                result.extend(flatten_initializer(elem, get_value));
            }
            result
        }
        _ => vec![Some(value)],
    }
}

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
        globals.offset += ty.size() + 4;
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
            return writeln!(
                f,
                "li t3, {}\nadd t3, fp, t3\nlw {register}, (t3)",
                self.map[&value],
            );
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
                    writeln!(
                        f,
                        "li t3, {}\nadd t3, sp, t3\nlw {register}, (t3)",
                        self.offset + (i - 8) * 4,
                    )
                }
            }
            koopa::ir::ValueKind::GlobalAlloc(_) => panic!("a local global?"),
            _ => writeln!(
                f,
                "li t3, {}\nadd t3, sp, t3\nlw {register}, (t3)",
                self.offset - self.map[&value],
            ),
        }
    }

    fn load_address<W: Write>(
        &self,
        f: &mut W,
        register: &str,
        value: koopa::ir::Value,
    ) -> std::io::Result<()> {
        if value.is_global() {
            writeln!(
                f,
                "li {register}, {}\nadd {register}, fp, {register}",
                self.map[&value] + 4,
            )
        } else {
            writeln!(
                f,
                "li {register}, {}\nadd {register}, sp, {register}",
                self.offset - self.map[&value] + 4,
            )
        }
    }

    fn store<W: Write>(
        &self,
        f: &mut W,
        register: &str,
        value: koopa::ir::Value,
    ) -> std::io::Result<()> {
        if value.is_global() {
            writeln!(
                f,
                "li t3, {}\nadd t3, fp, t3\nsw {}, (t3)",
                self.map[&value], register,
            )
        } else {
            writeln!(
                f,
                "li t3, {}\nadd t3, sp, t3\nsw {}, (t3)",
                self.offset - self.map[&value],
                register,
            )
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
            let ty = func_data.dfg().value(inst).ty();
            if let koopa::ir::ValueKind::Alloc(_) = func_data.dfg().value(inst).kind()
                && let koopa::ir::TypeKind::Pointer(ty) = ty.kind()
            {
                stack_frame.offset += ty.size();
            }
            if !ty.is_unit() {
                stack_frame.offset += ty.size();
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
        "\n{}:\nsw ra, -4(sp)\nli t1, -{}\nadd sp, sp, t1",
        &func_data.name()[1..],
        stack_frame.offset,
    )?;
    if is_main {
        writeln!(
            f,
            "li t1, {}\nadd fp, sp, t1",
            stack_frame.offset - globals.offset - 4,
        )?;
        for &var in program.inst_layout() {
            let alloc = program.borrow_value(var);
            let koopa::ir::ValueKind::GlobalAlloc(alloc) = alloc.kind() else {
                panic!("top-level computation")
            };
            stack_frame.load_address(f, "t1", var)?;
            stack_frame.store(f, "t1", var)?;
            match program.borrow_value(alloc.init()).kind() {
                koopa::ir::ValueKind::ZeroInit(_) | koopa::ir::ValueKind::Undef(_) => {
                    let size = program.borrow_value(alloc.init()).ty().size();
                    writeln!(
                        f,
                        "li t2, {size}
add t2, t1, t2
1:
sw x0, (t1)
addi t1, t1, 4
blt t1, t2, 1b"
                    )?;
                }
                _ => {
                    let initializer = flatten_initializer(alloc.init(), |value| {
                        program.borrow_value(value).clone()
                    });
                    for elem in initializer {
                        let integer = match elem {
                            Some(elem) => {
                                let elem = program.borrow_value(elem);
                                let koopa::ir::ValueKind::Integer(integer) = elem.kind() else {
                                    panic!("nested initializer list not supported")
                                };
                                integer.value()
                            }
                            None => 0,
                        };
                        writeln!(f, "li t2, {}\nsw t2, (t1)\naddi t1, t1, 4", integer)?;
                    }
                }
            }
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
        koopa::ir::ValueKind::ZeroInit(_) => panic!("how did you do that?"),
        koopa::ir::ValueKind::Undef(_) => panic!("how did you do that?"),
        koopa::ir::ValueKind::Aggregate(_) => panic!("how did you do that?"),
        koopa::ir::ValueKind::FuncArgRef(_) => panic!("how did you do that?"),
        koopa::ir::ValueKind::BlockArgRef(_) => panic!("how did you do that?"),
        koopa::ir::ValueKind::Alloc(_) => {
            stack_frame.load_address(f, "t1", value)?;
            stack_frame.store(f, "t1", value)
        }
        koopa::ir::ValueKind::GlobalAlloc(_) => panic!("how? that's not a local value"),
        koopa::ir::ValueKind::Load(load) => {
            stack_frame.load(f, dfg, "t1", load.src())?;
            writeln!(f, "lw t1, (t1)")?;
            stack_frame.store(f, "t1", value)
        }
        koopa::ir::ValueKind::Store(store) => {
            let value = store.value();
            let elems = flatten_initializer(value, |value| dfg.value(value).clone());
            stack_frame.load(f, dfg, "t2", store.dest())?;
            for elem in elems {
                match elem {
                    Some(elem) => {
                        stack_frame.load(f, dfg, "t1", elem)?;
                        writeln!(f, "sw t1, (t2)")?;
                    }
                    None => {
                        writeln!(f, "sw x0, (t2)")?;
                    }
                }
                writeln!(f, "addi t2, t2, 4")?;
            }
            Ok(())
        }
        koopa::ir::ValueKind::GetPtr(get_ptr) => {
            stack_frame.load(f, dfg, "t1", get_ptr.index())?;
            let src = get_ptr.src();
            let src = if src.is_global() {
                &program.borrow_value(src)
            } else {
                dfg.value(src)
            };
            let koopa::ir::TypeKind::Pointer(ty) = src.ty().kind() else {
                panic!("how? a value to getptr of?")
            };
            writeln!(f, "li t2, {}\nmul t1, t1, t2", ty.size())?;
            stack_frame.load(f, dfg, "t2", get_ptr.src())?;
            writeln!(f, "add t1, t2, t1")?;
            stack_frame.store(f, "t1", value)
        }
        koopa::ir::ValueKind::GetElemPtr(get_elem_ptr) => {
            stack_frame.load(f, dfg, "t1", get_elem_ptr.index())?;
            let src = get_elem_ptr.src();
            let src = if src.is_global() {
                &program.borrow_value(src)
            } else {
                dfg.value(src)
            };
            let koopa::ir::TypeKind::Pointer(ty) = src.ty().kind() else {
                panic!("how? a value to getelemptr of?")
            };
            let koopa::ir::TypeKind::Array(ty, _) = ty.kind() else {
                panic!("how? a basic value to getelemptr of?")
            };
            writeln!(f, "li t2, {}\nmul t1, t1, t2", ty.size())?;
            stack_frame.load(f, dfg, "t2", get_elem_ptr.src())?;
            writeln!(f, "add t1, t2, t1")?;
            stack_frame.store(f, "t1", value)
        }
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
                "li t1, {}\nadd sp, sp, t1\nlw ra, -4(sp)\nret",
                stack_frame.offset,
            )
        }
    }
}
