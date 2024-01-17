use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::convert::TryInto;
use std::rc::Rc;
use std::usize;

use dash_log::{debug, span, Level};
use dash_middle::compiler::constant::{Buffer, Constant, ConstantPool, Function};
use dash_middle::compiler::external::External;
use dash_middle::compiler::instruction::{AssignKind, IntrinsicOperation};
use dash_middle::compiler::scope::{CompileValueType, Scope, ScopeLocal};
use dash_middle::compiler::{CompileResult, DebugSymbols, FunctionCallMetadata, StaticImportKind};
use dash_middle::interner::{sym, StringInterner, Symbol};
use dash_middle::lexer::token::TokenType;
use dash_middle::parser::error::Error;
use dash_middle::parser::expr::{
    ArrayLiteral, ArrayMemberKind, AssignmentExpr, AssignmentTarget, BinaryExpr, CallArgumentKind, ConditionalExpr,
    Expr, ExprKind, FunctionCall, GroupingExpr, LiteralExpr, ObjectLiteral, ObjectMemberKind, Postfix,
    PropertyAccessExpr, Seq, UnaryExpr,
};
use dash_middle::parser::statement::{
    BlockStatement, Class, ClassMemberKind, DoWhileLoop, ExportKind, ForInLoop, ForLoop, ForOfLoop, FuncId,
    FunctionDeclaration, FunctionKind, IfStatement, ImportKind, Loop, Parameter, ReturnStatement, SpecifierKind,
    Statement, StatementKind, SwitchCase, SwitchStatement, TryCatch, VariableBinding, VariableDeclaration,
    VariableDeclarationKind, VariableDeclarationName, VariableDeclarations, WhileLoop,
};
use dash_middle::sourcemap::Span;
use dash_middle::visitor::Visitor;
use dash_optimizer::consteval::ConstFunctionEvalCtx;
use dash_optimizer::type_infer::TypeInferCtx;
use dash_optimizer::OptLevel;
use instruction::compile_local_load;
use jump_container::JumpContainer;

use crate::builder::{InstructionBuilder, Label};

use self::instruction::NamedExportKind;

pub mod builder;
#[cfg(feature = "from_string")]
pub mod from_string;
pub mod instruction;
pub mod transformations;
// #[cfg(test)]
// mod test;
mod jump_container;

macro_rules! unimplementedc {
    ($span:expr,$($what:expr),*) => {
        return Err(Error::Unimplemented($span,format_args!($($what),*).to_string()))
    };
}

#[derive(Debug, Clone, Copy)]
enum Breakable {
    Loop { loop_id: usize },
    Switch { switch_id: usize },
}

/// Function-specific state, such as
#[derive(Debug)]
struct FunctionLocalState {
    /// Instruction buffer
    buf: Vec<u8>,
    /// A list of constants used throughout this function.
    ///
    /// Bytecode can refer to constants using the [Instruction::Constant] instruction, followed by a u8 index.
    cp: ConstantPool,
    /// Current try catch depth
    try_catch_depth: u16,
    /// The type of function that this FunctionCompiler compiles
    ty: FunctionKind,
    /// Whether the function being compiled is async
    r#async: bool,
    /// Container, used for storing global labels that can be jumped to
    jc: JumpContainer,
    /// A stack of breakable labels (loop/switch)
    breakables: Vec<Breakable>,
    /// Keeps track of the total number of loops to be able to have unique IDs
    loop_counter: usize,
    /// Keeps track of the total number of loops to be able to have unique IDs
    switch_counter: usize,
    id: FuncId,
    debug_symbols: DebugSymbols,
    /// Whether this function references `arguments` anywhere in its body
    ///
    /// Also tracks the span for error reporting, but is discarded past the compiler stage.
    references_arguments: Option<Span>,
}

impl FunctionLocalState {
    pub fn new(ty: FunctionKind, id: FuncId, r#async: bool) -> Self {
        Self {
            buf: Vec::new(),
            cp: ConstantPool::new(),
            try_catch_depth: 0,
            ty,
            r#async,
            jc: JumpContainer::new(),
            breakables: Vec::new(),
            loop_counter: 0,
            switch_counter: 0,
            id,
            debug_symbols: DebugSymbols::default(),
            references_arguments: None,
        }
    }

    /// "Prepares" a loop and returns a unique ID that identifies this loop
    ///
    /// Specifically, this function increments a FunctionCompiler-local loop counter and
    /// inserts the loop into a stack of switch-case/loops so that `break` (and `continue`)
    /// statements can be resolved at compile-time
    fn prepare_loop(&mut self) -> usize {
        let loop_id = self.loop_counter;
        self.breakables.push(Breakable::Loop { loop_id });
        self.loop_counter += 1;
        loop_id
    }

    /// Same as [`prepare_loop`] but for switch statements
    fn prepare_switch(&mut self) -> usize {
        let switch_id = self.switch_counter;
        self.breakables.push(Breakable::Switch { switch_id });
        self.switch_counter += 1;
        switch_id
    }
    fn exit_loop(&mut self) {
        let item = self.breakables.pop();
        match item {
            None | Some(Breakable::Switch { .. }) => panic!("Tried to exit loop, but no breakable was found"),
            Some(Breakable::Loop { .. }) => {}
        }
    }

    fn exit_switch(&mut self) {
        let item = self.breakables.pop();
        match item {
            None | Some(Breakable::Loop { .. }) => panic!("Tried to exit switch, but no breakable was found"),
            Some(Breakable::Switch { .. }) => {}
        }
    }

    fn add_global_label(&mut self, label: Label) {
        jump_container::add_label(&mut self.jc, label, &mut self.buf)
    }

    /// Jumps to a label that was previously (or will be) created by a call to `add_global_label`
    fn add_global_jump(&mut self, label: Label) {
        jump_container::add_jump(&mut self.jc, label, &mut self.buf)
    }
}

#[derive(Debug)]
pub struct FunctionCompiler<'interner> {
    function_stack: Vec<FunctionLocalState>,
    tcx: TypeInferCtx,
    interner: &'interner mut StringInterner,
    /// Optimization level
    #[allow(unused)]
    opt_level: OptLevel,
    source: Rc<str>,
}

impl<'interner> FunctionCompiler<'interner> {
    pub fn new(source: &str, opt_level: OptLevel, tcx: TypeInferCtx, interner: &'interner mut StringInterner) -> Self {
        Self {
            opt_level,
            tcx,
            interner,
            function_stack: Vec::new(),
            source: Rc::from(source),
        }
    }

    pub fn compile_ast(mut self, mut ast: Vec<Statement>, implicit_return: bool) -> Result<CompileResult, Error> {
        let compile_span = span!(Level::TRACE, "compile ast");
        let _enter = compile_span.enter();

        transformations::hoist_declarations(&mut ast);
        if implicit_return {
            transformations::ast_patch_implicit_return(&mut ast);
        } else {
            // Push an implicit `return undefined;` statement at the end in case there is not already an explicit one
            transformations::ast_insert_implicit_return(&mut ast);
        }

        let tif_span = span!(Level::TRACE, "type infer");
        tif_span.in_scope(|| {
            debug!("begin type inference");
            for stmt in &ast {
                self.tcx.visit_statement(stmt, FuncId::ROOT);
            }
            debug!("finished type inference");
        });

        let consteval_span = span!(Level::TRACE, "const eval");
        consteval_span.in_scope(|| {
            let opt_level = self.opt_level;
            if opt_level.enabled() {
                debug!("begin const eval, opt level: {:?}", opt_level);
                let mut cfx = ConstFunctionEvalCtx::new(&mut self.tcx, self.interner, opt_level);

                for stmt in &mut ast {
                    cfx.visit_statement(stmt, FuncId::ROOT);
                }
                debug!("finished const eval");
            } else {
                debug!("skipping const eval");
            }
        });

        self.function_stack
            .push(FunctionLocalState::new(FunctionKind::Function, FuncId::ROOT, false));

        self.accept_multiple(ast)?;

        let root = self.function_stack.pop().expect("No root function");
        assert_eq!(root.id, FuncId::ROOT, "Function must be the root function");
        if let Some(span) = root.references_arguments {
            return Err(Error::ArgumentsInRoot(span));
        }
        let root_scope = self.tcx.scope(root.id);
        let locals = root_scope.locals().len();
        let externals = root_scope.externals().to_owned();

        Ok(CompileResult {
            instructions: root.buf,
            cp: root.cp,
            locals,
            externals,
            source: self.source.into(),
            debug_symbols: root.debug_symbols,
        })
    }

    pub fn accept_multiple(&mut self, stmts: Vec<Statement>) -> Result<(), Error> {
        for stmt in stmts {
            self.accept(stmt)?;
        }
        Ok(())
    }

    fn current_function(&self) -> &FunctionLocalState {
        self.function_stack.last().expect("Function must be present")
    }

    fn current_function_mut(&mut self) -> &mut FunctionLocalState {
        self.function_stack.last_mut().expect("Function must be present")
    }

    fn current_scope(&self) -> &Scope {
        let id = self.current_function().id;
        self.tcx.scope(id)
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        let id = self.current_function().id;
        self.tcx.scope_mut(id)
    }

