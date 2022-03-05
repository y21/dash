use std::convert::TryInto;

use crate::{
    compiler::builder::{InstructionBuilder, Label},
    parser::{
        expr::{
            ArrayLiteral, AssignmentExpr, BinaryExpr, ConditionalExpr, Expr, FunctionCall,
            GroupingExpr, LiteralExpr, ObjectLiteral, Postfix, PropertyAccessExpr, Seq, UnaryExpr,
        },
        statement::{
            BlockStatement, Class, ExportKind, ForLoop, ForOfLoop, FunctionDeclaration,
            IfStatement, ImportKind, ReturnStatement, Statement, TryCatch, VariableBinding,
            VariableDeclaration, VariableDeclarationKind, WhileLoop,
        },
        token::TokenType,
    },
};

use self::{
    builder::{force_utf8, force_utf8_borrowed},
    constant::{Constant, ConstantPool, Function},
    error::CompileError,
    instruction::InstructionWriter,
    scope::Scope,
    visitor::Visitor,
};

pub mod builder;
pub mod constant;
#[cfg(feature = "decompile")]
pub mod decompiler;
pub mod error;
pub mod instruction;
mod scope;
/// Visitor trait, used to walk the AST
mod visitor;

macro_rules! unimplementedc {
    ($($what:expr),*) => {
        return Err(CompileError::Unimplemented(format_args!($($what),*).to_string()))
    };
}

pub struct SharedCompilerState<'a> {
    cp: ConstantPool,
    scope: Scope<'a>,
}

impl<'a> SharedCompilerState<'a> {
    pub fn new() -> Self {
        Self {
            cp: ConstantPool::new(),
            scope: Scope::new(),
        }
    }
}

pub struct FunctionCompiler<'a, 's> {
    state: SharedCompilerState<'a>,
    caller: Option<&'s mut SharedCompilerState<'a>>,
}

#[derive(Debug)]
pub struct CompileResult {
    pub instructions: Vec<u8>,
    pub cp: ConstantPool,
    pub locals: usize,
}

fn ast_insert_return<'a>(ast: &mut Vec<Statement<'a>>) {
    match ast.last_mut() {
        Some(Statement::Return(..)) => {}
        Some(Statement::Expression(_)) => {
            let expr = if let Statement::Expression(expr) = ast.pop().unwrap() {
                expr
            } else {
                unreachable!()
            };

            ast.push(Statement::Return(ReturnStatement(expr)));
        }
        Some(Statement::Block(b)) => ast_insert_return(&mut b.0),
        _ => ast.push(Statement::Return(ReturnStatement::default())),
    }
}

impl<'a, 's> FunctionCompiler<'a, 's> {
    pub fn new() -> Self {
        Self {
            state: SharedCompilerState::new(),
            caller: None,
        }
    }

    pub fn with_caller(caller: &'s mut SharedCompilerState<'a>) -> Self {
        Self {
            state: SharedCompilerState::new(),
            caller: Some(caller),
        }
    }

    pub fn compile_ast(
        mut self,
        mut ast: Vec<Statement<'a>>,
    ) -> Result<CompileResult, CompileError> {
        ast_insert_return(&mut ast);
        let instructions = self.accept_multiple(&ast)?;
        Ok(CompileResult {
            instructions,
            cp: self.state.cp,
            locals: self.state.scope.locals().len(),
        })
    }

    pub fn accept_multiple(&mut self, v: &[Statement<'a>]) -> Result<Vec<u8>, CompileError> {
        let mut insts = Vec::new();

        for stmt in v {
            insts.append(&mut self.accept(stmt)?);
        }

        Ok(insts)
    }
}

impl<'a, 's> Visitor<'a, Result<Vec<u8>, CompileError>> for FunctionCompiler<'a, 's> {
    fn visit_binary_expression(&mut self, e: &BinaryExpr<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        ib.append(&mut self.accept_expr(&e.left)?);
        ib.append(&mut self.accept_expr(&e.right)?);

        match e.operator {
            TokenType::Plus => ib.build_add(),
            TokenType::Minus => ib.build_sub(),
            TokenType::Star => ib.build_mul(),
            TokenType::Slash => ib.build_div(),
            TokenType::Remainder => ib.build_rem(),
            TokenType::Exponentiation => ib.build_pow(),
            TokenType::Greater => ib.build_gt(),
            TokenType::GreaterEqual => ib.build_ge(),
            TokenType::Less => ib.build_lt(),
            TokenType::LessEqual => ib.build_le(),
            TokenType::Equality => ib.build_eq(),
            TokenType::Inequality => ib.build_ne(),
            other => unreachable!("Binary token is never emitted: {:?}", other),
        }
        Ok(ib.build())
    }

