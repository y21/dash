use std::{convert::TryInto, ptr::NonNull, usize};

use dash_middle::compiler::constant::{Constant, Function};
use dash_middle::compiler::{constant::ConstantPool, external::External};
use dash_middle::compiler::{CompileResult, FunctionCallMetadata};
use dash_middle::lexer::token::TokenType;
use dash_middle::parser::expr::ArrayLiteral;
use dash_middle::parser::expr::AssignmentExpr;
use dash_middle::parser::expr::BinaryExpr;
use dash_middle::parser::expr::ConditionalExpr;
use dash_middle::parser::expr::Expr;
use dash_middle::parser::expr::FunctionCall;
use dash_middle::parser::expr::GroupingExpr;
use dash_middle::parser::expr::LiteralExpr;
use dash_middle::parser::expr::ObjectLiteral;
use dash_middle::parser::expr::Postfix;
use dash_middle::parser::expr::PropertyAccessExpr;
use dash_middle::parser::expr::Seq;
use dash_middle::parser::expr::UnaryExpr;
use dash_middle::parser::statement::BlockStatement;
use dash_middle::parser::statement::Class;
use dash_middle::parser::statement::ExportKind;
use dash_middle::parser::statement::ForLoop;
use dash_middle::parser::statement::ForOfLoop;
use dash_middle::parser::statement::FunctionDeclaration;
use dash_middle::parser::statement::FunctionKind;
use dash_middle::parser::statement::IfStatement;
use dash_middle::parser::statement::ImportKind;
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::SpecifierKind;
use dash_middle::parser::statement::Statement;
use dash_middle::parser::statement::TryCatch;
use dash_middle::parser::statement::VariableBinding;
use dash_middle::parser::statement::VariableDeclaration;
use dash_middle::parser::statement::VariableDeclarationKind;
use dash_middle::parser::statement::WhileLoop;

use crate::builder::{InstructionBuilder, Label};

use self::{
    error::CompileError,
    instruction::{InstructionWriter, NamedExportKind},
    scope::{Scope, ScopeLocal},
    visitor::Visitor,
};

pub mod builder;
#[cfg(feature = "decompile")]
pub mod decompiler;
pub mod error;
#[cfg(feature = "from_string")]
pub mod from_string;
pub mod instruction;
mod scope;
// #[cfg(test)]
// mod test;
/// Visitor trait, used to walk the AST
mod visitor;

macro_rules! unimplementedc {
    ($($what:expr),*) => {
        return Err(CompileError::Unimplemented(format_args!($($what),*).to_string()))
    };
}

pub struct FunctionCompiler<'a> {
    ib: InstructionBuilder,
    cp: ConstantPool,
    scope: Scope<'a>,
    externals: Vec<External>,
    try_catch_depth: u16,
    ty: FunctionKind,
    caller: Option<NonNull<FunctionCompiler<'a>>>,
}

/// Implicitly inserts a `return` statement for the last expression
fn ast_insert_return<'a>(ast: &mut Vec<Statement<'a>>) {
    match ast.last_mut() {
        Some(Statement::Return(..)) => {}
        Some(Statement::Expression(..)) => {
            let expr = match ast.pop() {
                Some(Statement::Expression(expr)) => expr,
                _ => unreachable!(),
            };

            ast.push(Statement::Return(ReturnStatement(expr)));
        }
        Some(Statement::Block(BlockStatement(block))) => ast_insert_return(block),
        _ => ast.push(Statement::Return(ReturnStatement::default())),
    }
}

impl<'a> FunctionCompiler<'a> {
    pub fn new() -> Self {
        Self {
            ib: InstructionBuilder::new(),
            cp: ConstantPool::new(),
            scope: Scope::new(),
            externals: Vec::new(),
            try_catch_depth: 0,
            ty: FunctionKind::Function,
            caller: None,
        }
    }