    /// Adds an external to the current [`FunctionLocalState`] if it's not already present
    /// and returns its ID
    fn add_external_to_func(&mut self, func_id: FuncId, external_id: u16, is_nested_external: bool) -> usize {
        let externals = self.tcx.scope_mut(func_id).externals_mut();
        let id = externals.iter().position(
            |External {
                 id,
                 is_nested_external: is_external,
             }| *id == external_id && *is_external == is_nested_external,
        );

        match id {
            Some(id) => id,
            None => {
                externals.push(External {
                    id: external_id,
                    is_nested_external,
                });
                externals.len() - 1
            }
        }
    }

    fn find_local_in_scope(&mut self, ident: Symbol, func_id: FuncId) -> Option<(u16, ScopeLocal, bool)> {
        if let Some((id, local)) = self.tcx.scope(func_id).find_local(ident) {
            Some((id, local.clone(), false))
        } else {
            let parent = self.tcx.scope_node(func_id).parent()?;

            let (local_id, loc, nested_extern) = self.find_local_in_scope(ident, parent.into())?;
            // TODO: don't hardcast
            let external_id = self.add_external_to_func(func_id, local_id, nested_extern) as u16;
            // println!("{func_id:?} {external_id}");
            Some((external_id, loc, true))
        }
    }
    /// Tries to find a local in the current or surrounding scopes
    ///
    /// If a local variable is found in a parent scope, it is marked as an extern local
    pub fn find_local(&mut self, ident: Symbol) -> Option<(u16, ScopeLocal, bool)> {
        let func_id = self.current_function().id;
        self.find_local_in_scope(ident, func_id)
    }

    fn visit_for_each_kinded_loop(
        &mut self,
        kind: ForEachLoopKind,
        binding: VariableBinding,
        expr: Expr,
        mut body: Box<Statement>,
    ) -> Result<(), Error> {
        // For-Of Loop Desugaring:

        // === ORIGINAL ===
        // for (const x of [1,2]) console.log(x)

        // === AFTER DESUGARING ===
        // let __forOfIter = [1,2][Symbol.iterator]();
        // let __forOfGenStep;
        // let x;

        // while (!(__forOfGenStep = __forOfIter.next()).done) {
        //     console.log(x)
        // }

        // For-In Loop Desugaring

        // === ORIGINAL ===
        // for (const x in { a: 3, b: 4 }) console.log(x);

        // === AFTER DESUGARING ===
        // let __forInIter = [1,2][__intrinsicForInIter]();
        // let __forInGenStep;
        // let x;

        // while (!(__forInGenStep = __forOfIter.next()).done) {
        //     console.log(x)
        // }

        let mut ib = InstructionBuilder::new(self);
        let for_of_iter_id = ib
            .current_scope_mut()
            .add_local(sym::for_of_iter, VariableDeclarationKind::Unnameable, None)
            .map_err(|_| Error::LocalLimitExceeded(expr.span))?;

        let for_of_gen_step_id = ib
            .current_scope_mut()
            .add_local(sym::for_of_gen_step, VariableDeclarationKind::Unnameable, None)
            .map_err(|_| Error::LocalLimitExceeded(expr.span))?;

        ib.accept_expr(expr)?;
        match kind {
            ForEachLoopKind::ForOf => ib.build_symbol_iterator(),
            ForEachLoopKind::ForIn => ib.build_for_in_iterator(),
        }
        ib.build_local_store(AssignKind::Assignment, for_of_iter_id, false);
        ib.build_pop();

        // Prepend variable assignment to body
        if !matches!(body.kind, StatementKind::Block(..)) {
            let old_body = std::mem::replace(&mut *body, Statement::dummy_empty());

            *body = Statement {
                span: old_body.span,
                kind: StatementKind::Block(BlockStatement(vec![old_body])),
            };
        }

        // Assign iterator value to binding at the very start of the for loop body
        match &mut body.kind {
            StatementKind::Block(BlockStatement(stmts)) => {
                let gen_step = compile_local_load(for_of_gen_step_id, false);

                let var = Statement {
                    span: Span::COMPILER_GENERATED,
                    kind: StatementKind::Variable(VariableDeclarations(vec![VariableDeclaration::new(
                        binding,
                        Some(Expr {
                            span: Span::COMPILER_GENERATED,
                            kind: ExprKind::property_access(
                                false,
                                Expr {
                                    span: Span::COMPILER_GENERATED,
                                    kind: ExprKind::compiled(gen_step),
                                },
                                Expr {
                                    span: Span::COMPILER_GENERATED,
                                    kind: ExprKind::identifier(sym::value),
                                },
                            ),
                        }),
                    )])),
                };

                if stmts.is_empty() {
                    stmts.push(var);
                } else {
                    stmts.insert(0, var);
                }
            }
            _ => unreachable!("For-of body was not a statement"),
        }

        let for_of_iter_binding_bc = compile_local_load(for_of_iter_id, false);

        // for..of -> while loop rewrite
        ib.visit_while_loop(
            Span::COMPILER_GENERATED,
            WhileLoop {
                condition: Expr {
                    span: Span::COMPILER_GENERATED,
                    kind: ExprKind::unary(
                        TokenType::LogicalNot,
                        Expr {
                            span: Span::COMPILER_GENERATED,
                            kind: ExprKind::property_access(
                                false,
                                Expr {
                                    span: Span::COMPILER_GENERATED,
                                    kind: ExprKind::assignment_local_space(
                                        for_of_gen_step_id,
                                        Expr {
                                            span: Span::COMPILER_GENERATED,
                                            kind: ExprKind::function_call(
                                                Expr {
                                                    span: Span::COMPILER_GENERATED,
                                                    kind: ExprKind::property_access(
                                                        false,
                                                        Expr {
                                                            span: Span::COMPILER_GENERATED,
                                                            kind: ExprKind::compiled(for_of_iter_binding_bc),
                                                        },
                                                        Expr {
                                                            span: Span::COMPILER_GENERATED,
                                                            kind: ExprKind::identifier(sym::next),
                                                        },
                                                    ),
                                                },
                                                Vec::new(),
                                                false,
                                            ),
                                        },
                                        TokenType::Assignment,
                                    ),
                                },
                                Expr {
                                    span: Span::COMPILER_GENERATED,
                                    kind: ExprKind::identifier(sym::done),
                                },
                            ),
                        },
                    ),
                },
                body,
            },
        )?;

        Ok(())
    }
}

enum ForEachLoopKind {
    ForOf,
    ForIn,
}

impl<'interner> Visitor<Result<(), Error>> for FunctionCompiler<'interner> {
    fn accept(&mut self, Statement { kind, span }: Statement) -> Result<(), Error> {
        match kind {
            StatementKind::Expression(e) => self.visit_expression_statement(e),
            StatementKind::Variable(v) => self.visit_variable_declaration(span, v),
            StatementKind::If(i) => self.visit_if_statement(span, i),
            StatementKind::Block(b) => self.visit_block_statement(span, b),
            StatementKind::Function(f) => self.visit_function_declaration(span, f),
            StatementKind::Loop(Loop::For(f)) => self.visit_for_loop(span, f),
            StatementKind::Loop(Loop::While(w)) => self.visit_while_loop(span, w),
            StatementKind::Loop(Loop::ForOf(f)) => self.visit_for_of_loop(span, f),
            StatementKind::Loop(Loop::ForIn(f)) => self.visit_for_in_loop(span, f),
            StatementKind::Loop(Loop::DoWhile(d)) => self.visit_do_while_loop(span, d),
            StatementKind::Return(r) => self.visit_return_statement(span, r),
            StatementKind::Try(t) => self.visit_try_catch(span, t),
            StatementKind::Throw(t) => self.visit_throw(span, t),
            StatementKind::Import(i) => self.visit_import_statement(span, i),
            StatementKind::Export(e) => self.visit_export_statement(span, e),
            StatementKind::Class(c) => self.visit_class_declaration(span, c),
            StatementKind::Continue => self.visit_continue(span),
            StatementKind::Break => self.visit_break(span),
            StatementKind::Debugger => self.visit_debugger(span),
            StatementKind::Empty => self.visit_empty_statement(),
            StatementKind::Switch(s) => self.visit_switch_statement(span, s),
        }
    }

    fn accept_expr(&mut self, Expr { kind, span }: Expr) -> Result<(), Error> {
        match kind {
            ExprKind::Binary(e) => self.visit_binary_expression(span, e),
            ExprKind::Assignment(e) => self.visit_assignment_expression(span, e),
            ExprKind::Grouping(e) => self.visit_grouping_expression(span, e),
            ExprKind::Literal(LiteralExpr::Identifier(i)) => self.visit_identifier_expression(span, i),
            ExprKind::Literal(l) => self.visit_literal_expression(span, l),
            ExprKind::Unary(e) => self.visit_unary_expression(span, e),
            ExprKind::Call(e) => self.visit_function_call(span, e),
            ExprKind::Conditional(e) => self.visit_conditional_expr(span, e),
            ExprKind::PropertyAccess(e) => self.visit_property_access_expr(span, e, false),
            ExprKind::Sequence(e) => self.visit_sequence_expr(span, e),
            ExprKind::Postfix(e) => self.visit_postfix_expr(span, e),
            ExprKind::Prefix(e) => self.visit_prefix_expr(span, e),
            ExprKind::Function(e) => self.visit_function_expr(span, e),
            ExprKind::Array(e) => self.visit_array_literal(span, e),
            ExprKind::Object(e) => self.visit_object_literal(span, e),
            ExprKind::Compiled(mut buf) => {
                self.current_function_mut().buf.append(&mut buf);
                Ok(())
            }
            ExprKind::Empty => self.visit_empty_expr(),
        }
    }

