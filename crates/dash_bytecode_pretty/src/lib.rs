use dash_middle::compiler::constant::ConstantPool;
use dash_middle::compiler::extract::NoValueExtractContext;
use dash_middle::compiler::operands::InstructionSuccessorOffset;
use std::fmt::Write;

/// Formats raw bytecode as a string
pub fn pretty(bytecode: &[u8], constants: &ConstantPool) -> String {
    let mut ecx = NoValueExtractContext {
        bytes: bytecode,
        ip: 0,
        constants,
    };

    let mut out = String::new();

    while let Some((instr_ip, instr)) = ecx.next_instruction_with_operands() {
        if let Some(successor) = instr.successor() {
            let (InstructionSuccessorOffset::Conditional(offset) | InstructionSuccessorOffset::Unconditional(offset)) =
                successor;
            let target = ecx.ip.strict_add_signed(offset.into());
            write!(&mut out, "{:04x}: {:?} # {:#x}\n", instr_ip, instr, target).unwrap();
        } else {
            write!(&mut out, "{:04x}: {:?}\n", instr_ip, instr).unwrap();
        };
    }

    out
}