    /// # Safety
    /// * Requires `caller` to not be invalid (i.e. due to moving) during calls
    pub unsafe fn with_caller<'s>(caller: &'s mut FunctionCompiler<'a>, ty: FunctionKind) -> Self {
        Self {
            ib: InstructionBuilder::new(),
            cp: ConstantPool::new(),
            scope: Scope::new(),
            externals: Vec::new(),
            try_catch_depth: 0,
            ty,
            caller: Some(NonNull::new(caller).unwrap()),
        }
    }

    pub fn compile_ast(
        mut self,
        mut ast: Vec<Statement<'a>>,
        implicit_return: bool,
    ) -> Result<CompileResult, CompileError> {
        if implicit_return {
            ast_insert_return(&mut ast);
        } else {
            // Push an implicit `return undefined;` statement at the end in case there is not already an explicit one
            ast.push(Statement::Return(Default::default()));
        }

        self.accept_multiple(&ast)?;
        Ok(CompileResult {
            instructions: self.ib.build(),
            cp: self.cp,
            locals: self.scope.locals().len(),
            externals: self.externals,
        })
    }

    pub fn accept_multiple(&mut self, v: &[Statement<'a>]) -> Result<(), CompileError> {
        for stmt in v {
            self.accept(stmt)?;
        }
        Ok(())
    }

    fn add_external(&mut self, external_id: u16, is_nested_external: bool) -> usize {
        let id = self.externals.iter().position(|External { id, .. }| *id == external_id);

        match id {
            Some(id) => id,
            None => {
                self.externals.push(External {
                    id: external_id,
                    is_external: is_nested_external,
                });
                self.externals.len() - 1
            }
        }
    }

    /// Tries to find a local in the current or surrounding scopes
    ///
    /// If a local variable is found in a parent scope, it is marked as an extern local
    pub fn find_local(&mut self, ident: &str) -> Option<(u16, ScopeLocal<'a>)> {
        if let Some((id, local)) = self.scope.find_local(ident) {
            Some((id, local.clone()))
        } else {
            let mut caller = self.caller;

            while let Some(mut up) = caller {
                let this = unsafe { up.as_mut() };

                if let Some((id, local)) = this.find_local(ident) {
                    // If the local found in a parent scope is already an external,
                    // it needs to be resolved differently at runtime
                    let is_nested_extern = local.is_extern();

                    // If it's not already marked external, mark it as such
                    local.set_extern();

                    // TODO: don't hardcast
                    let id = self.add_external(id, is_nested_extern) as u16;
                    return Some((id, local.clone()));
                }

                caller = this.caller;
            }

            None
        }
    }
}