    fn visit_expression_statement(&mut self, e: &Expr<'a>) -> Result<Vec<u8>, CompileError> {
        let expr = self.accept_expr(e)?;
        let mut ib = InstructionBuilder::from(expr);
        ib.build_pop();
        Ok(ib.build())
    }

    fn visit_grouping_expression(&mut self, e: &GroupingExpr<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        for expr in &e.0 {
            ib.append(&mut self.accept_expr(expr)?);
            ib.build_pop();
        }

        ib.remove_pop_end();

        Ok(ib.build())
    }

    fn visit_literal_expression(&mut self, e: &LiteralExpr<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        ib.build_constant(&mut self.state.cp, e.into())?;
        Ok(ib.build())
    }

    fn visit_identifier_expression(&mut self, i: &'a [u8]) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        if let Some((index, _)) = self.state.scope.find_local(i) {
            ib.build_local_load(index);
        } else {
            ib.build_global_load(&mut self.state.cp, i)?;
        }

        Ok(ib.build())
    }

    fn visit_unary_expression(&mut self, e: &UnaryExpr<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        ib.append(&mut self.accept_expr(&e.expr)?);

        match e.operator {
            TokenType::Plus => ib.build_pos(),
            TokenType::Minus => ib.build_neg(),
            TokenType::Typeof => ib.build_typeof(),
            TokenType::BitwiseNot => ib.build_bitnot(),
            TokenType::LogicalNot => ib.build_not(),
            TokenType::Void => ib.build_pop(),
            _ => unimplementedc!("Unary operator {:?}", e.operator),
        }

        Ok(ib.build())
    }

    fn visit_variable_declaration(
        &mut self,
        v: &VariableDeclaration<'a>,
    ) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        let id = self.state.scope.add_local(v.binding.clone())?;

        if let Some(expr) = &v.value {
            ib.append(&mut self.accept_expr(expr)?);
            ib.build_local_store(id);
            ib.build_pop();
        }

        Ok(ib.build())
    }

    fn visit_if_statement(&mut self, i: &IfStatement<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        // Desugar last `else` block into `else if(true)` for simplicity
        if let Some(then) = &i.el {
            let then = &**then;
            let mut branches = i.branches.borrow_mut();

            branches.push(IfStatement::new(
                Expr::bool_literal(true),
                then.clone(),
                Vec::new(),
                None,
            ));
        }

        let branches = i.branches.borrow();
        let len: u16 = branches
            .len()
            .try_into()
            .map_err(|_| CompileError::IfBranchLimitExceeded)?;

        ib.append(&mut self.accept_expr(&i.condition)?);
        if branches.is_empty() {
            ib.build_jmpfalsep(Label::IfEnd)?;
        } else {
            ib.build_jmpfalsep(Label::IfBranch(0))?;
        }
        ib.append(&mut self.accept(&i.then)?);
        ib.build_jmp(Label::IfEnd)?;

        for (id, branch) in branches.iter().enumerate() {
            let id = id as u16;

            ib.add_label(Label::IfBranch(id));
            ib.append(&mut self.accept_expr(&branch.condition)?);
            if id == len - 1 {
                ib.build_jmpfalsep(Label::IfEnd)?;

                ib.append(&mut self.accept(&branch.then)?);
            } else {
                ib.build_jmpfalsep(Label::IfBranch(id + 1))?;

                ib.append(&mut self.accept(&branch.then)?);

                ib.build_jmp(Label::IfEnd)?;
            }
        }

        ib.add_label(Label::IfEnd);
        Ok(ib.build())
    }

    fn visit_block_statement(&mut self, b: &BlockStatement<'a>) -> Result<Vec<u8>, CompileError> {
        self.state.scope.enter();
        let re = self.accept_multiple(&b.0);
        self.state.scope.exit();
        re
    }

    fn visit_function_declaration(
        &mut self,
        f: &FunctionDeclaration<'a>,
    ) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        let id = self.state.scope.add_local(VariableBinding {
            name: f.name.expect("Function declaration did not have a name"),
            kind: VariableDeclarationKind::Var,
        })?;
        ib.append(&mut self.visit_function_expr(f)?);
        ib.build_local_store(id);
        ib.build_pop();
        Ok(ib.build())
    }

    fn visit_while_loop(&mut self, l: &WhileLoop<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        ib.add_label(Label::LoopCondition);
        ib.append(&mut self.accept_expr(&l.condition)?);
        ib.build_jmpfalsep(Label::LoopEnd)?;

        ib.append(&mut self.accept(&l.body)?);
        ib.build_jmp(Label::LoopCondition)?;

        ib.add_label(Label::LoopEnd);
        Ok(ib.build())
    }

    fn visit_assignment_expression(
        &mut self,
        e: &AssignmentExpr<'a>,
    ) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        match &*e.left {
            Expr::Literal(lit) => {
                let ident = lit.to_identifier();
                let ident = ident.as_bytes();

                if let Some((id, local)) = self.state.scope.find_local(ident) {
                    if matches!(local.binding().kind, VariableDeclarationKind::Const) {
                        return Err(CompileError::ConstAssignment);
                    }

                    ib.append(&mut self.accept_expr(&e.right)?);
                    ib.build_local_store(id);
                } else {
                    ib.append(&mut self.accept_expr(&e.right)?);
                    ib.build_global_store(&mut self.state.cp, ident)?;
                }
            }
            _ => unimplementedc!("Assignment to non-identifier"),
        }

        Ok(ib.build())
    }

    fn visit_function_call(&mut self, c: &FunctionCall<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        // specialize property access
        let has_this = if let Expr::PropertyAccess(p) = &*c.target {
            ib.append(&mut self.visit_property_access_expr(p, true)?);
            true
        } else {
            ib.append(&mut self.accept_expr(&c.target)?);
            false
        };

        for a in &c.arguments {
            ib.append(&mut self.accept_expr(a)?);
        }
        let argc = c
            .arguments
            .len()
            .try_into()
            .map_err(|_| CompileError::ParameterLimitExceeded)?;

        let meta = FunctionCallMetadata::new_checked(argc, c.constructor_call, has_this)
            .ok_or(CompileError::ParameterLimitExceeded)?;

        ib.build_call(meta);

        Ok(ib.build())
    }

    fn visit_return_statement(&mut self, s: &ReturnStatement<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        ib.append(&mut self.accept_expr(&s.0)?);
        ib.build_ret();
        Ok(ib.build())
    }

    fn visit_conditional_expr(&mut self, c: &ConditionalExpr<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        ib.append(&mut self.accept_expr(&c.condition)?);
        ib.build_jmpfalsep(Label::IfBranch(0))?;

        ib.append(&mut self.accept_expr(&c.then)?);
        ib.build_jmp(Label::IfEnd)?;

        ib.add_label(Label::IfBranch(0));
        ib.append(&mut self.accept_expr(&c.el)?);

        ib.add_label(Label::IfEnd);
        Ok(ib.build())
    }

    fn visit_property_access_expr(
        &mut self,
        e: &PropertyAccessExpr<'a>,
        preserve_this: bool,
    ) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        ib.append(&mut self.accept_expr(&e.target)?);
        match &*e.property {
            Expr::Literal(lit) => {
                let ident = lit.to_identifier();
                ib.build_static_prop_access(&mut self.state.cp, ident.as_bytes(), preserve_this)?;
            }
            e => {
                ib.append(&mut self.accept_expr(e)?);
                ib.build_dynamic_prop_access(preserve_this);
            }
        }

        Ok(ib.build())
    }

    fn visit_sequence_expr(&mut self, s: &Seq<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        ib.append(&mut self.accept_expr(&s.0)?);
        ib.build_pop();
        ib.append(&mut self.accept_expr(&s.1)?);

        Ok(ib.build())
    }

    fn visit_postfix_expr(&mut self, p: &Postfix<'a>) -> Result<Vec<u8>, CompileError> {
        unimplementedc!("Postfix expression {:?}", p.0)
    }

    fn visit_function_expr(
        &mut self,
        f: &FunctionDeclaration<'a>,
    ) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        let mut compiler = FunctionCompiler::with_caller(&mut self.state);
        let scope = &mut compiler.state.scope;

        for name in &f.arguments {
            scope.add_local(VariableBinding {
                kind: VariableDeclarationKind::Var,
                name,
            })?;
        }

        let cmp = compiler.compile_ast(f.statements.clone())?;

        let function = Function {
            buffer: cmp.instructions.into(),
            constants: cmp.cp.into_vec().into(),
            locals: cmp.locals,
            name: f.name.map(force_utf8),
            ty: f.ty,
            params: f.arguments.len(),
        };
        ib.build_constant(&mut self.state.cp, Constant::Function(function))?;

        Ok(ib.build())
    }

    fn visit_array_literal(&mut self, a: &ArrayLiteral<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        let len = a
            .len()
            .try_into()
            .map_err(|_| CompileError::ArrayLitLimitExceeded)?;

        for e in a.iter() {
            ib.append(&mut self.accept_expr(e)?);
        }

        ib.build_arraylit(len);
        Ok(ib.build())
    }

    fn visit_object_literal(&mut self, o: &ObjectLiteral<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        let mut idents = Vec::with_capacity(o.len());
        for (ident, value) in o {
            ib.append(&mut self.accept_expr(value)?);
            let ident = Constant::Identifier(force_utf8_borrowed(ident).into());
            idents.push(ident);
        }

        ib.build_objlit(&mut self.state.cp, idents)?;
        Ok(ib.build())
    }

    fn visit_try_catch(&mut self, t: &TryCatch<'a>) -> Result<Vec<u8>, CompileError> {
        unimplementedc!("Try catch")
    }

    fn visit_throw(&mut self, e: &Expr<'a>) -> Result<Vec<u8>, CompileError> {
        unimplementedc!("Throw statement")
    }

    fn visit_for_loop(&mut self, f: &ForLoop<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        self.state.scope.enter();

        // Initialization
        if let Some(init) = &f.init {
            ib.append(&mut self.accept(&init)?);
        }

        // Condition
        ib.add_label(Label::LoopCondition);
        if let Some(condition) = &f.condition {
            ib.append(&mut self.accept_expr(&condition)?);
            ib.build_jmpfalsep(Label::LoopEnd)?;
        }

        // Body
        ib.append(&mut self.accept(&f.body)?);

        // Increment
        if let Some(finalizer) = &f.finalizer {
            ib.append(&mut self.accept_expr(&finalizer)?);
            ib.build_pop();
        }
        ib.build_jmp(Label::LoopCondition)?;

        ib.add_label(Label::LoopEnd);
        self.state.scope.exit();

        Ok(ib.build())
    }

    fn visit_for_of_loop(&mut self, f: &ForOfLoop<'a>) -> Result<Vec<u8>, CompileError> {
        unimplementedc!("For of loop")
    }

    fn visit_import_statement(&mut self, i: &ImportKind<'a>) -> Result<Vec<u8>, CompileError> {
        unimplementedc!("Import statement")
    }

    fn visit_export_statement(&mut self, e: &ExportKind<'a>) -> Result<Vec<u8>, CompileError> {
        unimplementedc!("Export statement")
    }

    fn visit_empty_statement(&mut self) -> Result<Vec<u8>, CompileError> {
        Ok(Vec::new())
    }

    fn visit_break(&mut self) -> Result<Vec<u8>, CompileError> {
        unimplementedc!("Break statement")
    }

    fn visit_continue(&mut self) -> Result<Vec<u8>, CompileError> {
        unimplementedc!("Continue statement")
    }

    fn visit_debugger(&mut self) -> Result<Vec<u8>, CompileError> {
        unimplementedc!("Debugger statement")
    }

    fn visit_empty_expr(&mut self) -> Result<Vec<u8>, CompileError> {
        Ok(Vec::new())
    }

    fn visit_class_declaration(&mut self, c: &Class<'a>) -> Result<Vec<u8>, CompileError> {
        unimplementedc!("Class declaration")
    }
}

/// Function call metadata
///
/// Highest bit = set if constructor call
/// 2nd highest bit = set if object call
/// remaining 6 bits = number of arguments
#[repr(transparent)]
pub struct FunctionCallMetadata(u8);

impl From<u8> for FunctionCallMetadata {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<FunctionCallMetadata> for u8 {
    fn from(value: FunctionCallMetadata) -> Self {
        value.0
    }
}

impl FunctionCallMetadata {
    pub fn new_checked(mut value: u8, constructor: bool, object: bool) -> Option<Self> {
        if value & 0b11000000 == 0 {
            if constructor {
                value |= 0b10000000;
            }

            if object {
                value |= 0b01000000;
            }

            Some(Self(value))
        } else {
            None
        }
    }

    pub fn value(&self) -> u8 {
        self.0 & !0b11000000
    }

    pub fn is_constructor_call(&self) -> bool {
        self.0 & (1 << 7) != 0
    }

    pub fn is_object_call(&self) -> bool {
        self.0 & (1 << 6) != 0
    }
}
