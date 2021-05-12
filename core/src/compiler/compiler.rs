use crate::{
    parser::{
        expr::{
            AssignmentExpr, BinaryExpr, ConditionalExpr, Expr, FunctionCall, GroupingExpr,
            LiteralExpr, PropertyAccessExpr, Seq, UnaryExpr,
        },
        statement::{
            BlockStatement, FunctionDeclaration, IfStatement, Print, ReturnStatement, Statement,
            VariableDeclaration, WhileLoop,
        },
        token::TokenType,
    },
    visitor::Visitor,
    vm::{
        instruction::{Instruction, Opcode},
        value::{FunctionType, UserFunction, Value, ValueKind},
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
        let scope = ScopeGuard::new();
        Self {
            ast: Some(ast),
            scope,
        }
    }

    pub fn with_scopeguard(ast: Ast<'a>, scope: ScopeGuard<'a, 1024>) -> Self {
        Self {
            ast: Some(ast),
            scope,
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
                    instructions.push(Instruction::Operand(Value::new(ValueKind::Number(
                        stack_idx as f64,
                    ))));
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

        match e.operator {
            TokenType::LogicalAnd | TokenType::LogicalOr => {
                let ty = e.operator;

                instructions.push(Instruction::Op(Opcode::Constant));
                jmp_idx = isize::try_from(instructions.len()).unwrap();
                instructions.push(Instruction::Op(Opcode::Nop));

                if ty == TokenType::LogicalAnd {
                    instructions.push(Instruction::Op(Opcode::ShortJmpIfFalse));
                } else {
                    instructions.push(Instruction::Op(Opcode::ShortJmpIfTrue));
                }

                instructions.push(Instruction::Op(Opcode::Pop));
            }
            _ => {}
        };

        let right = self.accept_expr(&e.right);
        instructions.extend(right);

        if jmp_idx > -1 {
            let jmp_idx = jmp_idx as usize;

            let instruction_count = instructions.len() - jmp_idx - 2;
            instructions[jmp_idx] =
                Instruction::Operand(Value::new(ValueKind::Number(instruction_count as f64)));
        } else {
            instructions.push(Instruction::Op(e.operator.into()));
        }

        instructions
    }

    fn visit_while_loop(&mut self, l: &WhileLoop<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&l.condition);

        instructions.push(Instruction::Op(Opcode::Constant));
        let jmp_idx = instructions.len();
        instructions.push(Instruction::Op(Opcode::Nop));

        instructions.push(Instruction::Op(Opcode::ShortJmpIfFalse));

        // Compile body
        instructions.extend(self.accept(&l.body));

        let instruction_count_ = instructions.len() - jmp_idx + 1;
        let instruction_count =
            Instruction::Operand(Value::new(ValueKind::Number(instruction_count_ as f64)));
        instructions[jmp_idx] = instruction_count.clone();

        // Emit backjump to evaluate condition
        instructions.push(Instruction::Op(Opcode::Constant));
        let backjmp_count = instruction_count_ + jmp_idx + 2;
        instructions.push(Instruction::Operand(Value::new(ValueKind::Number(
            backjmp_count as f64,
        ))));
        instructions.push(Instruction::Op(Opcode::BackJmp));

        instructions
    }

    fn visit_grouping_expression(&mut self, e: &GroupingExpr<'a>) -> Vec<Instruction> {
        self.accept_expr(&e.0)
    }

    fn visit_unary_expression(&mut self, e: &UnaryExpr<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&e.expr);

        match e.operator {
            TokenType::Minus => instructions.push(Instruction::Op(Opcode::Negate)),
            TokenType::Typeof => instructions.push(Instruction::Op(Opcode::Typeof)),
            _ => todo!(),
        }

        return instructions;
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
            instructions.push(Instruction::Operand(Value::new(ValueKind::Number(
                stack_idx as f64,
            ))));
        } else {
            instructions.push(Instruction::Op(Opcode::Constant));
            instructions.push(Instruction::Operand(Value::new(ValueKind::Ident(
                std::str::from_utf8(v.name).unwrap().to_owned(),
            ))));
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
        instructions.push(Instruction::Op(Opcode::Nop));
        instructions.push(Instruction::Op(Opcode::ShortJmpIfFalse));

        let then_instructions = self.accept(&i.then);
        instructions[jmp_idx] = Instruction::Operand(Value::new(ValueKind::Number(
            then_instructions.len() as f64,
        )));

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
        let mut instructions = vec![Instruction::Op(Opcode::Constant)];

        let params = f.arguments.len();
        let statements = f.statements.clone(); // TODO: somehow avoid this clone

        let mut scope = ScopeGuard::new();
        scope.enter_scope();
        for argument in &f.arguments {
            scope.push_local(Local::new(argument, 0));
        }

        let mut func_instructions = Self::with_scopeguard(statements, scope).compile();

        if func_instructions.len() == 0 {
            func_instructions.push(Instruction::Op(Opcode::Constant));
            func_instructions.push(Instruction::Operand(Value::new(ValueKind::Undefined)));
            func_instructions.push(Instruction::Op(Opcode::Return));
        } else if let Some(Instruction::Op(op)) = func_instructions.last() {
            if !op.eq(&Opcode::Return) {
                func_instructions.push(Instruction::Op(Opcode::Constant));
                func_instructions.push(Instruction::Operand(Value::new(ValueKind::Undefined)));
                func_instructions.push(Instruction::Op(Opcode::Return));
            }
        }

        let func = UserFunction::new(func_instructions, params as u32, FunctionType::Function);
        instructions.push(Instruction::Operand(func.into()));

        instructions.push(Instruction::Op(Opcode::Constant));
        instructions.push(Instruction::Operand(Value::new(ValueKind::Ident(
            std::str::from_utf8(f.name).unwrap().to_owned(),
        ))));

        if self.scope.is_global() {
            instructions.push(Instruction::Op(Opcode::SetGlobal));
        } else {
            todo!()
            // instructions.push(Instruction::Op(Opcode::SetLocal));
        }

        instructions
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

    fn visit_function_call(&mut self, c: &FunctionCall<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&c.target);

        let argument_len = c.arguments.len();

        for argument in &c.arguments {
            instructions.extend(self.accept_expr(argument));
        }

        instructions.push(Instruction::Op(Opcode::Constant));
        instructions.push(Instruction::Operand(Value::new(ValueKind::Number(
            argument_len as f64,
        ))));

        instructions.push(Instruction::Op(Opcode::FunctionCall));

        instructions
    }

    fn visit_return_statement(&mut self, s: &ReturnStatement<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&s.0);
        instructions.push(Instruction::Op(Opcode::Return));
        instructions
    }

    fn visit_conditional_expr(&mut self, c: &ConditionalExpr<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&c.condition);

        instructions.push(Instruction::Op(Opcode::Constant));
        let then_jmp_idx = instructions.len();
        instructions.push(Instruction::Op(Opcode::Nop));

        instructions.push(Instruction::Op(Opcode::ShortJmpIfFalse));
        let then_instructions = self.accept_expr(&c.then);
        let then_instruction_count = then_instructions.len();
        instructions.extend(then_instructions);
        instructions[then_jmp_idx] = Instruction::Operand(Value::new(ValueKind::Number(
            (then_instruction_count + 3) as f64,
        )));

        instructions.push(Instruction::Op(Opcode::Constant));
        let else_jmp_idx = instructions.len();
        instructions.push(Instruction::Op(Opcode::Nop));
        instructions.push(Instruction::Op(Opcode::ShortJmp));

        let else_instructions = self.accept_expr(&c.el);
        let else_instruction_count = else_instructions.len();
        instructions[else_jmp_idx] =
            Instruction::Operand(Value::new(ValueKind::Number(else_instruction_count as f64)));
        instructions.extend(else_instructions);

        instructions
    }

    fn visit_property_access_expr(&mut self, e: &PropertyAccessExpr<'a>) -> Vec<Instruction> {
        assert!(!e.computed); // computed property access foo[bar] doesnt work yet

        let mut instructions = self.accept_expr(&e.target);

        let ident: &[u8] = if let Expr::Literal(lit) = &*e.property {
            match lit {
                LiteralExpr::Identifier(ident) => ident,
                _ => todo!(),
            }
        } else {
            todo!()
        };

        instructions.push(Instruction::Op(Opcode::Constant));
        instructions.push(Instruction::Operand(Value::new(ValueKind::Ident(
            std::str::from_utf8(ident).unwrap().to_owned(),
        ))));

        instructions.push(Instruction::Op(Opcode::StaticPropertyAccess));

        instructions
    }

    fn visit_sequence_expr(&mut self, s: &Seq<'a>) -> Vec<Instruction> {
        let mut instructions = self.accept_expr(&s.0);
        instructions.push(Instruction::Op(Opcode::Pop));

        let rhs = self.accept_expr(&s.1);
        instructions.extend(rhs);

        instructions
    }
}
