use std::{convert::TryInto, ptr::NonNull, usize};

use strum_macros::FromRepr;

use crate::{
    compiler::builder::{InstructionBuilder, Label},
    parser::{
        expr::{
            ArrayLiteral, AssignmentExpr, BinaryExpr, ConditionalExpr, Expr, FunctionCall, GroupingExpr, LiteralExpr,
            ObjectLiteral, Postfix, PropertyAccessExpr, Seq, UnaryExpr,
        },
        statement::{
            BlockStatement, Class, ExportKind, ForLoop, ForOfLoop, FunctionDeclaration, FunctionKind, IfStatement,
            ImportKind, ReturnStatement, SpecifierKind, Statement, TryCatch, VariableBinding, VariableDeclaration,
            VariableDeclarationKind, WhileLoop,
        },
        token::TokenType,
    },
};

use self::{
    constant::{Constant, ConstantPool, Function},
    error::CompileError,
    instruction::{InstructionWriter, NamedExportKind},
    scope::{Scope, ScopeLocal},
    visitor::Visitor,
};

pub mod builder;
pub mod constant;
#[cfg(feature = "decompile")]
pub mod decompiler;
pub mod error;
pub mod instruction;
mod scope;
#[cfg(test)]
mod test;
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
    externals: Vec<u16>,
    in_try_block: bool,
    ty: FunctionKind,
}

impl<'a> SharedCompilerState<'a> {
    pub fn new(ty: FunctionKind) -> Self {
        Self {
            cp: ConstantPool::new(),
            scope: Scope::new(),
            externals: Vec::new(),
            in_try_block: false,
            ty,
        }
    }
}

pub struct FunctionCompiler<'a> {
    state: SharedCompilerState<'a>,
    caller: Option<NonNull<FunctionCompiler<'a>>>,
}

#[derive(Debug)]
pub struct CompileResult {
    pub instructions: Vec<u8>,
    pub cp: ConstantPool,
    pub locals: usize,
    pub externals: Vec<u16>,
}

fn ast_insert_return<'a>(ast: &mut Vec<Statement<'a>>) {
    match ast.last_mut() {
        Some(Statement::Return(..)) => {}
        Some(Statement::Expression(_)) => {
            let expr = match ast.pop() {
                Some(Statement::Expression(expr)) => expr,
                _ => unreachable!(),
            };

            ast.push(Statement::Return(ReturnStatement(expr)));
        }
        Some(Statement::Block(b)) => ast_insert_return(&mut b.0),
        _ => ast.push(Statement::Return(ReturnStatement::default())),
    }
}

impl<'a> FunctionCompiler<'a> {
    pub fn new() -> Self {
        Self {
            state: SharedCompilerState::new(FunctionKind::Function),
            caller: None,
        }
    }

    /// # Safety
    /// * Requires `caller` to not be invalid (i.e. due to moving) during calls
    pub unsafe fn with_caller<'s>(caller: &'s mut FunctionCompiler<'a>, ty: FunctionKind) -> Self {
        Self {
            state: SharedCompilerState::new(ty),
            caller: Some(unsafe { NonNull::new_unchecked(caller) }),
        }
    }

    pub fn compile_ast(mut self, mut ast: Vec<Statement<'a>>) -> Result<CompileResult, CompileError> {
        ast_insert_return(&mut ast);
        let instructions = self.accept_multiple(&ast)?;
        Ok(CompileResult {
            instructions,
            cp: self.state.cp,
            locals: self.state.scope.locals().len(),
            externals: self.state.externals,
        })
    }

    pub fn accept_multiple(&mut self, v: &[Statement<'a>]) -> Result<Vec<u8>, CompileError> {
        let mut insts = Vec::new();

        for stmt in v {
            insts.append(&mut self.accept(stmt)?);
        }

        Ok(insts)
    }

    fn add_external(&mut self, external_id: u16) -> usize {
        let id = self.state.externals.iter().position(|&x| x == external_id);

        match id {
            Some(id) => id,
            None => {
                self.state.externals.push(external_id);
                self.state.externals.len() - 1
            }
        }
    }

    /// Tries to find a local in the current or surrounding scopes
    ///
    /// If a local variable is found in a parent scope, it is marked as an extern local
    pub fn find_local(&mut self, ident: &str) -> Option<(u16, ScopeLocal<'a>)> {
        if let Some((id, local)) = self.state.scope.find_local(ident) {
            Some((id, local.clone()))
        } else {
            let mut caller = self.caller;

            while let Some(mut up) = caller {
                let this = unsafe { up.as_mut() };

                if let Some((id, local)) = this.find_local(ident) {
                    // TODO: handle this case
                    assert!(!local.is_extern());

                    // Extern values need to be marked as such
                    local.set_extern();

                    // TODO: don't hardcast
                    let id = self.add_external(id) as u16;
                    return Some((id, local.clone()));
                }

                caller = this.caller;
            }

            None
        }
    }
}

