use dash_middle::compiler::constant::ConstantPool;
use dash_middle::compiler::extract::NoValueExtractContext;
use dash_middle::compiler::operands::{FunctionConstantOperands, InstructionOperands, InstructionSuccessorOffset};
use dash_middle::interner::StringInterner;
use std::fmt::Write;

fn pretty_into(out: &mut String, name: &str, bytecode: &[u8], constants: &ConstantPool, interner: &StringInterner) {
    write!(out, "{name}:\n").unwrap();
    let mut nested_functions = Vec::new();

    let mut ecx = NoValueExtractContext {
        bytes: bytecode,
        ip: 0,
        constants: constants,
    };

    while let Some((instr_ip, instr)) = ecx.next_instruction_with_operands() {
        if let Some(successor) = instr.successor() {
            let (InstructionSuccessorOffset::Conditional(offset) | InstructionSuccessorOffset::Unconditional(offset)) =
                successor;
            let target = ecx.ip.strict_add_signed(offset.into());
            write!(out, "{:04x}: {:?} # {:#x}\n", instr_ip, instr, target).unwrap();
        } else {
            write!(out, "{:04x}: {:?}\n", instr_ip, instr).unwrap();
        };

        if let InstructionOperands::Function(FunctionConstantOperands(func)) = instr {
            nested_functions.push(func);
        }
    }

    for func in nested_functions {
        out.push('\n');
        let func = &constants.functions[func];
        let name = func.name.map(|name| interner.resolve(name)).unwrap_or("<anonymous>");
        func.buffer.with(|buffer| {
            pretty_into(out, name, buffer, &func.constants, interner);
        });
    }
}

/// Formats raw bytecode as a string
pub fn pretty(bytecode: &[u8], constants: &ConstantPool, interner: &StringInterner) -> String {
    let mut out = String::new();
    pretty_into(&mut out, "main", bytecode, constants, interner);
    out
}
