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
        let src = r#"let a=3+1; a+1"#;

        let tokens = Lexer::new(src).scan_all();

        let statements = Parser::new(tokens).parse_all();

        let instructions = Compiler::new(statements).compile();

        let mut vm = VM::new(instructions);
        vm.interpret().unwrap();

        vm.stack.dump();
    }
}