impl<'a> Visitor<'a, Result<(), CompileError>> for FunctionCompiler<'a> {
    fn visit_binary_expression(&mut self, e: &BinaryExpr<'a>) -> Result<(), CompileError> {
        self.accept_expr(&e.left)?;

        macro_rules! trivial_case {
            ($k:expr) => {{
                self.accept_expr(&e.right)?;
                $k(&mut self.ib)
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
                self.ib.build_jmptruenp(Label::IfEnd);
                self.ib.build_pop(); // Only pop LHS if it is false
                self.accept_expr(&e.right)?;
                self.ib.add_label(Label::IfEnd);
            }
            TokenType::LogicalAnd => {
                self.ib.build_jmpfalsenp(Label::IfEnd);
                self.ib.build_pop(); // Only pop LHS if it is true
                self.accept_expr(&e.right)?;
                self.ib.add_label(Label::IfEnd);
            }
            TokenType::NullishCoalescing => {
                self.ib.build_jmpnullishnp(Label::IfBranch(0));
                self.ib.build_jmp(Label::IfEnd);

                self.ib.add_label(Label::IfBranch(0));
                self.ib.build_pop();
                self.accept_expr(&e.right)?;
                self.ib.add_label(Label::IfEnd);
            }
            other => unimplementedc!("Binary operator {:?}", other),
        }

        Ok(())
    }

    fn visit_expression_statement(&mut self, e: &Expr<'a>) -> Result<(), CompileError> {
        self.accept_expr(e)?;
        self.ib.build_pop();
        Ok(())
    }

    fn visit_grouping_expression(&mut self, e: &GroupingExpr<'a>) -> Result<(), CompileError> {
        for expr in &e.0 {
            self.accept_expr(expr)?;
            self.ib.build_pop();
        }

        self.ib.remove_pop_end();

        Ok(())
    }

    fn visit_literal_expression(&mut self, e: &LiteralExpr<'a>) -> Result<(), CompileError> {
        self.ib.build_constant(&mut self.cp, Constant::from_literal(e))?;
        Ok(())
    }

    fn visit_identifier_expression(&mut self, ident: &str) -> Result<(), CompileError> {
        match ident {
            "this" => self.ib.build_this(),
            "super" => self.ib.build_super(),
            "globalThis" => self.ib.build_global(),
            ident => match self.find_local(ident) {
                Some((index, local)) => self.ib.build_local_load(index, local.is_extern()),
                _ => self.ib.build_global_load(&mut self.cp, ident)?,
            },
        };

        Ok(())
    }

    fn visit_unary_expression(&mut self, e: &UnaryExpr<'a>) -> Result<(), CompileError> {
        self.accept_expr(&e.expr)?;

        match e.operator {
            TokenType::Plus => self.ib.build_pos(),
            TokenType::Minus => self.ib.build_neg(),
            TokenType::Typeof => self.ib.build_typeof(),
            TokenType::BitwiseNot => self.ib.build_bitnot(),
            TokenType::LogicalNot => self.ib.build_not(),
            TokenType::Void => {
                self.ib.build_pop();
                self.ib.build_undef();
            }
            TokenType::Yield => {
                if !matches!(self.ty, FunctionKind::Generator) {
                    return Err(CompileError::YieldOutsideGenerator);
                }

                self.ib.build_yield();
            }
            _ => unimplementedc!("Unary operator {:?}", e.operator),
        }

        Ok(())
    }

    fn visit_variable_declaration(&mut self, v: &VariableDeclaration<'a>) -> Result<(), CompileError> {
        let id = self.scope.add_local(v.binding.clone(), false)?;

        if let Some(expr) = &v.value {
            self.accept_expr(expr)?;
            self.ib.build_local_store(id, false);
            self.ib.build_pop();
        }

        Ok(())
    }

    fn visit_if_statement(&mut self, i: &IfStatement<'a>) -> Result<(), CompileError> {
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

        self.accept_expr(&i.condition)?;
        if branches.is_empty() {
            self.ib.build_jmpfalsep(Label::IfEnd);
        } else {
            self.ib.build_jmpfalsep(Label::IfBranch(0));
        }
        self.accept(&i.then)?;
        self.ib.build_jmp(Label::IfEnd);

        for (id, branch) in branches.iter().enumerate() {
            let id = id as u16;

            self.ib.add_label(Label::IfBranch(id));
            self.accept_expr(&branch.condition)?;
            if id == len - 1 {
                self.ib.build_jmpfalsep(Label::IfEnd);

                self.accept(&branch.then)?;
            } else {
                self.ib.build_jmpfalsep(Label::IfBranch(id + 1));

                self.accept(&branch.then)?;

                self.ib.build_jmp(Label::IfEnd);
            }
        }

        self.ib.add_label(Label::IfEnd);
        Ok(())
    }

    fn visit_block_statement(&mut self, b: &BlockStatement<'a>) -> Result<(), CompileError> {
        self.scope.enter();
        let re = self.accept_multiple(&b.0);
        self.scope.exit();
        re
    }

    fn visit_function_declaration(&mut self, f: &FunctionDeclaration<'a>) -> Result<(), CompileError> {
        let id = self.scope.add_local(
            VariableBinding {
                name: f.name.expect("Function declaration did not have a name"),
                kind: VariableDeclarationKind::Var,
            },
            false,
        )?;
        self.visit_function_expr(f)?;
        self.ib.build_local_store(id, false);
        self.ib.build_pop();
        Ok(())
    }

    fn visit_while_loop(&mut self, l: &WhileLoop<'a>) -> Result<(), CompileError> {
        self.ib.add_label(Label::LoopCondition);
        self.accept_expr(&l.condition)?;
        self.ib.build_jmpfalsep(Label::LoopEnd);

        self.accept(&l.body)?;
        self.ib.build_jmp(Label::LoopCondition);

        self.ib.add_label(Label::LoopEnd);
        Ok(())
    }

    fn visit_assignment_expression(&mut self, e: &AssignmentExpr<'a>) -> Result<(), CompileError> {
        if let Expr::PropertyAccess(prop) = &*e.left {
            self.accept_expr(&prop.target)?;
        }

        self.accept_expr(&e.right)?;

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
                            self.ib.build_local_load(id, is_extern);
                            // += requires reversing (right, left)
                            // we effectively need to rewrite it from
                            // left = right + left
                            // to
                            // left = left + right
                            self.ib.build_revstck(2);
                            self.ib.build_add();
                        }
                        TokenType::SubtractionAssignment => {
                            self.ib.build_local_load(id, is_extern);
                            self.ib.build_revstck(2);
                            self.ib.build_sub();
                        }
                        _ => unimplementedc!("Unknown operator"),
                    }

                    self.ib.build_local_store(id, is_extern);
                } else {
                    match e.operator {
                        TokenType::Assignment => {}
                        TokenType::AdditionAssignment => {
                            self.ib.build_global_load(&mut self.cp, &ident)?;
                            self.ib.build_add();
                        }
                        TokenType::SubtractionAssignment => {
                            self.ib.build_global_load(&mut self.cp, &ident)?;
                            self.ib.build_sub();
                        }
                        _ => unimplementedc!("Unknown operator"),
                    }

                    self.ib.build_global_store(&mut self.cp, &ident)?;
                }
            }
            Expr::PropertyAccess(prop) => {
                if !matches!(e.operator, TokenType::Assignment) {
                    unimplementedc!("Assignment operator {:?}", e.operator);
                }

                match (&*prop.property, prop.computed) {
                    (Expr::Literal(lit), false) => {
                        let ident = lit.to_identifier();
                        self.ib.build_static_prop_set(&mut self.cp, &ident)?;
                    }
                    (e, _) => {
                        self.accept_expr(&e)?;
                        self.ib.build_dynamic_prop_set();
                    }
                }
            }
            _ => unimplementedc!("Assignment to non-identifier"),
        }

        Ok(())
    }

    fn visit_function_call(&mut self, c: &FunctionCall<'a>) -> Result<(), CompileError> {
        // specialize property access
        // TODO: this also needs to be specialized for assignment expressions with property access as target
        let has_this = if let Expr::PropertyAccess(p) = &*c.target {
            self.visit_property_access_expr(p, true)?;
            true
        } else {
            self.accept_expr(&c.target)?;
            false
        };

        for a in &c.arguments {
            self.accept_expr(a)?;
        }
        let argc = c
            .arguments
            .len()
            .try_into()
            .map_err(|_| CompileError::ParameterLimitExceeded)?;

        let meta = FunctionCallMetadata::new_checked(argc, c.constructor_call, has_this)
            .ok_or(CompileError::ParameterLimitExceeded)?;

        self.ib.build_call(meta);

        Ok(())
    }

    fn visit_return_statement(&mut self, s: &ReturnStatement<'a>) -> Result<(), CompileError> {
        self.accept_expr(&s.0)?;
        self.ib.build_ret(self.try_catch_depth);
        Ok(())
    }

    fn visit_conditional_expr(&mut self, c: &ConditionalExpr<'a>) -> Result<(), CompileError> {
        self.accept_expr(&c.condition)?;
        self.ib.build_jmpfalsep(Label::IfBranch(0));

        self.accept_expr(&c.then)?;
        self.ib.build_jmp(Label::IfEnd);

        self.ib.add_label(Label::IfBranch(0));
        self.accept_expr(&c.el)?;

        self.ib.add_label(Label::IfEnd);
        Ok(())
    }

    fn visit_property_access_expr(
        &mut self,
        e: &PropertyAccessExpr<'a>,
        preserve_this: bool,
    ) -> Result<(), CompileError> {
        self.accept_expr(&e.target)?;

        match (&*e.property, e.computed) {
            (Expr::Literal(lit), false) => {
                let ident = lit.to_identifier();
                self.ib.build_static_prop_access(&mut self.cp, &ident, preserve_this)?;
            }
            (e, _) => {
                self.accept_expr(e)?;
                self.ib.build_dynamic_prop_access(preserve_this);
            }
        }

        Ok(())
    }

    fn visit_sequence_expr(&mut self, s: &Seq<'a>) -> Result<(), CompileError> {
        self.accept_expr(&s.0)?;
        self.ib.build_pop();
        self.accept_expr(&s.1)?;

        Ok(())
    }

    fn visit_postfix_expr(&mut self, p: &Postfix<'a>) -> Result<(), CompileError> {
        match &*p.1 {
            Expr::Literal(lit) => {
                let ident = lit.to_identifier();

                if let Some((id, loc)) = self.find_local(&ident) {
                    self.ib.build_local_load(id, loc.is_extern());
                } else {
                    unimplementedc!("Global postfix expression");
                }
            }
            _ => unimplementedc!("Non-identifier postfix expression"),
        }

        self.visit_assignment_expression(&AssignmentExpr {
            left: p.1.clone(),
            operator: match p.0 {
                TokenType::Increment => TokenType::AdditionAssignment,
                TokenType::Decrement => TokenType::SubtractionAssignment,
                _ => unreachable!("Token never emitted"),
            },
            right: Box::new(Expr::number_literal(1.0)),
        })?;
        self.ib.build_pop();

        Ok(())
    }

    fn visit_function_expr(&mut self, f: &FunctionDeclaration<'a>) -> Result<(), CompileError> {
        let mut compiler = unsafe { FunctionCompiler::with_caller(self, f.ty) };
        let scope = &mut compiler.scope;

        for name in &f.arguments {
            scope.add_local(
                VariableBinding {
                    kind: VariableDeclarationKind::Var,
                    name,
                },
                false,
            )?;
        }

        let cmp = compiler.compile_ast(f.statements.clone(), false)?;

        let function = Function {
            buffer: cmp.instructions.into(),
            constants: cmp.cp.into_vec().into(),
            locals: cmp.locals,
            name: f.name.map(ToOwned::to_owned),
            ty: f.ty,
            params: f.arguments.len(),
            externals: cmp.externals.into(),
        };
        self.ib.build_constant(&mut self.cp, Constant::Function(function))?;

        Ok(())
    }

    fn visit_array_literal(&mut self, ArrayLiteral(a): &ArrayLiteral<'a>) -> Result<(), CompileError> {
        let len = a.len().try_into().map_err(|_| CompileError::ArrayLitLimitExceeded)?;

        for e in a.iter() {
            self.accept_expr(e)?;
        }

        self.ib.build_arraylit(len);
        Ok(())
    }

    fn visit_object_literal(&mut self, ObjectLiteral(o): &ObjectLiteral<'a>) -> Result<(), CompileError> {
        let mut idents = Vec::with_capacity(o.len());
        for (ident, value) in o {
            self.accept_expr(value)?;
            let ident = Constant::Identifier((*ident).into());
            idents.push(ident);
        }

        self.ib.build_objlit(&mut self.cp, idents)?;
        Ok(())
    }

    fn visit_try_catch(&mut self, t: &TryCatch<'a>) -> Result<(), CompileError> {
        self.ib.build_try_block();

        self.try_catch_depth += 1;
        self.scope.enter();
        self.accept(&t.try_)?;
        self.scope.exit();
        self.try_catch_depth -= 1;

        self.ib.build_jmp(Label::TryEnd);

        self.ib.add_label(Label::Catch);

        self.scope.enter();

        if let Some(ident) = t.catch.ident {
            let id = self.scope.add_local(
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

            self.ib.writew(id);
        } else {
            self.ib.writew(u16::MAX);
        }

        self.accept(&t.catch.body)?;
        self.scope.exit();

        self.ib.add_label(Label::TryEnd);
        self.ib.build_try_end();

        Ok(())
    }

    fn visit_throw(&mut self, e: &Expr<'a>) -> Result<(), CompileError> {
        self.accept_expr(e)?;
        self.ib.build_throw();
        Ok(())
    }

    fn visit_for_loop(&mut self, f: &ForLoop<'a>) -> Result<(), CompileError> {
        self.scope.enter();

        // Initialization
        if let Some(init) = &f.init {
            self.accept(&init)?;
        }

        // Condition
        self.ib.add_label(Label::LoopCondition);
        if let Some(condition) = &f.condition {
            self.accept_expr(&condition)?;
            self.ib.build_jmpfalsep(Label::LoopEnd);
        }

        // Body
        self.accept(&f.body)?;

        // Increment
        if let Some(finalizer) = &f.finalizer {
            self.accept_expr(&finalizer)?;
            self.ib.build_pop();
        }
        self.ib.build_jmp(Label::LoopCondition);

        self.ib.add_label(Label::LoopEnd);
        self.scope.exit();

        Ok(())
    }

    fn visit_for_of_loop(&mut self, _f: &ForOfLoop<'a>) -> Result<(), CompileError> {
        unimplementedc!("For of loop")
    }

    fn visit_import_statement(&mut self, import: &ImportKind<'a>) -> Result<(), CompileError> {
        match import {
            ImportKind::Dynamic(ex) => {
                self.accept_expr(ex)?;
                self.ib.build_dynamic_import();
            }
            ImportKind::DefaultAs(spec, path) | ImportKind::AllAs(spec, path) => {
                let ident = match spec {
                    SpecifierKind::Ident(id) => id,
                };

                let local_id = self.scope.add_local(
                    VariableBinding {
                        kind: VariableDeclarationKind::Var,
                        name: ident,
                    },
                    false,
                )?;

                let path_id = self.cp.add(Constant::String((*path).into()))?;

                self.ib.build_static_import(import, local_id, path_id);
            }
        }

        Ok(())
    }

    fn visit_export_statement(&mut self, e: &ExportKind<'a>) -> Result<(), CompileError> {
        match e {
            ExportKind::Default(expr) => {
                self.accept_expr(expr)?;
                self.ib.build_default_export();
            }
            ExportKind::Named(names) => {
                let mut it = Vec::with_capacity(names.len());

                for name in names.iter().copied() {
                    let ident_id = self.cp.add(Constant::Identifier(name.into()))?;

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

                self.ib.build_named_export(&it)?;
            }
            ExportKind::NamedVar(vars) => {
                for var in vars.iter() {
                    self.visit_variable_declaration(var)?;
                }

                let it = vars.iter().map(|var| var.binding.name).collect::<Vec<_>>();

                self.visit_export_statement(&ExportKind::Named(it))?;
            }
        };
        Ok(())
    }

    fn visit_empty_statement(&mut self) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_break(&mut self) -> Result<(), CompileError> {
        unimplementedc!("Break statement")
    }

    fn visit_continue(&mut self) -> Result<(), CompileError> {
        unimplementedc!("Continue statement")
    }

    fn visit_debugger(&mut self) -> Result<(), CompileError> {
        self.ib.build_debugger();
        Ok(())
    }

    fn visit_empty_expr(&mut self) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_class_declaration(&mut self, _c: &Class<'a>) -> Result<(), CompileError> {
        unimplementedc!("Class declaration")
    }
}