    fn visit_binary_expression(
        &mut self,
        span: Span,
        BinaryExpr { left, right, operator }: BinaryExpr,
    ) -> Result<(), Error> {
        let func_id = self.current_function().id;
        let left_type = self.tcx.visit(&left, func_id);
        let right_type = self.tcx.visit(&right, func_id);

        let mut ib = InstructionBuilder::new(self);
        ib.accept_expr(*left)?;

        macro_rules! generic_bin {
            ($gen:expr) => {{
                ib.accept_expr(*right)?;
                $gen(&mut ib);
            }};
        }

        macro_rules! numeric_bin {
            ($gen:expr, $spec: expr) => {{
                ib.accept_expr(*right)?;

                match (left_type, right_type) {
                    (Some(CompileValueType::Number), Some(CompileValueType::Number)) => {
                        ib.build_intrinsic_op($spec);
                    }
                    _ => {
                        $gen(&mut ib);
                    }
                }
            }};
        }

        macro_rules! numeric_bin_const_spec {
            ($gen:expr, $spec: expr, $( $t:ty => $spec_const:expr ),*) => {{
                match (left_type, right_type) {
                    (Some(CompileValueType::Number), Some(CompileValueType::Number)) => {
                        fn try_const_spec(ib: &mut InstructionBuilder<'_, '_>, right: &ExprKind) -> bool {
                            if let ExprKind::Literal(LiteralExpr::Number(n)) = right {
                                let n = *n;
                                // Using match to be able to expand type->spec metavars
                                match n.floor() == n {
                                    $(
                                        true if n >= (<$t>::MIN as f64) && n <= (<$t>::MAX as f64) => {
                                            // n can be safely cast to $t
                                            $spec_const(ib, n as $t);
                                            return true;
                                        }
                                    )*
                                    _ => {}
                                }
                            }
                            false
                        }

                        if !try_const_spec(&mut ib, &right.kind) {
                            // Less specialized: both sides are numbers, but dynamic values
                            ib.accept_expr(*right)?;
                            ib.build_intrinsic_op($spec);
                        }
                    }
                    _ => {
                        ib.accept_expr(*right)?;
                        $gen(&mut ib);
                    }
                }
            }};
        }

        match operator {
            TokenType::Plus => numeric_bin!(InstructionBuilder::build_add, IntrinsicOperation::AddNumLR),
            TokenType::Minus => numeric_bin!(InstructionBuilder::build_sub, IntrinsicOperation::SubNumLR),
            TokenType::Star => numeric_bin!(InstructionBuilder::build_mul, IntrinsicOperation::MulNumLR),
            TokenType::Slash => numeric_bin!(InstructionBuilder::build_div, IntrinsicOperation::DivNumLR),
            TokenType::Remainder => numeric_bin!(InstructionBuilder::build_rem, IntrinsicOperation::RemNumLR),
            TokenType::Exponentiation => numeric_bin!(InstructionBuilder::build_pow, IntrinsicOperation::PowNumLR),
            TokenType::Greater => numeric_bin_const_spec!(
                InstructionBuilder::build_gt,
                IntrinsicOperation::GtNumLR,
                u8 => InstructionBuilder::build_gt_numl_constr,
                u32 => InstructionBuilder::build_gt_numl_constr32
            ),
            TokenType::GreaterEqual => numeric_bin_const_spec!(
                InstructionBuilder::build_ge,
                IntrinsicOperation::GeNumLR,
                u8 => InstructionBuilder::build_ge_numl_constr,
                u32 => InstructionBuilder::build_ge_numl_constr32
            ),
            TokenType::Less => numeric_bin_const_spec!(
                InstructionBuilder::build_lt,
                IntrinsicOperation::LtNumLR,
                u8 => InstructionBuilder::build_lt_numl_constr,
                u32 => InstructionBuilder::build_lt_numl_constr32
            ),
            TokenType::LessEqual => numeric_bin_const_spec!(
                InstructionBuilder::build_le,
                IntrinsicOperation::LeNumLR,
                u8 => InstructionBuilder::build_le_numl_constr,
                u32 => InstructionBuilder::build_le_numl_constr32
            ),
            TokenType::Equality => numeric_bin!(InstructionBuilder::build_eq, IntrinsicOperation::EqNumLR),
            TokenType::Inequality => numeric_bin!(InstructionBuilder::build_ne, IntrinsicOperation::NeNumLR),
            TokenType::StrictEquality => numeric_bin!(InstructionBuilder::build_strict_eq, IntrinsicOperation::EqNumLR),
            TokenType::StrictInequality => {
                numeric_bin!(InstructionBuilder::build_strict_ne, IntrinsicOperation::NeNumLR)
            }
            TokenType::BitwiseOr => numeric_bin!(InstructionBuilder::build_bitor, IntrinsicOperation::BitOrNumLR),
            TokenType::BitwiseXor => numeric_bin!(InstructionBuilder::build_bitxor, IntrinsicOperation::BitXorNumLR),
            TokenType::BitwiseAnd => numeric_bin!(InstructionBuilder::build_bitand, IntrinsicOperation::BitAndNumLR),
            TokenType::LeftShift => numeric_bin!(InstructionBuilder::build_bitshl, IntrinsicOperation::BitShlNumLR),
            TokenType::RightShift => numeric_bin!(InstructionBuilder::build_bitshr, IntrinsicOperation::BitShrNumLR),
            TokenType::UnsignedRightShift => {
                numeric_bin!(InstructionBuilder::build_bitushr, IntrinsicOperation::BitUshrNumLR)
            }
            TokenType::In => generic_bin!(InstructionBuilder::build_objin),
            TokenType::Instanceof => generic_bin!(InstructionBuilder::build_instanceof),
            TokenType::LogicalOr => {
                ib.build_jmptruenp(Label::IfEnd, true);
                ib.build_pop(); // Only pop LHS if it is false
                ib.accept_expr(*right)?;
                ib.add_local_label(Label::IfEnd);
            }
            TokenType::LogicalAnd => {
                ib.build_jmpfalsenp(Label::IfEnd, true);
                ib.build_pop(); // Only pop LHS if it is true
                ib.accept_expr(*right)?;
                ib.add_local_label(Label::IfEnd);
            }
            TokenType::NullishCoalescing => {
                ib.build_jmpnullishnp(Label::IfBranch { branch_id: 0 }, true);
                ib.build_jmp(Label::IfEnd, true);

                ib.add_local_label(Label::IfBranch { branch_id: 0 });
                ib.build_pop();
                ib.accept_expr(*right)?;
                ib.add_local_label(Label::IfEnd);
            }
            other => unimplementedc!(span, "binary operator {:?}", other),
        }

        Ok(())
    }

    fn visit_expression_statement(&mut self, expr: Expr) -> Result<(), Error> {
        self.accept_expr(expr)?;
        InstructionBuilder::new(self).build_pop();
        Ok(())
    }

    fn visit_grouping_expression(&mut self, _span: Span, GroupingExpr(exprs): GroupingExpr) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        for expr in exprs {
            ib.accept_expr(expr)?;
            ib.build_pop();
        }

        ib.remove_pop_end();

