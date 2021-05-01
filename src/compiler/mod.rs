pub mod compiler;

#[cfg(test)]
mod tests {
    use crate::{
        parser::{lexer::Lexer, parser::Parser},
        vm::{
            instruction::{Instruction, Opcode},
            value::Value,
            VM,
        },
    };

    use super::compiler::Compiler;

    #[test]
    pub fn compiler() {
        let src = r#"let a=3; a"#;

        let tokens = Lexer::new(src).scan_all();

        let statements = Parser::new(tokens).parse_all();

        let mut instructions = Compiler::new(statements).compile();

        instructions.push(Instruction::Op(Opcode::Constant));
        instructions.push(Instruction::Operand(Value::Ident(String::from("a"))));
        instructions.push(Instruction::Op(Opcode::GetGlobal));

        dbg!(&instructions);

        let mut vm = VM::new(instructions);
        vm.interpret().unwrap();

        vm.stack.dump();
    }
}
