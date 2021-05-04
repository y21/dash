use crate::{
    parser::{
        expr::{AssignmentExpr, BinaryExpr, Expr, GroupingExpr, LiteralExpr, UnaryExpr},
        statement::{
            BlockStatement, FunctionDeclaration, IfStatement, Print, Statement,
            VariableDeclaration, WhileLoop,
        },
        token::TokenType,
    },
    visitor::Visitor,
    vm::{
        instruction::{Instruction, Opcode},
        stack::Stack,
        value::Value,
    },
};
use std::convert::TryFrom;

use super::scope::{Local, ScopeGuard};

pub type Ast<'a> = Vec<Statement<'a>>;

#[derive(Debug)]
pub struct Compiler<'a> {
    ast: Option<Ast<'a>>,
    scope: ScopeGuard<'a, 1024>,
}

impl<'a> Compiler<'a> {
    pub fn new(ast: Ast<'a>) -> Self {
        Self {
            ast: Some(ast),
            scope: ScopeGuard::new(),
        }
    }

    pub fn compile(mut self) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        let statements = self.ast.take().unwrap();

        for statement in statements {
            for instruction in self.accept(&statement) {
                instructions.push(instruction);
            }
        }

        instructions
    }
}

impl<'a> Visitor<'a, Vec<Instruction>> for Compiler<'a> {
    fn visit_literal_expression(&mut self, e: &LiteralExpr<'a>) -> Vec<Instruction> {
        let mut instructions = Vec::with_capacity(3);
        instructions.push(Instruction::Op(Opcode::Constant));

        if let LiteralExpr::Identifier(ident) = e {
            if !self.scope.is_global() {
                let stack_idx = self.scope.find_variable(ident);

                if let Some(stack_idx) = stack_idx {
                    instructions.push(Instruction::Operand(Value::Number(stack_idx as f64)));
                    instructions.push(Instruction::Op(Opcode::GetLocal));
                    return instructions;
                }
            }

            instructions.push(Instruction::Operand(e.to_value()));
            instructions.push(Instruction::Op(Opcode::GetGlobal));
        } else {
            instructions.push(Instruction::Operand(e.to_value()));
        }

        instructions
    }

    fn visit_binary_expression(&mut self, e: &BinaryExpr<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&e.left);

        // Will stay -1 if it's not && or ||
        // TODO: implement ??
        let mut jmp_idx: isize = -1;

        if [TokenType::LogicalAnd, TokenType::LogicalOr].contains(&e.operator) {
            let ty = e.operator;

            instructions.push(Instruction::Op(Opcode::Constant));
            jmp_idx = isize::try_from(instructions.len()).unwrap();
            instructions.push(Instruction::Operand(Value::Number(0f64)));

            if ty == TokenType::LogicalAnd {
                instructions.push(Instruction::Op(Opcode::ShortJmpIfFalse));
            } else {
                instructions.push(Instruction::Op(Opcode::ShortJmpIfTrue));
            }

            instructions.push(Instruction::Op(Opcode::Pop));
        }

        let right = self.accept_expr(&e.right);
        instructions.extend(right);

        if jmp_idx > -1 {
            let jmp_idx = jmp_idx as usize;

            let instruction_count = instructions.len() - jmp_idx - 2;
            instructions[jmp_idx] = Instruction::Operand(Value::Number(instruction_count as f64));
        } else {
            instructions.push(Instruction::Op(e.operator.into()));
        }

        instructions
    }

    fn visit_while_loop(&mut self, l: &WhileLoop<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&l.condition);

        instructions.push(Instruction::Op(Opcode::Constant));
        let jmp_idx = instructions.len();
        instructions.push(Instruction::Operand(Value::Number(0f64)));

        instructions.push(Instruction::Op(Opcode::ShortJmpIfFalse));

        // Compile body
        instructions.extend(self.accept(&l.body));

        let instruction_count_ = instructions.len() - jmp_idx + 1;
        let instruction_count = Instruction::Operand(Value::Number(instruction_count_ as f64));
        instructions[jmp_idx] = instruction_count.clone();

        // Emit backjump to evaluate condition
        instructions.push(Instruction::Op(Opcode::Constant));
        let backjmp_count = instruction_count_ + jmp_idx + 2;
        instructions.push(Instruction::Operand(Value::Number(backjmp_count as f64)));
        instructions.push(Instruction::Op(Opcode::BackJmp));

        instructions
    }

    fn visit_grouping_expression(&mut self, e: &GroupingExpr<'a>) -> Vec<Instruction> {
        self.accept_expr(&e.0)
    }

    fn visit_unary_expression(&mut self, e: &UnaryExpr<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&e.expr);
        // TODO: don't assume negation to be `-`, it could be ! or any other unary operator
        instructions.push(Instruction::Op(Opcode::Negate));
        instructions
    }

    fn visit_variable_declaration(&mut self, v: &VariableDeclaration<'a>) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        if let Some(value) = &v.value {
            instructions.extend(self.accept_expr(value));
        }

        let global = self.scope.is_global();

        let (op_with_value, op_no_value) = if global {
            (Opcode::SetGlobal, Opcode::SetGlobalNoValue)
        } else {
            (Opcode::SetLocal, Opcode::SetLocalNoValue)
        };

        if !global {
            let stack_idx = self.scope.push_local(Local::new(v.name, self.scope.depth));
            instructions.push(Instruction::Op(Opcode::Constant));
            instructions.push(Instruction::Operand(Value::Number(stack_idx as f64)));
        } else {
            instructions.push(Instruction::Op(Opcode::Constant));
            instructions.push(Instruction::Operand(Value::Ident(
                std::str::from_utf8(v.name).unwrap().to_owned(),
            )));
        }

        if v.value.is_some() {
            instructions.push(Instruction::Op(op_with_value));
        } else {
            instructions.push(Instruction::Op(op_no_value));
        }

        instructions
    }

    fn visit_if_statement(&mut self, i: &IfStatement<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&i.condition);

        instructions.push(Instruction::Op(Opcode::Constant));
        let jmp_idx = instructions.len();
        instructions.push(Instruction::Operand(Value::Number(0f64)));
        instructions.push(Instruction::Op(Opcode::ShortJmpIfFalse));

        let then_instructions = self.accept(&i.then);
        instructions[jmp_idx] = Instruction::Operand(Value::Number(then_instructions.len() as f64));

        instructions.extend(then_instructions);

        instructions
    }

    fn visit_block_statement(&mut self, b: &BlockStatement<'a>) -> Vec<Instruction> {
        self.scope.enter_scope();
        let instructions = b.0.iter().map(|s| self.accept(s)).flatten().collect();
        self.scope.leave_scope();
        instructions
    }

    fn visit_function_declaration(&mut self, f: &FunctionDeclaration<'a>) -> Vec<Instruction> {
        todo!()
    }

    fn visit_assignment_expression(&mut self, e: &AssignmentExpr<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&e.right);
        instructions.extend(self.accept_expr(&e.left));
        instructions.push(Instruction::Op(e.operator.into()));

        instructions
    }

    fn visit_expression_statement(&mut self, e: &Expr<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(e);
        instructions.push(Instruction::Op(Opcode::Pop));
        instructions
    }

    fn visit_print_statement(&mut self, p: &Print<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&p.0);
        instructions.push(Instruction::Op(Opcode::Print));
        instructions
    }
}