        Ok(())
    }

    fn visit_literal_expression(&mut self, span: Span, expr: LiteralExpr) -> Result<(), Error> {
        let constant = Constant::from_literal(&expr);
        InstructionBuilder::new(self)
            .build_constant(constant)
            .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
        Ok(())
    }

    fn visit_identifier_expression(&mut self, span: Span, ident: Symbol) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        match ident {
            sym::this => ib.build_this(),
            sym::super_ => ib.build_super(),
            sym::globalThis => ib.build_global(),
            sym::Infinity => ib.build_infinity(),
            sym::NaN => ib.build_nan(),
            sym::arguments => {
                ib.current_function_mut().references_arguments = Some(span);
                ib.build_arguments();
            }
            ident => match ib.find_local(ident) {
                Some((index, _, is_extern)) => ib.build_local_load(index, is_extern),
                _ => ib
                    .build_global_load(ident)
                    .map_err(|_| Error::ConstantPoolLimitExceeded(span))?,
            },
        };

        Ok(())
    }

    fn visit_unary_expression(&mut self, span: Span, UnaryExpr { operator, expr }: UnaryExpr) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        // Special case delete operator, as it works different from other unary operators
        if let TokenType::Delete = operator {
            match expr.kind {
                ExprKind::PropertyAccess(PropertyAccessExpr {
                    computed,
                    property,
                    target,
                }) => match (*property, computed) {
                    (
                        Expr {
                            kind: ExprKind::Literal(LiteralExpr::Identifier(ident)),
                            span,
                        },
                        false,
                    ) => {
                        ib.accept_expr(*target)?;
                        let id = ib
                            .current_function_mut()
                            .cp
                            .add(Constant::Identifier(ident))
                            .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                        ib.build_static_delete(id);
                    }
                    (expr, _) => {
                        ib.accept_expr(expr)?;
                        ib.accept_expr(*target)?;
                        ib.build_dynamic_delete();
                    }
                },
                ExprKind::Literal(LiteralExpr::Identifier(ident)) => {
                    ib.build_global();
                    let id = ib
                        .current_function_mut()
                        .cp
                        .add(Constant::Identifier(ident))
                        .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                    ib.build_static_delete(id);
                }
                _ => {
                    ib.build_constant(Constant::Boolean(true))
                        .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                }
            }
            return Ok(());
        }

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
                if !matches!(ib.current_function().ty, FunctionKind::Generator) {
                    return Err(Error::YieldOutsideGenerator { yield_expr: span });
                }

                ib.build_yield();
            }
            TokenType::Await => {
                if !ib.current_function().r#async {
                    return Err(Error::AwaitOutsideAsync { await_expr: span });
                }

                ib.build_await();
            }
            _ => unimplementedc!(span, "unary operator {:?}", operator),
        }

        Ok(())
    }

    fn visit_variable_declaration(
        &mut self,
        span: Span,
        VariableDeclarations(declarations): VariableDeclarations,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        for VariableDeclaration { binding, value } in declarations {
            match binding.name {
                VariableDeclarationName::Identifier(ident) => {
                    // Type infer pass must have discovered the local variable
                    let (id, _) = ib.current_scope().find_local(ident).unwrap();

                    if let Some(expr) = value {
                        ib.accept_expr(expr)?;
                        ib.build_local_store(AssignKind::Assignment, id, false);
                        ib.build_pop();
                    }
                }
                VariableDeclarationName::ObjectDestructuring { fields, rest } => {
                    if rest.is_some() {
                        unimplementedc!(span, "rest operator in object destructuring");
                    }

                    let field_count = fields
                        .len()
                        .try_into()
                        .map_err(|_| Error::DestructureLimitExceeded(span))?;

                    let value = value.ok_or(Error::MissingInitializerInDestructuring(span))?;
                    ib.accept_expr(value)?;

                    ib.build_objdestruct(field_count);

                    for (name, alias) in fields {
                        let name = alias.unwrap_or(name);
                        let id = ib
                            .current_scope_mut()
                            .add_local(name, binding.kind, None)
                            .map_err(|_| Error::LocalLimitExceeded(span))?;

                        let var_id = ib
                            .current_function_mut()
                            .cp
                            .add(Constant::Number(id as f64))
                            .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                        let ident_id = ib
                            .current_function_mut()
                            .cp
                            .add(Constant::Identifier(name))
                            .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                        ib.writew(var_id);
                        ib.writew(ident_id);
                    }
                }
                VariableDeclarationName::ArrayDestructuring { fields, rest } => {
                    if rest.is_some() {
                        unimplementedc!(span, "rest operator in array destructuring");
                    }

                    let field_count = fields
                        .len()
                        .try_into()
                        .map_err(|_| Error::DestructureLimitExceeded(span))?;

                    let value = value.ok_or(Error::MissingInitializerInDestructuring(span))?;
                    ib.accept_expr(value)?;

                    ib.build_arraydestruct(field_count);

                    for name in fields {
                        let id = ib
                            .current_scope_mut()
                            .add_local(name, binding.kind, None)
                            .map_err(|_| Error::LocalLimitExceeded(span))?;

                        let var_id = ib
                            .current_function_mut()
                            .cp
                            .add(Constant::Number(id as f64))
                            .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                        ib.writew(var_id);
                    }
                }
            }
        }

        Ok(())
    }

    fn visit_if_statement(
        &mut self,
        _span: Span,
        IfStatement {
            condition,
            then,
            mut branches,
            el,
        }: IfStatement,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        // Desugar last `else` block into `else if(true)` for simplicity
        if let Some(then) = &el {
            let then = &**then;

            branches.push(IfStatement::new(
                Expr {
                    span: Span::COMPILER_GENERATED,
                    kind: ExprKind::bool_literal(true),
                },
                then.clone(),
                Vec::new(),
                None,
            ));
        }

        let len = branches.len();

        ib.accept_expr(condition)?;
        if branches.is_empty() {
            ib.build_jmpfalsep(Label::IfEnd, true);
        } else {
            ib.build_jmpfalsep(Label::IfBranch { branch_id: 0 }, true);
        }
        ib.accept(*then)?;
        ib.build_jmp(Label::IfEnd, true);

        for (id, branch) in branches.into_iter().enumerate() {
            ib.add_local_label(Label::IfBranch { branch_id: id });
            ib.accept_expr(branch.condition)?;
            if id == len - 1 {
                ib.build_jmpfalsep(Label::IfEnd, true);

                ib.accept(*branch.then)?;
            } else {
                ib.build_jmpfalsep(Label::IfBranch { branch_id: id + 1 }, true);

                ib.accept(*branch.then)?;

                ib.build_jmp(Label::IfEnd, true);
            }
        }

        ib.add_local_label(Label::IfEnd);
        Ok(())
    }

    fn visit_block_statement(&mut self, _span: Span, BlockStatement(stmt): BlockStatement) -> Result<(), Error> {
        self.current_scope_mut().enter();

        // Note: No `?` here because we need to always exit the scope
        let re = self.accept_multiple(stmt);
        self.current_scope_mut().exit();
        re
    }

    fn visit_function_declaration(&mut self, span: Span, fun: FunctionDeclaration) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);
        let var_id = match fun.name {
            Some(name) => Some(
                ib.current_scope_mut()
                    .add_local(name, VariableDeclarationKind::Var, None)
                    .map_err(|_| Error::LocalLimitExceeded(span))?,
            ),
            None => None,
        };

        ib.visit_function_expr(span, fun)?;
        if let Some(var_id) = var_id {
            ib.build_local_store(AssignKind::Assignment, var_id, false);
        }
        ib.build_pop();
        Ok(())
    }

    fn visit_while_loop(&mut self, _span: Span, WhileLoop { condition, body }: WhileLoop) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        let loop_id = ib.current_function_mut().prepare_loop();

        ib.current_function_mut()
            .add_global_label(Label::LoopCondition { loop_id });
        ib.accept_expr(condition)?;
        ib.build_jmpfalsep(Label::LoopEnd { loop_id }, false);

        ib.accept(*body)?;
        ib.build_jmp(Label::LoopCondition { loop_id }, false);

        ib.current_function_mut().add_global_label(Label::LoopEnd { loop_id });

        ib.current_function_mut().exit_loop();

        Ok(())
    }

    fn visit_do_while_loop(&mut self, _span: Span, DoWhileLoop { body, condition }: DoWhileLoop) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        let loop_id = ib.current_function_mut().prepare_loop();

        ib.current_function_mut()
            .add_global_label(Label::LoopCondition { loop_id });

        ib.accept(*body)?;

        ib.accept_expr(condition)?;
        ib.build_jmptruep(Label::LoopCondition { loop_id }, false);

        ib.current_function_mut().add_global_label(Label::LoopEnd { loop_id });
        ib.current_function_mut().exit_loop();

        Ok(())
    }

    #[rustfmt::skip]
    fn visit_assignment_expression(
        &mut self,
        span: Span,
        AssignmentExpr { left, right, operator }: AssignmentExpr,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        match left {
            AssignmentTarget::Expr(left) => match left.kind {
                ExprKind::Literal(LiteralExpr::Identifier(ident)) => {
                    let local = ib.find_local(ident);

                    if let Some((id, local, is_extern)) = local {
                        if matches!(local.binding().kind, VariableDeclarationKind::Const) {
                            return Err(Error::ConstAssignment(span));
                        }

                        macro_rules! assign {
                            ($kind:expr) => {{
                                ib.accept_expr(*right)?;
                                ib.build_local_store($kind, id, is_extern);
                            }};
                        }

                        match operator {
                            TokenType::Assignment => assign!(AssignKind::Assignment),
                            TokenType::AdditionAssignment => assign!(AssignKind::AddAssignment),
                            TokenType::SubtractionAssignment => assign!(AssignKind::SubAssignment),
                            TokenType::MultiplicationAssignment => assign!(AssignKind::MulAssignment),
                            TokenType::DivisionAssignment => assign!(AssignKind::DivAssignment),
                            TokenType::RemainderAssignment => assign!(AssignKind::RemAssignment),
                            TokenType::ExponentiationAssignment => assign!(AssignKind::PowAssignment),
                            TokenType::LeftShiftAssignment => assign!(AssignKind::ShlAssignment),
                            TokenType::RightShiftAssignment => assign!(AssignKind::ShrAssignment),
                            TokenType::UnsignedRightShiftAssignment => assign!(AssignKind::UshrAssignment),
                            TokenType::BitwiseAndAssignment => assign!(AssignKind::BitAndAssignment),
                            TokenType::BitwiseOrAssignment => assign!(AssignKind::BitOrAssignment),
                            TokenType::BitwiseXorAssignment => assign!(AssignKind::BitXorAssignment),
                            _ => unimplementedc!(span, "unknown assignment operator {}", operator.fmt_for_expected_tys()),
                        }
                    } else {
                        macro_rules! assign {
                            ($kind:expr) => {{
                                ib.accept_expr(*right)?;
                                ib.build_global_store($kind, ident)
                                    .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                            }};
                        }

                        match operator {
                            TokenType::Assignment => assign!(AssignKind::Assignment),
                            TokenType::AdditionAssignment => assign!(AssignKind::AddAssignment),
                            TokenType::SubtractionAssignment => assign!(AssignKind::SubAssignment),
                            TokenType::MultiplicationAssignment => assign!(AssignKind::MulAssignment),
                            TokenType::DivisionAssignment => assign!(AssignKind::DivAssignment),
                            TokenType::RemainderAssignment => assign!(AssignKind::RemAssignment),
                            TokenType::ExponentiationAssignment => assign!(AssignKind::PowAssignment),
                            TokenType::LeftShiftAssignment => assign!(AssignKind::ShlAssignment),
                            TokenType::RightShiftAssignment => assign!(AssignKind::ShrAssignment),
                            TokenType::UnsignedRightShiftAssignment => assign!(AssignKind::UshrAssignment),
                            TokenType::BitwiseAndAssignment => assign!(AssignKind::BitAndAssignment),
                            TokenType::BitwiseOrAssignment => assign!(AssignKind::BitOrAssignment),
                            TokenType::BitwiseXorAssignment => assign!(AssignKind::BitXorAssignment),
                            _ => unimplementedc!(span, "unknown assignment operator {}", operator.fmt_for_expected_tys()),
                        }
                    }
                }
                ExprKind::PropertyAccess(prop) => {
                    ib.accept_expr(*prop.target)?;

                    macro_rules! staticassign {
                        ($ident:expr, $kind:expr) => {{
                            ib.accept_expr(*right)?;
                            ib.build_static_prop_assign($kind, $ident)
                                .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                        }};
                    }
                    macro_rules! dynamicassign {
                        ($prop:expr, $kind:expr) => {{
                            ib.accept_expr(*right)?;
                            ib.accept_expr($prop)?;
                            ib.build_dynamic_prop_assign($kind);
                        }};
                    }

                    match (*prop.property, prop.computed, operator) {
                        (Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. }, false, TokenType::Assignment) => {
                            staticassign!(ident, AssignKind::Assignment)
                        }
                        (Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. }, false, TokenType::AdditionAssignment) => {
                            staticassign!(ident, AssignKind::AddAssignment)
                        }
                        (
                            Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. },
                            false,
                            TokenType::SubtractionAssignment,
                        ) => {
                            staticassign!(ident, AssignKind::SubAssignment)
                        }
                        (
                            Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. },
                            false,
                            TokenType::MultiplicationAssignment,
                        ) => {
                            staticassign!(ident, AssignKind::MulAssignment)
                        }
                        (Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. }, false, TokenType::DivisionAssignment) => {
                            staticassign!(ident, AssignKind::DivAssignment)
                        }
                        (Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. }, false, TokenType::RemainderAssignment) => {
                            staticassign!(ident, AssignKind::RemAssignment)
                        }
                        (
                            Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. },
                            false,
                            TokenType::ExponentiationAssignment,
                        ) => {
                            staticassign!(ident, AssignKind::PowAssignment)
                        }
                        (Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. }, false, TokenType::LeftShiftAssignment) => {
                            staticassign!(ident, AssignKind::ShlAssignment)
                        }
                        (Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. }, false, TokenType::RightShiftAssignment) => {
                            staticassign!(ident, AssignKind::ShrAssignment)
                        }
                        (
                            Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. },
                            false,
                            TokenType::UnsignedRightShiftAssignment,
                        ) => {
                            staticassign!(ident, AssignKind::UshrAssignment)
                        }
                        (Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. }, false, TokenType::BitwiseAndAssignment) => {
                            staticassign!(ident, AssignKind::BitAndAssignment)
                        }
                        (Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. }, false, TokenType::BitwiseOrAssignment) => {
                            staticassign!(ident, AssignKind::BitOrAssignment)
                        }
                        (Expr { kind:ExprKind::Literal(LiteralExpr::Identifier(ident)), .. }, false, TokenType::BitwiseXorAssignment) => {
                            staticassign!(ident, AssignKind::BitXorAssignment)
                        }
                        (expr, true, TokenType::Assignment) => dynamicassign!(expr, AssignKind::Assignment),
                        (expr, true, TokenType::AdditionAssignment) => {
                            dynamicassign!(expr, AssignKind::AddAssignment)
                        }
                        (expr, true, TokenType::SubtractionAssignment) => {
                            dynamicassign!(expr, AssignKind::SubAssignment)
                        }
                        (expr, true, TokenType::MultiplicationAssignment) => {
                            dynamicassign!(expr, AssignKind::MulAssignment)
                        }
                        (expr, true, TokenType::DivisionAssignment) => {
                            dynamicassign!(expr, AssignKind::DivAssignment)
                        }
                        (expr, true, TokenType::RemainderAssignment) => {
                            dynamicassign!(expr, AssignKind::RemAssignment)
                        }
                        (expr, true, TokenType::ExponentiationAssignment) => {
                            dynamicassign!(expr, AssignKind::PowAssignment)
                        }
                        (expr, true, TokenType::LeftShiftAssignment) => {
                            dynamicassign!(expr, AssignKind::ShlAssignment)
                        }
                        (expr, true, TokenType::RightShiftAssignment) => {
                            dynamicassign!(expr, AssignKind::ShrAssignment)
                        }
                        (expr, true, TokenType::UnsignedRightShiftAssignment) => {
                            dynamicassign!(expr, AssignKind::UshrAssignment)
                        }
                        (expr, true, TokenType::BitwiseAndAssignment) => {
                            dynamicassign!(expr, AssignKind::BitAndAssignment)
                        }
                        (expr, true, TokenType::BitwiseOrAssignment) => {
                            dynamicassign!(expr, AssignKind::BitOrAssignment)
                        }
                        (expr, true, TokenType::BitwiseXorAssignment) => {
                            dynamicassign!(expr, AssignKind::BitXorAssignment)
                        }
                        other => unimplementedc!(span, "assignment to computed property {other:?}"),
                    }
                }
                _ => unimplementedc!(span, "assignment to non-identifier"),
            },
            AssignmentTarget::LocalId(id) => {
                ib.accept_expr(*right)?;
                ib.build_local_store(AssignKind::Assignment, id, false);
            }
        }

        Ok(())
    }

    fn visit_function_call(
        &mut self,
        span: Span,
        FunctionCall {
            constructor_call,
            target,
            arguments,
        }: FunctionCall,
    ) -> Result<(), Error> {
        let target_span = target.span;
        let mut ib = InstructionBuilder::new(self);
        // TODO: this also needs to be specialized for assignment expressions with property access as target

        /// Attempts to specialize a function call
        ///
        /// For example, if the expression is `Math.max(a, b)`, then we can skip
        /// the overhead of a dynamic property lookup at runtime and emit a specialized `max` instruction.
        /// Of course, the VM still needs a guard to account for bad code messing with builtins, e.g.
        /// ```js
        /// let k = input(); // assume k = "max", black box to the compiler
        /// delete Math[k];
        ///
        /// Math.max(1, 2); // *should* throw a TypeError, but will not without a guard
        /// ```
        fn try_spec_function_call(
            ib: &mut InstructionBuilder<'_, '_>,
            target: &ExprKind,
            arguments: &[CallArgumentKind],
        ) -> Result<bool, Error> {
            if let ExprKind::PropertyAccess(PropertyAccessExpr { target, property, .. }) = target {
                let Some(target) = target.kind.as_identifier() else {
                    return Ok(false);
                };

                let Some(property) = property.kind.as_identifier() else {
                    return Ok(false);
                };

                let Ok(arg_len) = u8::try_from(arguments.len()) else {
                    return Ok(false);
                };

                macro_rules! emit_spec {
                    ($spec:expr) => {{
                        for arg in arguments {
                            if let CallArgumentKind::Normal(arg) = arg {
                                // TODO: we dont actually need to clone, we could take mem::take, if worth it
                                ib.accept_expr(arg.clone())?;
                            } else {
                                // Can't specialize spread args for now
                                return Ok(false);
                            }
                        }
                        $spec(ib, arg_len);
                        return Ok(true);
                    }};
                }

                match (target, property) {
                    (sym::Math, sym::exp) => emit_spec!(InstructionBuilder::build_exp),
                    (sym::Math, sym::log2) => emit_spec!(InstructionBuilder::build_log2),
                    (sym::Math, sym::expm1) => emit_spec!(InstructionBuilder::build_expm1),
                    (sym::Math, sym::cbrt) => emit_spec!(InstructionBuilder::build_cbrt),
                    (sym::Math, sym::clz32) => emit_spec!(InstructionBuilder::build_clz32),
                    (sym::Math, sym::atanh) => emit_spec!(InstructionBuilder::build_atanh),
                    (sym::Math, sym::atan2) => emit_spec!(InstructionBuilder::build_atanh2),
                    (sym::Math, sym::round) => emit_spec!(InstructionBuilder::build_round),
                    (sym::Math, sym::acosh) => emit_spec!(InstructionBuilder::build_acosh),
                    (sym::Math, sym::abs) => emit_spec!(InstructionBuilder::build_abs),
                    (sym::Math, sym::sinh) => emit_spec!(InstructionBuilder::build_sinh),
                    (sym::Math, sym::sin) => emit_spec!(InstructionBuilder::build_sin),
                    (sym::Math, sym::ceil) => emit_spec!(InstructionBuilder::build_ceil),
                    (sym::Math, sym::tan) => emit_spec!(InstructionBuilder::build_tan),
                    (sym::Math, sym::trunc) => emit_spec!(InstructionBuilder::build_trunc),
                    (sym::Math, sym::asinh) => emit_spec!(InstructionBuilder::build_asinh),
                    (sym::Math, sym::log10) => emit_spec!(InstructionBuilder::build_log10),
                    (sym::Math, sym::asin) => emit_spec!(InstructionBuilder::build_asin),
                    (sym::Math, sym::random) => emit_spec!(InstructionBuilder::build_random),
                    (sym::Math, sym::log1p) => emit_spec!(InstructionBuilder::build_log1p),
                    (sym::Math, sym::sqrt) => emit_spec!(InstructionBuilder::build_sqrt),
                    (sym::Math, sym::atan) => emit_spec!(InstructionBuilder::build_atan),
                    (sym::Math, sym::log) => emit_spec!(InstructionBuilder::build_log),
                    (sym::Math, sym::floor) => emit_spec!(InstructionBuilder::build_floor),
                    (sym::Math, sym::cosh) => emit_spec!(InstructionBuilder::build_cosh),
                    (sym::Math, sym::acos) => emit_spec!(InstructionBuilder::build_acos),
                    (sym::Math, sym::cos) => emit_spec!(InstructionBuilder::build_cos),
                    _ => {}
                }
            }
            Ok(false)
        }

        if try_spec_function_call(&mut ib, &target.kind, &arguments)? {
            return Ok(());
        }

        let has_this = if let ExprKind::PropertyAccess(p) = target.kind {
            ib.visit_property_access_expr(target.span, p, true)?;
            true
        } else {
            ib.accept_expr(*target)?;
            false
        };

        let argc = arguments.len();

        let mut spread_arg_indices = Vec::new();

        for (index, arg) in arguments.into_iter().enumerate() {
            match arg {
                CallArgumentKind::Normal(expr) => {
                    ib.accept_expr(expr)?;
                }
                CallArgumentKind::Spread(expr) => {
                    ib.accept_expr(expr)?;
                    spread_arg_indices.push(index.try_into().unwrap());
                }
            }
        }

        let meta = FunctionCallMetadata::new_checked(argc, constructor_call, has_this)
            .ok_or(Error::ParameterLimitExceeded(span))?;

        ib.build_call(meta, spread_arg_indices, target_span);

        Ok(())
    }

    fn visit_return_statement(&mut self, _span: Span, ReturnStatement(stmt): ReturnStatement) -> Result<(), Error> {
        let tc_depth = self.current_function().try_catch_depth;
        self.accept_expr(stmt)?;
        InstructionBuilder::new(self).build_ret(tc_depth);
        Ok(())
    }

    fn visit_conditional_expr(
        &mut self,
        _span: Span,
        ConditionalExpr { condition, then, el }: ConditionalExpr,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        ib.accept_expr(*condition)?;
        ib.build_jmpfalsep(Label::IfBranch { branch_id: 0 }, true);

        ib.accept_expr(*then)?;
        ib.build_jmp(Label::IfEnd, true);

        ib.add_local_label(Label::IfBranch { branch_id: 0 });
        ib.accept_expr(*el)?;

        ib.add_local_label(Label::IfEnd);
        Ok(())
    }

    fn visit_property_access_expr(
        &mut self,
        _span: Span,
        PropertyAccessExpr {
            computed,
            target,
            property,
        }: PropertyAccessExpr,
        preserve_this: bool,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        ib.accept_expr(*target)?;

        match (*property, computed) {
            (
                Expr {
                    kind: ExprKind::Literal(LiteralExpr::Identifier(ident)),
                    span,
                },
                false,
            ) => {
                ib.build_static_prop_access(ident, preserve_this)
                    .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
            }
            (e, _) => {
                ib.accept_expr(e)?;
                ib.build_dynamic_prop_access(preserve_this);
            }
        }

        Ok(())
    }

    fn visit_sequence_expr(&mut self, _span: Span, (expr1, expr2): Seq) -> Result<(), Error> {
        self.accept_expr(*expr1)?;
        InstructionBuilder::new(self).build_pop();
        self.accept_expr(*expr2)?;

        Ok(())
    }

    fn visit_postfix_expr(&mut self, span: Span, (tt, expr): Postfix) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        match expr.kind {
            ExprKind::Literal(LiteralExpr::Identifier(ident)) => {
                if let Some((id, loc, is_extern)) = ib.find_local(ident) {
                    let ty = loc.inferred_type().borrow();

                    // Specialize guaranteed local number increment
                    if let (Ok(id), Some(CompileValueType::Number), false) = (u8::try_from(id), &*ty, is_extern) {
                        match tt {
                            TokenType::Increment => ib.build_postfix_inc_local_num(id),
                            TokenType::Decrement => ib.build_postfix_dec_local_num(id),
                            _ => unreachable!("Token never emitted"),
                        }
                        return Ok(());
                    }

                    match tt {
                        TokenType::Increment => ib.build_local_store(AssignKind::PostfixIncrement, id, is_extern),
                        TokenType::Decrement => ib.build_local_store(AssignKind::PostfixDecrement, id, is_extern),
                        _ => unreachable!("Token never emitted"),
                    }
                } else {
                    match tt {
                        TokenType::Increment => ib
                            .build_global_store(AssignKind::PostfixIncrement, ident)
                            .map_err(|_| Error::ConstantPoolLimitExceeded(expr.span))?,
                        TokenType::Decrement => ib
                            .build_global_store(AssignKind::PostfixDecrement, ident)
                            .map_err(|_| Error::ConstantPoolLimitExceeded(expr.span))?,
                        _ => unreachable!("Token never emitted"),
                    }
                }
            }
            ExprKind::PropertyAccess(prop) => {
                ib.accept_expr(*prop.target)?;

                match (*prop.property, prop.computed) {
                    (
                        Expr {
                            kind: ExprKind::Literal(LiteralExpr::Identifier(ident)),
                            span,
                        },
                        false,
                    ) => match tt {
                        TokenType::Increment => ib
                            .build_static_prop_assign(AssignKind::PostfixIncrement, ident)
                            .map_err(|_| Error::ConstantPoolLimitExceeded(span))?,
                        TokenType::Decrement => ib
                            .build_static_prop_assign(AssignKind::PostfixDecrement, ident)
                            .map_err(|_| Error::ConstantPoolLimitExceeded(span))?,
                        _ => unreachable!("Token never emitted"),
                    },
                    (prop, true) => {
                        ib.accept_expr(prop)?;
                        match tt {
                            TokenType::Increment => ib.build_dynamic_prop_assign(AssignKind::PostfixIncrement),
                            TokenType::Decrement => ib.build_dynamic_prop_assign(AssignKind::PostfixDecrement),
                            _ => unreachable!("Token never emitted"),
                        }
                    }
                    _ => unreachable!("Static assignment was not a literal"),
                }
            }
            _ => unimplementedc!(span, "non-identifier postfix expression"),
        }

        Ok(())
    }

    fn visit_prefix_expr(&mut self, span: Span, (tt, expr): Postfix) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        match expr.kind {
            ExprKind::Literal(LiteralExpr::Identifier(ident)) => {
                if let Some((id, loc, is_extern)) = ib.find_local(ident) {
                    let ty = loc.inferred_type().borrow();

                    // Specialize guaranteed local number increment
                    if let (Ok(id), Some(CompileValueType::Number), false) = (u8::try_from(id), &*ty, is_extern) {
                        match tt {
                            TokenType::Increment => ib.build_prefix_inc_local_num(id),
                            TokenType::Decrement => ib.build_prefix_dec_local_num(id),
                            _ => unreachable!("Token never emitted"),
                        }
                        return Ok(());
                    }

                    match tt {
                        TokenType::Increment => ib.build_local_store(AssignKind::PrefixIncrement, id, is_extern),
                        TokenType::Decrement => ib.build_local_store(AssignKind::PrefixDecrement, id, is_extern),
                        _ => unreachable!("Token never emitted"),
                    }
                } else {
                    match tt {
                        TokenType::Increment => ib
                            .build_global_store(AssignKind::PrefixIncrement, ident)
                            .map_err(|_| Error::ConstantPoolLimitExceeded(expr.span))?,
                        TokenType::Decrement => ib
                            .build_global_store(AssignKind::PrefixDecrement, ident)
                            .map_err(|_| Error::ConstantPoolLimitExceeded(expr.span))?,
                        _ => unreachable!("Token never emitted"),
                    }
                }
            }
            ExprKind::PropertyAccess(prop) => {
                ib.accept_expr(*prop.target)?;

                match (*prop.property, prop.computed) {
                    (
                        Expr {
                            kind: ExprKind::Literal(LiteralExpr::Identifier(ident)),
                            span,
                        },
                        false,
                    ) => match tt {
                        TokenType::Increment => ib
                            .build_static_prop_assign(AssignKind::PrefixIncrement, ident)
                            .map_err(|_| Error::ConstantPoolLimitExceeded(span))?,
                        TokenType::Decrement => ib
                            .build_static_prop_assign(AssignKind::PrefixDecrement, ident)
                            .map_err(|_| Error::ConstantPoolLimitExceeded(span))?,
                        _ => unreachable!("Token never emitted"),
                    },
                    (prop, true) => {
                        ib.accept_expr(prop)?;
                        match tt {
                            TokenType::Increment => ib.build_dynamic_prop_assign(AssignKind::PrefixIncrement),
                            TokenType::Decrement => ib.build_dynamic_prop_assign(AssignKind::PrefixDecrement),
                            _ => unreachable!("Token never emitted"),
                        }
                    }
                    _ => unreachable!("Static assignment was not a literal"),
                }
            }
            _ => unimplementedc!(span, "non-identifier postfix expression"),
        }

        Ok(())
    }

    fn visit_function_expr(
        &mut self,
        span: Span,
        FunctionDeclaration {
            id,
            name,
            parameters: arguments,
            mut statements,
            ty,
            r#async,
            ty_segment: _,
        }: FunctionDeclaration,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);
        ib.function_stack.push(FunctionLocalState::new(ty, id, r#async));

        let mut rest_local = None;

        for (param, default, _ty) in &arguments {
            let name = match *param {
                Parameter::Identifier(ident) => ident,
                Parameter::Spread(ident) => ident,
            };

            let id = ib
                .tcx
                .scope_mut(id)
                .add_local(name, VariableDeclarationKind::Var, None)
                .map_err(|_| Error::LocalLimitExceeded(span))?;

            if let Parameter::Spread(..) = param {
                rest_local = Some(id);
            }

            if let Some(default) = default {
                let mut sub_ib = InstructionBuilder::new(&mut ib);
                // First, load parameter
                sub_ib.build_local_load(id, false);
                // Jump to InitParamWithDefaultValue if param is undefined
                sub_ib.build_jmpundefinedp(Label::InitParamWithDefaultValue, true);
                // If it isn't undefined, it won't jump to InitParamWithDefaultValue, so we jump to the end
                sub_ib.build_jmp(Label::FinishParamDefaultValueInit, true);
                sub_ib.add_local_label(Label::InitParamWithDefaultValue);
                sub_ib.accept_expr(default.clone())?;
                sub_ib.build_local_store(AssignKind::Assignment, id, false);

                sub_ib.add_local_label(Label::FinishParamDefaultValueInit);
            }
        }

        transformations::hoist_declarations(&mut statements);
        transformations::ast_insert_implicit_return(&mut statements);
        let res: Result<(), Error> = (|| {
            for stmt in statements {
                ib.accept(stmt)?;
            }
            Ok(())
        })();

        let cmp = ib.function_stack.pop().expect("Missing function state");
        res?; // Cannot early return error in the loop as we need to pop the function state in any case
        let scope = ib.tcx.scope(id);
        let externals = scope.externals();
        let locals = scope.locals().len();

        let function = Function {
            buffer: Buffer(Cell::new(cmp.buf.into())),
            constants: cmp.cp.into_vec().into(),
            locals,
            name,
            ty,
            params: match arguments.last() {
                Some((Parameter::Spread(..), ..)) => arguments.len() - 1,
                _ => arguments.len(),
            },
            externals: externals.into(),
            r#async,
            rest_local,
            poison_ips: RefCell::new(HashSet::new()),
            debug_symbols: cmp.debug_symbols,
            source: Rc::clone(&ib.source),
            references_arguments: cmp.references_arguments.is_some(),
        };
        ib.build_constant(Constant::Function(Rc::new(function)))
            .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;

        Ok(())
    }

    fn visit_array_literal(&mut self, span: Span, ArrayLiteral(exprs): ArrayLiteral) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        let len = exprs.len().try_into().map_err(|_| Error::ArrayLitLimitExceeded(span))?;

        let kinds = exprs
            .iter()
            .map(|kind| dash_middle::compiler::ArrayMemberKind::from(kind) as u8)
            .collect::<Vec<u8>>();

        for kind in exprs {
            match kind {
                ArrayMemberKind::Item(expr) => {
                    ib.accept_expr(expr)?;
                }
                ArrayMemberKind::Spread(expr) => {
                    ib.accept_expr(expr)?;
                }
            }
        }

        ib.build_arraylit(len);
        ib.write_all(&kinds);
        Ok(())
    }

    fn visit_object_literal(&mut self, span: Span, ObjectLiteral(exprs): ObjectLiteral) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        let mut members = Vec::with_capacity(exprs.len());
        for (member, value) in exprs {
            ib.accept_expr(value)?;

            if let ObjectMemberKind::Dynamic(expr) = member {
                // TODO: do not clone, the `expr` is not needed in ib.build_objlit
                members.push(ObjectMemberKind::Dynamic(expr.clone()));
                ib.accept_expr(expr)?;
            } else {
                members.push(member);
            }
        }

        ib.build_objlit(span, members)?;
        Ok(())
    }

    fn visit_try_catch(&mut self, span: Span, TryCatch { try_, catch, .. }: TryCatch) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        ib.build_try_block();

        ib.current_function_mut().try_catch_depth += 1;
        ib.current_scope_mut().enter();
        let res = ib.accept(*try_); // TODO: some API for making this nicer
        ib.current_scope_mut().exit();
        ib.current_function_mut().try_catch_depth -= 1;
        res?;

        ib.build_jmp(Label::TryEnd, true);

        ib.add_local_label(Label::Catch);

        ib.current_scope_mut().enter();

        if let Some(ident) = catch.ident {
            let id = ib
                .current_scope_mut()
                .add_local(ident, VariableDeclarationKind::Var, None)
                .map_err(|_| Error::LocalLimitExceeded(span))?;

            if id == u16::MAX {
                // Max u16 value is reserved for "no binding"
                return Err(Error::LocalLimitExceeded(span));
            }

            ib.writew(id);
        } else {
            ib.writew(u16::MAX);
        }

        ib.accept(*catch.body)?;
        ib.current_scope_mut().exit();

        ib.add_local_label(Label::TryEnd);
        ib.build_try_end();

        Ok(())
    }

    fn visit_throw(&mut self, _span: Span, expr: Expr) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);
        ib.accept_expr(expr)?;
        ib.build_throw();
        Ok(())
    }

    fn visit_for_loop(
        &mut self,
        _span: Span,
        ForLoop {
            init,
            condition,
            finalizer,
            body,
        }: ForLoop,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);
        ib.current_scope_mut().enter();

        // Initialization
        if let Some(init) = init {
            ib.accept(*init)?;
        }

        let loop_id = ib.current_function_mut().prepare_loop();

        // Condition
        ib.current_function_mut()
            .add_global_label(Label::LoopCondition { loop_id });
        if let Some(condition) = condition {
            ib.accept_expr(condition)?;
            ib.build_jmpfalsep(Label::LoopEnd { loop_id }, false);
        }

        // Body
        ib.accept(*body)?;

        // Increment
        ib.current_function_mut()
            .add_global_label(Label::LoopIncrement { loop_id });
        if let Some(finalizer) = finalizer {
            ib.accept_expr(finalizer)?;
            ib.build_pop();
        }
        ib.build_jmp(Label::LoopCondition { loop_id }, false);

        ib.current_function_mut().add_global_label(Label::LoopEnd { loop_id });
        ib.current_scope_mut().exit();
        ib.current_function_mut().exit_loop();

        Ok(())
    }

    fn visit_for_of_loop(&mut self, _span: Span, ForOfLoop { binding, expr, body }: ForOfLoop) -> Result<(), Error> {
        self.visit_for_each_kinded_loop(ForEachLoopKind::ForOf, binding, expr, body)
    }

    fn visit_for_in_loop(&mut self, _span: Span, ForInLoop { binding, expr, body }: ForInLoop) -> Result<(), Error> {
        self.visit_for_each_kinded_loop(ForEachLoopKind::ForIn, binding, expr, body)
    }

    fn visit_import_statement(&mut self, span: Span, import: ImportKind) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        match import {
            ImportKind::Dynamic(ex) => {
                ib.accept_expr(ex)?;
                ib.build_dynamic_import();
            }
            ref kind @ (ImportKind::DefaultAs(ref spec, path) | ImportKind::AllAs(ref spec, path)) => {
                let local_id = ib
                    .current_scope_mut()
                    .add_local(
                        match spec {
                            SpecifierKind::Ident(id) => *id,
                        },
                        VariableDeclarationKind::Var,
                        None,
                    )
                    .map_err(|_| Error::LocalLimitExceeded(span))?;

                let path_id = ib
                    .current_function_mut()
                    .cp
                    .add(Constant::String(path))
                    .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;

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

    fn visit_export_statement(&mut self, span: Span, export: ExportKind) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        match export {
            ExportKind::Default(expr) => {
                ib.accept_expr(expr)?;
                ib.build_default_export();
            }
            ExportKind::Named(names) => {
                let mut it = Vec::with_capacity(names.len());

                for name in names.iter().copied() {
                    let ident_id = ib
                        .current_function_mut()
                        .cp
                        .add(Constant::Identifier(name))
                        .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;

                    match ib.find_local(name) {
                        Some((loc_id, _, is_extern)) => {
                            // Top level exports shouldn't be able to refer to extern locals
                            assert!(!is_extern);

                            it.push(NamedExportKind::Local { loc_id, ident_id });
                        }
                        None => {
                            it.push(NamedExportKind::Global { ident_id });
                        }
                    }
                }

                ib.build_named_export(span, &it)?;
            }
            ExportKind::NamedVar(VariableDeclarations(vars)) => {
                let mut it = Vec::with_capacity(vars.len());

                for var in &vars {
                    match var.binding.name {
                        VariableDeclarationName::Identifier(ident) => it.push(ident),
                        VariableDeclarationName::ArrayDestructuring { ref fields, rest } => {
                            it.extend(fields.iter().copied());
                            it.extend(rest);
                        }
                        VariableDeclarationName::ObjectDestructuring { ref fields, rest } => {
                            it.extend(fields.iter().map(|&(name, ident)| ident.unwrap_or(name)));
                            it.extend(rest);
                        }
                    }
                }

                self.visit_variable_declaration(span, VariableDeclarations(vars))?;
                self.visit_export_statement(span, ExportKind::Named(it))?;
            }
        };
        Ok(())
    }

    fn visit_empty_statement(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn visit_break(&mut self, span: Span) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);
        let breakable = *ib
            .current_function_mut()
            .breakables
            .last()
            .ok_or(Error::IllegalBreak(span))?;

        match breakable {
            Breakable::Loop { loop_id } => {
                ib.build_jmp(Label::LoopEnd { loop_id }, false);
            }
            Breakable::Switch { switch_id } => {
                ib.build_jmp(Label::SwitchEnd { switch_id }, false);
            }
        }
        Ok(())
    }

    fn visit_continue(&mut self, span: Span) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);
        let breakable = *ib
            .current_function_mut()
            .breakables
            .last()
            .ok_or(Error::IllegalBreak(span))?;

        match breakable {
            Breakable::Loop { loop_id } => {
                ib.build_jmp(Label::LoopIncrement { loop_id }, false);
            }
            Breakable::Switch { .. } => {
                // TODO: make it possible to use `continue` in loops even if its used in a switch
                unimplementedc!(span, "`continue` used inside of a switch statement");
            }
        }
        Ok(())
    }

    fn visit_debugger(&mut self, _span: Span) -> Result<(), Error> {
        InstructionBuilder::new(self).build_debugger();
        Ok(())
    }

    fn visit_empty_expr(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn visit_class_declaration(&mut self, span: Span, class: Class) -> Result<(), Error> {
        if class.extends.is_some() {
            unimplementedc!(span, "extending classes");
        }

        let mut ib = InstructionBuilder::new(self);

        let constructor = class.members.iter().find_map(|member| {
            if let ClassMemberKind::Method(method) = &member.kind {
                if method.name == Some(sym::constructor) {
                    return Some(method.clone());
                }
            }

            None
        });

        let binding_id = match class.name {
            Some(name) => ib
                .current_scope_mut()
                .add_local(name, VariableDeclarationKind::Var, None)
                .map_err(|_| Error::LocalLimitExceeded(span))?,
            None => ib
                .current_scope_mut()
                .add_local(sym::DesugaredClass, VariableDeclarationKind::Unnameable, None)
                .map_err(|_| Error::LocalLimitExceeded(span))?,
        };

        let (parameters, mut statements, id) = match constructor {
            Some(fun) => (fun.parameters, fun.statements, fun.id),
            None => {
                let parent = ib.current_function().id;
                (Vec::new(), Vec::new(), ib.tcx.add_scope(Some(parent)))
            }
        };

        transformations::insert_initializer_in_constructor(&class, &mut statements);

        let desugared_class = FunctionDeclaration {
            id,
            name: class.name,
            parameters,
            statements,
            ty: FunctionKind::Function,
            r#async: false,
            ty_segment: None,
        };

        ib.visit_assignment_expression(
            span,
            AssignmentExpr::new_local_place(
                binding_id,
                Expr {
                    span: Span::COMPILER_GENERATED, // TODO: we might have to use a real span here?
                    kind: ExprKind::Function(desugared_class),
                },
                TokenType::Assignment,
            ),
        )?;
        let load_class_binding = Expr {
            span: Span::COMPILER_GENERATED,
            kind: ExprKind::Compiled(compile_local_load(binding_id, false)),
        };

        for member in class.members {
            if let ClassMemberKind::Method(method) = member.kind {
                let name = method.name.expect("Class method did not have a name");

                ib.accept(Statement {
                    span: Span::COMPILER_GENERATED,
                    kind: StatementKind::Expression(Expr {
                        span: Span::COMPILER_GENERATED,
                        kind: ExprKind::Assignment(AssignmentExpr {
                            left: AssignmentTarget::Expr(Box::new(match member.static_ {
                                true => Expr {
                                    span: Span::COMPILER_GENERATED,
                                    kind: ExprKind::property_access(
                                        false,
                                        load_class_binding.clone(),
                                        Expr {
                                            span: Span::COMPILER_GENERATED,
                                            kind: ExprKind::identifier(name),
                                        },
                                    ),
                                },
                                false => Expr {
                                    span: Span::COMPILER_GENERATED,
                                    kind: ExprKind::property_access(
                                        false,
                                        Expr {
                                            span: Span::COMPILER_GENERATED,
                                            kind: ExprKind::property_access(
                                                false,
                                                load_class_binding.clone(),
                                                Expr {
                                                    span: Span::COMPILER_GENERATED,
                                                    kind: ExprKind::identifier(sym::prototype),
                                                },
                                            ),
                                        },
                                        Expr {
                                            span: Span::COMPILER_GENERATED,
                                            kind: ExprKind::identifier(name),
                                        },
                                    ),
                                },
                            })),
                            right: Box::new(Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::Function(method),
                            }),
                            operator: TokenType::Assignment,
                        }),
                    }),
                })?;
            }
        }
        ib.build_pop();

        Ok(())
    }

    fn visit_switch_statement(
        &mut self,
        span: Span,
        SwitchStatement { expr, cases, default }: SwitchStatement,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);
        let switch_id = ib.current_function_mut().prepare_switch();

        // TODO: this currently always calls the naive implementation.
        // We can be smarter in certain cases, such as if all cases are literals,
        // we can create some kind of table.
        compile_switch_naive(&mut ib, span, expr, cases, default)?;

        ib.current_function_mut()
            .add_global_label(Label::SwitchEnd { switch_id });
        ib.current_function_mut().exit_switch();

        Ok(())
    }
}