impl<'a> Visitor<'a, Result<Vec<u8>, CompileError>> for FunctionCompiler<'a> {
    fn visit_binary_expression(&mut self, e: &BinaryExpr<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        ib.append(&mut self.accept_expr(&e.left)?);

        macro_rules! trivial_case {
            ($k:expr) => {{
                ib.append(&mut self.accept_expr(&e.right)?);
                $k(&mut ib)
            }};
        }

        match e.operator {
            TokenType::Plus => trivial_case!(InstructionBuilder::build_add),
            TokenType::Minus => trivial_case!(InstructionBuilder::build_sub),
            TokenType::Star => trivial_case!(InstructionBuilder::build_mul),
            TokenType::Slash => trivial_case!(InstructionBuilder::build_div),
            TokenType::Remainder => trivial_case!(InstructionBuilder::build_rem),
            TokenType::Exponentiation => trivial_case!(InstructionBuilder::build_pow),
            TokenType::Greater => trivial_case!(InstructionBuilder::build_gt),
            TokenType::GreaterEqual => trivial_case!(InstructionBuilder::build_ge),
            TokenType::Less => trivial_case!(InstructionBuilder::build_lt),
            TokenType::LessEqual => trivial_case!(InstructionBuilder::build_le),
            TokenType::Equality => trivial_case!(InstructionBuilder::build_eq),
            TokenType::Inequality => trivial_case!(InstructionBuilder::build_ne),
            TokenType::StrictEquality => trivial_case!(InstructionBuilder::build_strict_eq),
            TokenType::StrictInequality => trivial_case!(InstructionBuilder::build_strict_ne),
            TokenType::BitwiseOr => trivial_case!(InstructionBuilder::build_bitor),
            TokenType::BitwiseXor => trivial_case!(InstructionBuilder::build_bitxor),
            TokenType::BitwiseAnd => trivial_case!(InstructionBuilder::build_bitand),
            TokenType::LeftShift => trivial_case!(InstructionBuilder::build_bitshl),
            TokenType::RightShift => trivial_case!(InstructionBuilder::build_bitshr),
            TokenType::UnsignedRightShift => trivial_case!(InstructionBuilder::build_bitushr),
            TokenType::In => trivial_case!(InstructionBuilder::build_objin),
            TokenType::Instanceof => trivial_case!(InstructionBuilder::build_instanceof),
            TokenType::LogicalOr => {
                ib.build_jmptruenp(Label::IfEnd);
                ib.build_pop(); // Only pop LHS if it is false
                ib.append(&mut self.accept_expr(&e.right)?);
                ib.add_label(Label::IfEnd);
            }
            TokenType::LogicalAnd => {
                ib.build_jmpfalsenp(Label::IfEnd);
                ib.build_pop(); // Only pop LHS if it is true
                ib.append(&mut self.accept_expr(&e.right)?);
                ib.add_label(Label::IfEnd);
            }
            TokenType::NullishCoalescing => {
                ib.build_jmpnullishnp(Label::IfBranch(0));
                ib.build_jmp(Label::IfEnd);

                ib.add_label(Label::IfBranch(0));
                ib.build_pop();
                ib.append(&mut self.accept_expr(&e.right)?);
                ib.add_label(Label::IfEnd);
            }
            other => unimplementedc!("Binary operator {:?}", other),
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

    fn visit_identifier_expression(&mut self, i: &str) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        match self.find_local(i) {
            Some((index, local)) => ib.build_local_load(index, local.is_extern()),
            _ => ib.build_global_load(&mut self.state.cp, i)?,
        };

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
            TokenType::Yield => {
                if !matches!(self.state.ty, FunctionKind::Generator) {
                    return Err(CompileError::YieldOutsideGenerator);
                }

                ib.build_yield();
            }
            _ => unimplementedc!("Unary operator {:?}", e.operator),
        }

        Ok(ib.build())
    }

    fn visit_variable_declaration(&mut self, v: &VariableDeclaration<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        let id = self.state.scope.add_local(v.binding.clone(), false)?;

        if let Some(expr) = &v.value {
            ib.append(&mut self.accept_expr(expr)?);
            ib.build_local_store(id, false);
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
            ib.build_jmpfalsep(Label::IfEnd);
        } else {
            ib.build_jmpfalsep(Label::IfBranch(0));
        }
        ib.append(&mut self.accept(&i.then)?);
        ib.build_jmp(Label::IfEnd);

        for (id, branch) in branches.iter().enumerate() {
            let id = id as u16;

            ib.add_label(Label::IfBranch(id));
            ib.append(&mut self.accept_expr(&branch.condition)?);
            if id == len - 1 {
                ib.build_jmpfalsep(Label::IfEnd);

                ib.append(&mut self.accept(&branch.then)?);
            } else {
                ib.build_jmpfalsep(Label::IfBranch(id + 1));

                ib.append(&mut self.accept(&branch.then)?);

                ib.build_jmp(Label::IfEnd);
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

    fn visit_function_declaration(&mut self, f: &FunctionDeclaration<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        let id = self.state.scope.add_local(
            VariableBinding {
                name: f.name.expect("Function declaration did not have a name"),
                kind: VariableDeclarationKind::Var,
            },
            false,
        )?;
        ib.append(&mut self.visit_function_expr(f)?);
        ib.build_local_store(id, false);
        ib.build_pop();
        Ok(ib.build())
    }

    fn visit_while_loop(&mut self, l: &WhileLoop<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        ib.add_label(Label::LoopCondition);
        ib.append(&mut self.accept_expr(&l.condition)?);
        ib.build_jmpfalsep(Label::LoopEnd);

        ib.append(&mut self.accept(&l.body)?);
        ib.build_jmp(Label::LoopCondition);

        ib.add_label(Label::LoopEnd);
        Ok(ib.build())
    }

    fn visit_assignment_expression(&mut self, e: &AssignmentExpr<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        if let Expr::PropertyAccess(prop) = &*e.left {
            ib.append(&mut self.accept_expr(&prop.target)?);
        }

        ib.append(&mut self.accept_expr(&e.right)?);

        match &*e.left {
            Expr::Literal(lit) => {
                let ident = lit.to_identifier();

                if let Some((id, local)) = self.find_local(&ident) {
                    if matches!(local.binding().kind, VariableDeclarationKind::Const) {
                        return Err(CompileError::ConstAssignment);
                    }

                    let is_extern = local.is_extern();

                    match e.operator {
                        TokenType::Assignment => {}
                        TokenType::AdditionAssignment => {
                            ib.build_local_load(id, is_extern);
                            ib.build_add();
                        }
                        TokenType::SubtractionAssignment => {
                            ib.build_local_load(id, is_extern);
                            ib.build_sub();
                        }
                        _ => unimplementedc!("Unknown operator"),
                    }

                    ib.build_local_store(id, is_extern);
                } else {
                    match e.operator {
                        TokenType::Assignment => {}
                        TokenType::AdditionAssignment => {
                            ib.build_global_load(&mut self.state.cp, &ident)?;
                            ib.build_add();
                        }
                        TokenType::SubtractionAssignment => {
                            ib.build_global_load(&mut self.state.cp, &ident)?;
                            ib.build_sub();
                        }
                        _ => unimplementedc!("Unknown operator"),
                    }

                    ib.build_global_store(&mut self.state.cp, &ident)?;
                }
            }
            Expr::PropertyAccess(prop) => {
                if !matches!(e.operator, TokenType::Assignment) {
                    unimplementedc!("Assignment operator {:?}", e.operator);
                }

                match (&*prop.property, prop.computed) {
                    (Expr::Literal(lit), false) => {
                        let ident = lit.to_identifier();
                        ib.build_static_prop_set(&mut self.state.cp, &ident)?;
                    }
                    (e, _) => {
                        ib.append(&mut self.accept_expr(&e)?);
                        ib.build_dynamic_prop_set();
                    }
                }
            }
            _ => unimplementedc!("Assignment to non-identifier"),
        }

        Ok(ib.build())
    }

    fn visit_function_call(&mut self, c: &FunctionCall<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        // specialize property access
        // TODO: this also needs to be specialized for assignment expressions with property access as target
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
        ib.build_jmpfalsep(Label::IfBranch(0));

        ib.append(&mut self.accept_expr(&c.then)?);
        ib.build_jmp(Label::IfEnd);

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
        match (&*e.property, e.computed) {
            (Expr::Literal(lit), false) => {
                let ident = lit.to_identifier();
                ib.build_static_prop_access(&mut self.state.cp, &ident, preserve_this)?;
            }
            (e, _) => {
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
        let mut ib = InstructionBuilder::new();

        match &*p.1 {
            Expr::Literal(lit) => {
                let ident = lit.to_identifier();
                if let Some((id, loc)) = self.find_local(&ident) {
                    ib.build_local_load(id, loc.is_extern());
                } else {
                    unimplementedc!("Global postfix expression");
                }
            }
            _ => unimplementedc!("Non-identifier postfix expression"),
        }

        let mut desugar = self.visit_assignment_expression(&AssignmentExpr {
            left: p.1.clone(),
            operator: match p.0 {
                TokenType::Increment => TokenType::AdditionAssignment,
                TokenType::Decrement => TokenType::SubtractionAssignment,
                _ => unreachable!("Token never emitted"),
            },
            right: Box::new(Expr::number_literal(1.0)),
        })?;

        ib.append(&mut desugar);
        ib.build_pop();

        Ok(ib.build())
    }

    fn visit_function_expr(&mut self, f: &FunctionDeclaration<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        let mut compiler = unsafe { FunctionCompiler::with_caller(self, f.ty) };
        let scope = &mut compiler.state.scope;

        for name in &f.arguments {
            scope.add_local(
                VariableBinding {
                    kind: VariableDeclarationKind::Var,
                    name,
                },
                false,
            )?;
        }

        let cmp = compiler.compile_ast(f.statements.clone())?;

        let function = Function {
            buffer: cmp.instructions.into(),
            constants: cmp.cp.into_vec().into(),
            locals: cmp.locals,
            name: f.name.map(ToOwned::to_owned),
            ty: f.ty,
            params: f.arguments.len(),
            externals: cmp.externals.into(),
        };
        ib.build_constant(&mut self.state.cp, Constant::Function(function))?;

        Ok(ib.build())
    }

    fn visit_array_literal(&mut self, ArrayLiteral(a): &ArrayLiteral<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        let len = a.len().try_into().map_err(|_| CompileError::ArrayLitLimitExceeded)?;

        for e in a.iter() {
            ib.append(&mut self.accept_expr(e)?);
        }

        ib.build_arraylit(len);
        Ok(ib.build())
    }

    fn visit_object_literal(&mut self, ObjectLiteral(o): &ObjectLiteral<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        let mut idents = Vec::with_capacity(o.len());
        for (ident, value) in o {
            ib.append(&mut self.accept_expr(value)?);
            let ident = Constant::Identifier((*ident).into());
            idents.push(ident);
        }

        ib.build_objlit(&mut self.state.cp, idents)?;
        Ok(ib.build())
    }

    fn visit_try_catch(&mut self, t: &TryCatch<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        ib.build_try_block();

        self.state.scope.enter();
        ib.append(&mut self.accept(&t.try_)?);
        self.state.scope.exit();

        ib.build_jmp(Label::TryEnd);

        ib.add_label(Label::Catch);

        self.state.scope.enter();

        if let Some(ident) = t.catch.ident {
            let id = self.state.scope.add_local(
                VariableBinding {
                    kind: VariableDeclarationKind::Var,
                    name: ident,
                },
                false,
            )?;

            if id == u16::MAX {
                // Max u16 value is reserved for "no binding"
                return Err(CompileError::LocalLimitExceeded);
            }

            ib.writew(id);
        } else {
            ib.writew(u16::MAX);
        }

        ib.append(&mut self.accept(&t.catch.body)?);
        self.state.scope.exit();

        ib.add_label(Label::TryEnd);
        ib.build_try_end();

        Ok(ib.build())
    }

    fn visit_throw(&mut self, e: &Expr<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();
        ib.append(&mut self.accept_expr(e)?);
        ib.build_throw();
        Ok(ib.build())
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
            ib.build_jmpfalsep(Label::LoopEnd);
        }

        // Body
        ib.append(&mut self.accept(&f.body)?);

        // Increment
        if let Some(finalizer) = &f.finalizer {
            ib.append(&mut self.accept_expr(&finalizer)?);
            ib.build_pop();
        }
        ib.build_jmp(Label::LoopCondition);

        ib.add_label(Label::LoopEnd);
        self.state.scope.exit();

        Ok(ib.build())
    }

    fn visit_for_of_loop(&mut self, f: &ForOfLoop<'a>) -> Result<Vec<u8>, CompileError> {
        unimplementedc!("For of loop")
    }

    fn visit_import_statement(&mut self, import: &ImportKind<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        match import {
            ImportKind::Dynamic(ex) => {
                ib.append(&mut self.accept_expr(ex)?);
                ib.build_dynamic_import();
            }
            ImportKind::DefaultAs(spec, path) | ImportKind::AllAs(spec, path) => {
                let ident = match spec {
                    SpecifierKind::Ident(id) => id,
                };

                let local_id = self.state.scope.add_local(
                    VariableBinding {
                        kind: VariableDeclarationKind::Var,
                        name: ident,
                    },
                    false,
                )?;

                let path_id = self.state.cp.add(Constant::String((*path).into()))?;

                ib.build_static_import(import, local_id, path_id);
            }
        }

        Ok(ib.build())
    }

    fn visit_export_statement(&mut self, e: &ExportKind<'a>) -> Result<Vec<u8>, CompileError> {
        let mut ib = InstructionBuilder::new();

        match e {
            ExportKind::Default(expr) => {
                ib.append(&mut self.accept_expr(expr)?);
                ib.build_default_export();
            }
            ExportKind::Named(names) => {
                let mut it = Vec::with_capacity(names.len());

                for name in names.iter().copied() {
                    let ident_id = self.state.cp.add(Constant::Identifier(name.into()))?;

                    match self.find_local(name) {
                        Some((loc_id, loc)) => {
                            // Top level exports shouldn't be able to refer to extern locals
                            assert!(!loc.is_extern());

                            it.push(NamedExportKind::Local { loc_id, ident_id });
                        }
                        None => {
                            it.push(NamedExportKind::Global { ident_id });
                        }
                    }
                }

                ib.build_named_export(&it)?;
            }
            ExportKind::NamedVar(vars) => {
                for var in vars.iter() {
                    ib.append(&mut self.visit_variable_declaration(var)?);
                }

                let it = vars.iter().map(|var| var.binding.name).collect::<Vec<_>>();

                ib.append(&mut self.visit_export_statement(&ExportKind::Named(it))?);
            }
        };
        Ok(ib.build())
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

#[repr(u8)]
#[derive(FromRepr)]
pub enum StaticImportKind {
    All,
    Default,
}
