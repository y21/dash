use std::{convert::TryInto, ptr::NonNull, usize};

use dash_middle::compiler::constant::{Constant, Function};
use dash_middle::compiler::{constant::ConstantPool, external::External};
use dash_middle::compiler::{CompileResult, FunctionCallMetadata, StaticImportKind};
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
use dash_middle::parser::statement::{BlockStatement, Loop};
use dash_optimizer::consteval::Eval;
use dash_optimizer::OptLevel;

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
    buf: Vec<u8>,
    cp: ConstantPool,
    scope: Scope<'a>,
    externals: Vec<External>,
    try_catch_depth: u16,
    ty: FunctionKind,
    caller: Option<NonNull<FunctionCompiler<'a>>>,
    opt_level: OptLevel,
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
    pub fn new(opt_level: OptLevel) -> Self {
        Self {
            buf: Vec::new(),
            cp: ConstantPool::new(),
            scope: Scope::new(),
            externals: Vec::new(),
            try_catch_depth: 0,
            ty: FunctionKind::Function,
            caller: None,
            opt_level,
        }
    }

    /// # Safety
    /// * Requires `caller` to not be invalid (i.e. due to moving) during calls
    pub unsafe fn with_caller<'s>(caller: &'s mut FunctionCompiler<'a>, ty: FunctionKind) -> Self {
        Self {
            buf: Vec::new(),
            cp: ConstantPool::new(),
            scope: Scope::new(),
            externals: Vec::new(),
            try_catch_depth: 0,
            ty,
            caller: Some(NonNull::new(caller).unwrap()),
            opt_level: caller.opt_level,
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

        self.accept_multiple(ast)?;
        Ok(CompileResult {
            instructions: self.buf,
            cp: self.cp,
            locals: self.scope.locals().len(),
            externals: self.externals,
        })
    }

    pub fn accept_multiple(&mut self, stmts: Vec<Statement<'a>>) -> Result<(), CompileError> {
        for stmt in stmts {
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
    fn accept(&mut self, mut stmt: Statement<'a>) -> Result<(), CompileError> {
        if self.opt_level.enabled() {
            stmt.fold(true);
        }

        match stmt {
            Statement::Expression(e) => self.visit_expression_statement(e),
            Statement::Variable(v) => self.visit_variable_declaration(v),
            Statement::If(i) => self.visit_if_statement(i),
            Statement::Block(b) => self.visit_block_statement(b),
            Statement::Function(f) => self.visit_function_declaration(f),
            Statement::Loop(Loop::For(f)) => self.visit_for_loop(f),
            Statement::Loop(Loop::While(w)) => self.visit_while_loop(w),
            Statement::Loop(Loop::ForOf(f)) => self.visit_for_of_loop(f),
            Statement::Return(r) => self.visit_return_statement(r),
            Statement::Try(t) => self.visit_try_catch(t),
            Statement::Throw(t) => self.visit_throw(t),
            Statement::Import(i) => self.visit_import_statement(i),
            Statement::Export(e) => self.visit_export_statement(e),
            Statement::Class(c) => self.visit_class_declaration(c),
            Statement::Continue => self.visit_continue(),
            Statement::Break => self.visit_break(),
            Statement::Debugger => self.visit_debugger(),
            Statement::Empty => self.visit_empty_statement(),
            Statement::Switch(s) => self.visit_switch_statement(s),
        }
    }

    fn accept_expr(&mut self, expr: Expr<'a>) -> Result<(), CompileError> {
        match expr {
            Expr::Binary(e) => self.visit_binary_expression(e),
            Expr::Assignment(e) => self.visit_assignment_expression(e),
            Expr::Grouping(e) => self.visit_grouping_expression(e),
            Expr::Literal(LiteralExpr::Identifier(i)) => self.visit_identifier_expression(&i),
            Expr::Literal(l) => self.visit_literal_expression(l),
            Expr::Unary(e) => self.visit_unary_expression(e),
            Expr::Call(e) => self.visit_function_call(e),
            Expr::Conditional(e) => self.visit_conditional_expr(e),
            Expr::PropertyAccess(e) => self.visit_property_access_expr(e, false),
            Expr::Sequence(e) => self.visit_sequence_expr(e),
            Expr::Postfix(e) => self.visit_postfix_expr(e),
            Expr::Function(e) => self.visit_function_expr(e),
            Expr::Array(e) => self.visit_array_literal(e),
            Expr::Object(e) => self.visit_object_literal(e),
            Expr::Empty => self.visit_empty_expr(),
        }
    }

    fn visit_binary_expression(
        &mut self,
        BinaryExpr { left, right, operator }: BinaryExpr<'a>,
    ) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);
        ib.accept_expr(*left)?;

        macro_rules! trivial_case {
            ($k:expr) => {{
                ib.accept_expr(*right)?;
                $k(&mut ib)
            }};
        }

        match operator {
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
                ib.accept_expr(*right)?;
                ib.add_local_label(Label::IfEnd);
            }
            TokenType::LogicalAnd => {
                ib.build_jmpfalsenp(Label::IfEnd);
                ib.build_pop(); // Only pop LHS if it is true
                ib.accept_expr(*right)?;
                ib.add_local_label(Label::IfEnd);
            }
            TokenType::NullishCoalescing => {
                ib.build_jmpnullishnp(Label::IfBranch(0));
                ib.build_jmp(Label::IfEnd);

                ib.add_local_label(Label::IfBranch(0));
                ib.build_pop();
                ib.accept_expr(*right)?;
                ib.add_local_label(Label::IfEnd);
            }
            other => unimplementedc!("Binary operator {:?}", other),
        }

        Ok(())
    }

    fn visit_expression_statement(&mut self, expr: Expr<'a>) -> Result<(), CompileError> {
        self.accept_expr(expr)?;
        InstructionBuilder::new(self).build_pop();
        Ok(())
    }

    fn visit_grouping_expression(&mut self, GroupingExpr(exprs): GroupingExpr<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        for expr in exprs {
            ib.accept_expr(expr)?;
            ib.build_pop();
        }

        ib.remove_pop_end();

        Ok(())
    }

    fn visit_literal_expression(&mut self, expr: LiteralExpr<'a>) -> Result<(), CompileError> {
        InstructionBuilder::new(self).build_constant(Constant::from_literal(&expr))?;
        Ok(())
    }

    fn visit_identifier_expression(&mut self, ident: &str) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        match ident {
            "this" => ib.build_this(),
            "super" => ib.build_super(),
            "globalThis" => ib.build_global(),
            ident => match ib.find_local(ident) {
                Some((index, local)) => ib.build_local_load(index, local.is_extern()),
                _ => ib.build_global_load(ident)?,
            },
        };

        Ok(())
    }

    fn visit_unary_expression(&mut self, UnaryExpr { operator, expr }: UnaryExpr<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);
        ib.accept_expr(*expr)?;

        match operator {
            TokenType::Plus => ib.build_pos(),
            TokenType::Minus => ib.build_neg(),
            TokenType::Typeof => ib.build_typeof(),
            TokenType::BitwiseNot => ib.build_bitnot(),
            TokenType::LogicalNot => ib.build_not(),
            TokenType::Void => {
                ib.build_pop();
                ib.build_undef();
            }
            TokenType::Yield => {
                if !matches!(ib.ty, FunctionKind::Generator) {
                    return Err(CompileError::YieldOutsideGenerator);
                }

                ib.build_yield();
            }
            _ => unimplementedc!("Unary operator {:?}", operator),
        }

        Ok(())
    }

    fn visit_variable_declaration(
        &mut self,
        VariableDeclaration { binding, value }: VariableDeclaration<'a>,
    ) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);
        let id = ib.scope.add_local(binding, false)?;

        if let Some(expr) = value {
            ib.accept_expr(expr)?;
            ib.build_local_store(id, false);
            ib.build_pop();
        }

        Ok(())
    }

    fn visit_if_statement(
        &mut self,
        IfStatement {
            condition,
            then,
            branches,
            el,
        }: IfStatement<'a>,
    ) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        // Desugar last `else` block into `else if(true)` for simplicity
        if let Some(then) = &el {
            let then = &**then;
            let mut branches = branches.borrow_mut();

            branches.push(IfStatement::new(
                Expr::bool_literal(true),
                then.clone(),
                Vec::new(),
                None,
            ));
        }

        let branches = branches.into_inner();
        let len: u16 = branches
            .len()
            .try_into()
            .map_err(|_| CompileError::IfBranchLimitExceeded)?;

        ib.accept_expr(condition)?;
        if branches.is_empty() {
            ib.build_jmpfalsep(Label::IfEnd);
        } else {
            ib.build_jmpfalsep(Label::IfBranch(0));
        }
        ib.accept(*then)?;
        ib.build_jmp(Label::IfEnd);

        for (id, branch) in branches.into_iter().enumerate() {
            let id = id as u16;

            ib.add_local_label(Label::IfBranch(id));
            ib.accept_expr(branch.condition)?;
            if id == len - 1 {
                ib.build_jmpfalsep(Label::IfEnd);

                ib.accept(*branch.then)?;
            } else {
                ib.build_jmpfalsep(Label::IfBranch(id + 1));

                ib.accept(*branch.then)?;

                ib.build_jmp(Label::IfEnd);
            }
        }

        ib.add_local_label(Label::IfEnd);
        Ok(())
    }

    fn visit_block_statement(&mut self, BlockStatement(stmt): BlockStatement<'a>) -> Result<(), CompileError> {
        self.scope.enter();
        let re = self.accept_multiple(stmt);
        self.scope.exit();
        re
    }

    fn visit_function_declaration(&mut self, fun: FunctionDeclaration<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);
        let id = ib.scope.add_local(
            VariableBinding {
                name: fun.name.expect("Function declaration did not have a name"),
                kind: VariableDeclarationKind::Var,
                ty: None,
            },
            false,
        )?;
        ib.visit_function_expr(fun)?;
        ib.build_local_store(id, false);
        ib.build_pop();
        Ok(())
    }

    fn visit_while_loop(&mut self, WhileLoop { condition, body }: WhileLoop<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        ib.add_local_label(Label::LoopCondition);
        ib.accept_expr(condition)?;
        ib.build_jmpfalsep(Label::LoopEnd);

        ib.accept(*body)?;
        ib.build_jmp(Label::LoopCondition);

        ib.add_local_label(Label::LoopEnd);
        Ok(())
    }

    fn visit_assignment_expression(
        &mut self,
        AssignmentExpr { left, right, operator }: AssignmentExpr<'a>,
    ) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        if let Expr::PropertyAccess(prop) = &*left {
            ib.accept_expr((*prop.target).clone())?;
        }

        ib.accept_expr(*right)?;

        match *left {
            Expr::Literal(lit) => {
                let ident = lit.to_identifier();

                if let Some((id, local)) = ib.find_local(&ident) {
                    if matches!(local.binding().kind, VariableDeclarationKind::Const) {
                        return Err(CompileError::ConstAssignment);
                    }

                    let is_extern = local.is_extern();

                    match operator {
                        TokenType::Assignment => {}
                        TokenType::AdditionAssignment => {
                            ib.build_local_load(id, is_extern);
                            // += requires reversing (right, left)
                            // we effectively need to rewrite it from
                            // left = right + left
                            // to
                            // left = left + right
                            ib.build_revstck(2);
                            ib.build_add();
                        }
                        TokenType::SubtractionAssignment => {
                            ib.build_local_load(id, is_extern);
                            ib.build_revstck(2);
                            ib.build_sub();
                        }
                        _ => unimplementedc!("Unknown operator"),
                    }

                    ib.build_local_store(id, is_extern);
                } else {
                    match operator {
                        TokenType::Assignment => {}
                        TokenType::AdditionAssignment => {
                            ib.build_global_load(&ident)?;
                            ib.build_add();
                        }
                        TokenType::SubtractionAssignment => {
                            ib.build_global_load(&ident)?;
                            ib.build_sub();
                        }
                        _ => unimplementedc!("Unknown operator"),
                    }

                    ib.build_global_store(&ident)?;
                }
            }
            Expr::PropertyAccess(PropertyAccessExpr { computed, property, .. }) => {
                if !matches!(operator, TokenType::Assignment) {
                    unimplementedc!("Assignment operator {:?}", operator);
                }

                match (*property, computed) {
                    (Expr::Literal(lit), false) => {
                        let ident = lit.to_identifier();
                        ib.build_static_prop_set(&ident)?;
                    }
                    (e, _) => {
                        ib.accept_expr(e)?;
                        ib.build_dynamic_prop_set();
                    }
                }
            }
            _ => unimplementedc!("Assignment to non-identifier"),
        }

        Ok(())
    }

    fn visit_function_call(
        &mut self,
        FunctionCall {
            constructor_call,
            target,
            arguments,
        }: FunctionCall<'a>,
    ) -> Result<(), CompileError> {
        // specialize property access
        // TODO: this also needs to be specialized for assignment expressions with property access as target

        let has_this = if let Expr::PropertyAccess(p) = *target {
            self.visit_property_access_expr(p, true)?;
            true
        } else {
            self.accept_expr(*target)?;
            false
        };

        let argc = arguments
            .len()
            .try_into()
            .map_err(|_| CompileError::ParameterLimitExceeded)?;

        for arg in arguments {
            self.accept_expr(arg)?;
        }

        let meta = FunctionCallMetadata::new_checked(argc, constructor_call, has_this)
            .ok_or(CompileError::ParameterLimitExceeded)?;

        InstructionBuilder::new(self).build_call(meta);

        Ok(())
    }

    fn visit_return_statement(&mut self, ReturnStatement(stmt): ReturnStatement<'a>) -> Result<(), CompileError> {
        let tc_depth = self.try_catch_depth;
        self.accept_expr(stmt)?;
        InstructionBuilder::new(self).build_ret(tc_depth);
        Ok(())
    }

    fn visit_conditional_expr(
        &mut self,
        ConditionalExpr { condition, then, el }: ConditionalExpr<'a>,
    ) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        ib.accept_expr(*condition)?;
        ib.build_jmpfalsep(Label::IfBranch(0));

        ib.accept_expr(*then)?;
        ib.build_jmp(Label::IfEnd);

        ib.add_local_label(Label::IfBranch(0));
        ib.accept_expr(*el)?;

        ib.add_local_label(Label::IfEnd);
        Ok(())
    }

    fn visit_property_access_expr(
        &mut self,
        PropertyAccessExpr {
            computed,
            target,
            property,
        }: PropertyAccessExpr<'a>,
        preserve_this: bool,
    ) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        ib.accept_expr(*target)?;

        match (*property, computed) {
            (Expr::Literal(lit), false) => {
                let ident = lit.to_identifier();
                ib.build_static_prop_access(&ident, preserve_this)?;
            }
            (e, _) => {
                ib.accept_expr(e)?;
                ib.build_dynamic_prop_access(preserve_this);
            }
        }

        Ok(())
    }

    fn visit_sequence_expr(&mut self, (expr1, expr2): Seq<'a>) -> Result<(), CompileError> {
        self.accept_expr(*expr1)?;
        InstructionBuilder::new(self).build_pop();
        self.accept_expr(*expr2)?;

        Ok(())
    }

    fn visit_postfix_expr(&mut self, (tt, expr): Postfix<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        match &*expr {
            Expr::Literal(lit) => {
                let ident = lit.to_identifier();

                if let Some((id, loc)) = ib.find_local(&ident) {
                    ib.build_local_load(id, loc.is_extern());
                } else {
                    unimplementedc!("Global postfix expression");
                }
            }
            _ => unimplementedc!("Non-identifier postfix expression"),
        }

        ib.visit_assignment_expression(AssignmentExpr {
            left: expr.clone(),
            operator: match tt {
                TokenType::Increment => TokenType::AdditionAssignment,
                TokenType::Decrement => TokenType::SubtractionAssignment,
                _ => unreachable!("Token never emitted"),
            },
            right: Box::new(Expr::number_literal(1.0)),
        })?;
        ib.build_pop();

        Ok(())
    }

    fn visit_function_expr(
        &mut self,
        FunctionDeclaration {
            name,
            parameters: arguments,
            statements,
            ty,
        }: FunctionDeclaration<'a>,
    ) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);
        let mut compiler = unsafe { FunctionCompiler::with_caller(&mut ib, ty) };
        let scope = &mut compiler.scope;

        for (name, _ty) in &arguments {
            scope.add_local(
                VariableBinding {
                    kind: VariableDeclarationKind::Var,
                    name,
                    ty: None,
                },
                false,
            )?;
        }

        let cmp = compiler.compile_ast(statements, false)?;

        let function = Function {
            buffer: cmp.instructions.into(),
            constants: cmp.cp.into_vec().into(),
            locals: cmp.locals,
            name: name.map(ToOwned::to_owned),
            ty,
            params: arguments.len(),
            externals: cmp.externals.into(),
        };
        ib.build_constant(Constant::Function(function))?;

        Ok(())
    }

    fn visit_array_literal(&mut self, ArrayLiteral(exprs): ArrayLiteral<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        let len = exprs
            .len()
            .try_into()
            .map_err(|_| CompileError::ArrayLitLimitExceeded)?;

        for expr in exprs {
            ib.accept_expr(expr)?;
        }

        ib.build_arraylit(len);
        Ok(())
    }

    fn visit_object_literal(&mut self, ObjectLiteral(exprs): ObjectLiteral<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        let mut idents = Vec::with_capacity(exprs.len());
        for (ident, value) in exprs {
            ib.accept_expr(value)?;
            let ident = Constant::Identifier((*ident).into());
            idents.push(ident);
        }

        ib.build_objlit(idents)?;
        Ok(())
    }

    fn visit_try_catch(&mut self, TryCatch { try_, catch, .. }: TryCatch<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        ib.build_try_block();

        ib.try_catch_depth += 1;
        ib.scope.enter();
        ib.accept(*try_)?;
        ib.scope.exit();
        ib.try_catch_depth -= 1;

        ib.build_jmp(Label::TryEnd);

        ib.add_local_label(Label::Catch);

        ib.scope.enter();

        if let Some(ident) = catch.ident {
            let id = ib.scope.add_local(
                VariableBinding {
                    kind: VariableDeclarationKind::Var,
                    name: ident,
                    ty: None,
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

        ib.accept(*catch.body)?;
        ib.scope.exit();

        ib.add_local_label(Label::TryEnd);
        ib.build_try_end();

        Ok(())
    }

    fn visit_throw(&mut self, expr: Expr<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);
        ib.accept_expr(expr)?;
        ib.build_throw();
        Ok(())
    }

    fn visit_for_loop(
        &mut self,
        ForLoop {
            init,
            condition,
            finalizer,
            body,
        }: ForLoop<'a>,
    ) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);
        ib.scope.enter();

        // Initialization
        if let Some(init) = init {
            ib.accept(*init)?;
        }

        // Condition
        ib.add_local_label(Label::LoopCondition);
        if let Some(condition) = condition {
            ib.accept_expr(condition)?;
            ib.build_jmpfalsep(Label::LoopEnd);
        }

        // Body
        ib.accept(*body)?;

        // Increment
        if let Some(finalizer) = finalizer {
            ib.accept_expr(finalizer)?;
            ib.build_pop();
        }
        ib.build_jmp(Label::LoopCondition);

        ib.add_local_label(Label::LoopEnd);
        self.scope.exit();

        Ok(())
    }

    fn visit_for_of_loop(&mut self, _f: ForOfLoop<'a>) -> Result<(), CompileError> {
        unimplementedc!("For of loop")
    }

    fn visit_import_statement(&mut self, import: ImportKind<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        match import {
            ImportKind::Dynamic(ex) => {
                ib.accept_expr(ex)?;
                ib.build_dynamic_import();
            }
            ref kind @ (ImportKind::DefaultAs(ref spec, path) | ImportKind::AllAs(ref spec, path)) => {
                let local_id = ib.scope.add_local(
                    VariableBinding {
                        kind: VariableDeclarationKind::Var,
                        name: match spec {
                            SpecifierKind::Ident(id) => id,
                        },
                        ty: None,
                    },
                    false,
                )?;

                let path_id = ib.cp.add(Constant::String((*path).into()))?;

                ib.build_static_import(
                    match kind {
                        ImportKind::DefaultAs(..) => StaticImportKind::Default,
                        ImportKind::AllAs(..) => StaticImportKind::All,
                        _ => unreachable!(),
                    },
                    local_id,
                    path_id,
                );
            }
        }

        Ok(())
    }

    fn visit_export_statement(&mut self, export: ExportKind<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        match export {
            ExportKind::Default(expr) => {
                ib.accept_expr(expr)?;
                ib.build_default_export();
            }
            ExportKind::Named(names) => {
                let mut it = Vec::with_capacity(names.len());

                for name in names.iter().copied() {
                    let ident_id = ib.cp.add(Constant::Identifier(name.into()))?;

                    match ib.find_local(name) {
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
                let it = vars.iter().map(|var| var.binding.name).collect::<Vec<_>>();

                for var in vars {
                    self.visit_variable_declaration(var)?;
                }

                self.visit_export_statement(ExportKind::Named(it))?;
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
        InstructionBuilder::new(self).build_debugger();
        Ok(())
    }

    fn visit_empty_expr(&mut self) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_class_declaration(&mut self, _c: Class<'a>) -> Result<(), CompileError> {
        unimplementedc!("Class declaration")
    }

    fn visit_switch_statement(
        &mut self,
        _s: dash_middle::parser::statement::SwitchStatement<'a>,
    ) -> Result<(), CompileError> {
        unimplementedc!("Switch statement")
    }
}