/// "Naive" switch lowering:
/// ```js
/// switch(1) {
///     case w: /* FALLTHROUGH */
///     case x:
///         return 'case 1';
///         break;
///     case y:
///         return 'case 2';
///         break;
///     default:
///         return 'default';
/// }
/// ```
/// to
/// ```
/// _1 = condition
///
/// case_w_cond:
///     eq (ld _1 ld w)
///     jmpfalsep case_x_cond
///     # code for case w
///		# if there's a break anywhere in here, jump to end of switch
///     jmp case_x
///
///	case_x_cond:
///		eq (ld_1 ld x)
///		jmpfalsep case_y_cond
///  case_x:
///     # code for case x
///		constant 'case 1'
///		ret
///		jmp case_y
///
/// case_x_code:
///     ...
/// case_y_cond:
///		eq (ld_1 ld y)
///		jmpfalsep default
///	case y:
/// 	...
///		jmp default
///
/// default:
///     ...
/// ```
fn compile_switch_naive(
    ib: &mut InstructionBuilder<'_, '_>,
    span: Span,
    condition: Expr,
    cases: Vec<SwitchCase>,
    default: Option<Vec<Statement>>,
) -> Result<(), Error> {
    let condition_id = ib
        .current_scope_mut()
        .add_local(sym::switch_cond_desugar, VariableDeclarationKind::Unnameable, None)
        .map_err(|_| Error::LocalLimitExceeded(span))?;

    // Store condition in temporary
    ib.accept_expr(condition)?;
    ib.build_local_store(AssignKind::Assignment, condition_id, false);
    ib.build_pop();

    let local_load = compile_local_load(condition_id, false);

    let case_count = cases.len().try_into().unwrap();

    for (case_id, case) in cases.into_iter().enumerate() {
        let case_id = u16::try_from(case_id).unwrap();
        ib.add_local_label(Label::SwitchCaseCondition { case_id });

        let eq = Expr::binary(
            Expr {
                kind: ExprKind::compiled(local_load.clone()),
                span,
            },
            case.value,
            TokenType::Equality,
        );
        ib.accept_expr(eq)?;
        ib.build_jmpfalsep(Label::SwitchCaseCondition { case_id: case_id + 1 }, true);

        ib.add_local_label(Label::SwitchCaseCode { case_id });
        ib.accept_multiple(case.body)?;
        ib.build_jmp(Label::SwitchCaseCode { case_id: case_id + 1 }, true);
    }

    ib.add_local_label(Label::SwitchCaseCondition { case_id: case_count });
    ib.add_local_label(Label::SwitchCaseCode { case_id: case_count });
    if let Some(default) = default {
        ib.accept_multiple(default)?;
    }

    Ok(())
}
