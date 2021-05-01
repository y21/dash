use crate::{
    parser::{
        expr::{BinaryExpr, GroupingExpr, LiteralExpr, UnaryExpr},
        statement::{Statement, VariableDeclaration},
    },
    visitor::Visitor,
    vm::{
        instruction::{Instruction, Opcode},
        value::Value,
    },
};

pub type Ast<'a> = Vec<Statement<'a>>;

#[derive(Debug)]
pub struct Compiler<'a> {
    ast: Ast<'a>,
}

impl<'a> Compiler<'a> {
    pub fn new(ast: Ast<'a>) -> Self {
        Self { ast }
    }

    pub fn compile(self) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        for statement in &self.ast {
            for instruction in self.accept(&statement) {
                instructions.push(instruction);
            }
        }

        instructions
    }
}

impl<'a> Visitor<'a, Vec<Instruction>> for Compiler<'a> {
    fn visit_literal_expression(&self, e: &LiteralExpr<'a>) -> Vec<Instruction> {
        vec![
            Instruction::Op(Opcode::Constant),
            Instruction::Operand(e.to_value()),
        ]
    }

    fn visit_binary_expression(&self, e: &BinaryExpr<'a>) -> Vec<Instruction> {
        let mut left = self.accept_expr(&e.left);
        let right = self.accept_expr(&e.right);

        left.extend(right);
        left.push(Instruction::Op(e.operator.into()));
        left
    }

    fn visit_grouping_expression(&self, e: &GroupingExpr<'a>) -> Vec<Instruction> {
        self.accept_expr(&e.0)
    }

    fn visit_unary_expression(&self, e: &UnaryExpr<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&e.expr);
        instructions.push(Instruction::Op(Opcode::Negate));
        instructions
    }

    fn visit_variable_declaration(&self, v: &VariableDeclaration<'a>) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        if let Some(value) = &v.value {
            instructions.extend(self.accept_expr(value));
        }

        instructions.push(Instruction::Op(Opcode::Constant));
        instructions.push(Instruction::Operand(Value::Ident(
            std::str::from_utf8(v.name).unwrap().to_owned(),
        )));

        if v.value.is_some() {
            instructions.push(Instruction::Op(Opcode::SetGlobal));
        } else {
            instructions.push(Instruction::Op(Opcode::SetGlobalNoValue));
        }

        instructions
    }
}
