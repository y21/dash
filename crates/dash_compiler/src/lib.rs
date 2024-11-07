use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use dash_log::{debug, span, Level};
use dash_middle::compiler::constant::{Buffer, ConstantPool, Function, NumberConstant, SymbolConstant};
use dash_middle::compiler::external::External;
use dash_middle::compiler::instruction::{AssignKind, Instruction, IntrinsicOperation};
use dash_middle::compiler::scope::{CompileValueType, LimitExceededError, Local, ScopeGraph};
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
    Asyncness, Binding, BlockStatement, Class, ClassMember, ClassMemberKey, ClassMemberValue, DoWhileLoop, ExportKind,
    ForInLoop, ForLoop, ForOfLoop, FunctionDeclaration, FunctionKind, IfStatement, ImportKind, Loop, Parameter,
    Pattern, ReturnStatement, ScopeId, SpecifierKind, Statement, StatementKind, SwitchCase, SwitchStatement, TryCatch,
    VariableDeclaration, VariableDeclarationKind, VariableDeclarationName, VariableDeclarations, WhileLoop,
};
use dash_middle::sourcemap::Span;
use dash_middle::util::Counter;
use dash_middle::visitor::Visitor;
use dash_optimizer::consteval::ConstFunctionEvalCtx;
use dash_optimizer::type_infer::{InferMode, LocalDeclToSlot, NameResolutionResults, TypeInferCtx};
use dash_optimizer::OptLevel;
use for_each_loop::{ForEachDesugarCtxt, ForEachLoopKind};
use instruction::compile_local_load;
use jump_container::JumpContainer;

use crate::builder::{InstructionBuilder, Label};

use self::instruction::NamedExportKind;

pub mod builder;
mod for_each_loop;
#[cfg(feature = "from_string")]
pub mod from_string;
pub mod instruction;
pub mod transformations;

mod jump_container;

macro_rules! unimplementedc {
    ($span:expr,$($what:expr),*) => {
        return Err(Error::Unimplemented($span,format_args!($($what),*).to_string()))
    };
}

#[derive(Debug, Clone, Copy)]
enum Breakable {
    Loop { loop_id: usize, label: Option<Symbol> },
    Switch { switch_id: usize },
    Named { sym: Symbol, label_id: usize },
}

/// Function-specific state, such as
#[derive(Debug)]
struct FunctionLocalState {
    /// Instruction buffer
    buf: Vec<u8>,
    /// A list of constants used throughout this function
    cp: ConstantPool,
    /// Current `try` depth (note that this does NOT include `catch`es)
    try_depth: u16,
    /// A stack of try-catch-finally blocks and their optional `finally` label that can be jumped to
    finally_labels: Vec<Option<Label>>,
    finally_counter: Counter<usize>,
    /// Counter for user-defined labels
    user_label_counter: Counter<usize>,
    /// The type of function that this FunctionCompiler compiles
    ty: FunctionKind,
    /// Container, used for storing global labels that can be jumped to
    jc: JumpContainer,
    /// A stack of breakable labels (loop/switch)
    breakables: Vec<Breakable>,
    /// Keeps track of the total number of loops to be able to have unique IDs
    loop_counter: usize,
    /// Keeps track of the total number of loops to be able to have unique IDs
    switch_counter: usize,
    id: ScopeId,
    debug_symbols: DebugSymbols,
    externals: Vec<External>,
    /// Whether this function references `arguments` anywhere in its body
    ///
    /// Also tracks the span for error reporting, but is discarded past the compiler stage.
    references_arguments: Option<Span>,
}

macro_rules! exit_breakable {
    ($fc:expr, $what:pat) => {
        match $fc.breakables.pop() {
            Some($what) => {}
            _ => panic!("Tried to exit breakable, but wrong kind was on the stack"),
        }
    };
}

#[derive(Copy, Clone)]
enum BreakStmt {
    Break,
    Continue,
}

impl FunctionLocalState {
    pub fn new(ty: FunctionKind, id: ScopeId) -> Self {
        Self {
            buf: Vec::new(),
            cp: ConstantPool::default(),
            try_depth: 0,
            finally_labels: Vec::new(),
            finally_counter: Counter::new(),
            user_label_counter: Counter::new(),
            ty,
            jc: JumpContainer::new(),
            breakables: Vec::new(),
            loop_counter: 0,
            switch_counter: 0,
            id,
            debug_symbols: DebugSymbols::default(),
            externals: Vec::new(),
            references_arguments: None,
        }
    }

    /// "Prepares" a loop and returns a unique ID that identifies this loop
    ///
    /// Specifically, this function increments a FunctionCompiler-local loop counter and
    /// inserts the loop into a stack of switch-case/loops so that `break` (and `continue`)
    /// statements can be resolved at compile-time
    fn prepare_loop(&mut self, label: Option<Symbol>) -> usize {
        let loop_id = self.loop_counter;
        self.breakables.push(Breakable::Loop { loop_id, label });
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

    fn add_global_label(&mut self, label: Label) {
        jump_container::add_label(&mut self.jc, label, &mut self.buf)
    }

    /// Jumps to a label that was previously (or will be) created by a call to `add_global_label`
    fn add_global_jump(&mut self, label: Label) {
        jump_container::add_jump(&mut self.jc, label, &mut self.buf)
    }

    /// Tries to find the target to jump to for a `break` (or `continue`).
    fn find_breakable(&self, label: Option<Symbol>, brk_stmt: BreakStmt) -> Option<Breakable> {
        self.breakables
            .iter()
            .rev()
            .find(|brk| match (brk, brk_stmt, label) {
                (
                    Breakable::Named { sym, label_id: _ }
                    | Breakable::Loop {
                        label: Some(sym),
                        loop_id: _,
                    },
                    _,
                    Some(sym2),
                ) => *sym == sym2,
                // 'x: while(true) { continue; }
                (Breakable::Named { .. }, _, None) => false,
                (Breakable::Loop { .. } | Breakable::Switch { .. }, BreakStmt::Break, None) => true,
                (Breakable::Loop { .. }, BreakStmt::Continue, None) => true,
                (Breakable::Switch { .. }, BreakStmt::Continue, None) => false,
                (Breakable::Loop { .. } | Breakable::Switch { .. }, _, Some(_)) => false,
            })
            .copied()
    }

    pub fn is_async(&self) -> bool {
        match self.ty {
            FunctionKind::Function(a) => matches!(a, Asyncness::Yes),
            FunctionKind::Generator | FunctionKind::Arrow => false,
        }
    }

    fn enclosing_finally(&self) -> Option<Label> {
        self.finally_labels.iter().copied().rev().find_map(|lbl| lbl)
    }
}

#[derive(Debug)]
pub struct FunctionCompiler<'interner> {
    function_stack: Vec<FunctionLocalState>,
    scopes: ScopeGraph,
    decl_to_slot: LocalDeclToSlot,
    current: ScopeId,
    scope_counter: Counter<ScopeId>,
    interner: &'interner mut StringInterner,
    /// Optimization level
    #[allow(unused)]
    opt_level: OptLevel,
    source: Rc<str>,
}

impl<'interner> FunctionCompiler<'interner> {
    /// Creates a new compiler for compiling parsed JavaScript functions.
    /// Prior to this, you must call `name_res` to obtain name resolution results and scopes
    pub fn new(
        source: &str,
        opt_level: OptLevel,
        name_res: NameResolutionResults,
        scope_counter: Counter<ScopeId>,
        interner: &'interner mut StringInterner,
    ) -> Self {
        Self {
            opt_level,
            scopes: name_res.scopes,
            decl_to_slot: name_res.decl_to_slot,
            scope_counter,
            interner,
            current: ScopeId::ROOT,
            function_stack: Vec::new(),
            source: Rc::from(source),
        }
    }

    pub fn compile_ast(mut self, mut ast: Vec<Statement>, implicit_return: bool) -> Result<CompileResult, Error> {
        let compile_span = span!(Level::TRACE, "compile ast");
        let _enter = compile_span.enter();

        transformations::hoist_declarations(ScopeId::ROOT, &mut self.scope_counter, &mut self.scopes, &mut ast);
        if implicit_return {
            transformations::ast_patch_implicit_return(&mut ast);
        } else {
            // Push an implicit `return undefined;` statement at the end in case there is not already an explicit one
            transformations::ast_insert_implicit_return(&mut ast);
        }

        let consteval_span = span!(Level::TRACE, "const eval");
        consteval_span.in_scope(|| {
            let opt_level = self.opt_level;
            if opt_level.enabled() {
                debug!("begin const eval, opt level: {:?}", opt_level);
                let mut cfx = ConstFunctionEvalCtx::new(&self.scopes, self.interner, opt_level);

                for stmt in &mut ast {
                    cfx.visit_statement(stmt);
                }
                debug!("finished const eval");
            } else {
                debug!("skipping const eval");
            }
        });

        self.function_stack.push(FunctionLocalState::new(
            FunctionKind::Function(Asyncness::No),
            ScopeId::ROOT,
        ));

        self.accept_multiple(ast)?;

        let root = self.function_stack.pop().expect("No root function");
        assert_eq!(root.id, ScopeId::ROOT, "Function must be the root function");
        if let Some(span) = root.references_arguments {
            return Err(Error::ArgumentsInRoot(span));
        }
        let root_function = self.scopes[root.id].expect_function();
        let locals = root_function.locals.len();

        Ok(CompileResult {
            instructions: root.buf,
            cp: root.cp,
            locals,
            externals: root.externals,
            source: self.source,
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

    /// Adds an external to the current [`FunctionLocalState`] if it's not already present
    /// and returns its ID
    fn add_external_to_func(&mut self, func_id: ScopeId, external_id: u16, is_nested_external: bool) -> usize {
        let fun = self.function_stack.iter_mut().rev().find(|f| f.id == func_id);
        let externals = &mut fun.unwrap().externals;
        if let Some(id) = externals
            .iter()
            .position(|ext| ext.id == external_id && ext.is_nested_external == is_nested_external)
        {
            id
        } else {
            externals.push(External {
                id: external_id,
                is_nested_external,
            });
            externals.len() - 1
        }
    }

    /// Returns the scope id of the function that stores the local.
    fn find_local_in_scope(&mut self, ident: Symbol, scope: ScopeId) -> Option<(u16, Local, ScopeId)> {
        let enclosing_function = self.scopes.enclosing_function_of(scope);

        if let Some(slot) = self.scopes[scope].find_local(ident) {
            return Some((
                slot,
                self.scopes[enclosing_function].expect_function().locals[slot as usize].clone(),
                enclosing_function,
            ));
        } else {
            let parent = self.scopes[scope].parent?;
            let parent_enclosing_function = self.scopes.enclosing_function_of(parent);
            let (local_id, local, src_function) = self.find_local_in_scope(ident, parent)?;
            let local = local.clone();

            if parent_enclosing_function == enclosing_function {
                // if the parent scope is in the same function, then the parent has already dealt with the external
                // and we can just return it right away without having to add it (again)
                return Some((local_id, local, src_function));
            }

            // at this point, we know we crossed a function boundary, so store it in the externals
            // and return our external id

            let nested_external = src_function != parent_enclosing_function;

            let external_id = self.add_external_to_func(enclosing_function, local_id, nested_external);
            Some((external_id.try_into().unwrap(), local, src_function))
        }
    }
    /// Tries to dynamically find a local in the current- or surrounding scopes.
    ///
    /// If a local variable is found in a parent scope, it is marked as an extern local
    pub fn find_local(&mut self, ident: Symbol) -> Option<(u16, Local, bool)> {
        let scope = self.current;
        let enclosing_function = self.scopes.enclosing_function_of(scope);
        self.find_local_in_scope(ident, scope)
            .map(|(id, loc, target_fn_scope)| (id, loc, target_fn_scope != enclosing_function))
    }

    /// This is the same as `find_local` but should be used
    /// when type_infer must have discovered/registered the local variable in the current scope.
    fn find_local_from_binding(&mut self, binding: Binding) -> u16 {
        self.decl_to_slot.slot_from_local(binding.id)
    }

    fn add_unnameable_local(&mut self, name: Symbol) -> Result<u16, LimitExceededError> {
        self.scopes.add_unnameable_local(self.current, name, None)
    }
}

impl<'interner> Visitor<Result<(), Error>> for FunctionCompiler<'interner> {
    fn accept(&mut self, Statement { kind, span }: Statement) -> Result<(), Error> {
        match kind {
            StatementKind::Expression(e) => self.visit_expression_statement(e),
            StatementKind::Variable(v) => self.visit_variable_declaration(span, v),
            StatementKind::If(i) => self.visit_if_statement(span, i),
            StatementKind::Block(b) => self.visit_block_statement(span, b),
            StatementKind::Function(f) => self.visit_function_declaration(span, f),
            StatementKind::Loop(Loop::For(f)) => self.visit_for_loop(span, None, f),
            StatementKind::Loop(Loop::While(w)) => self.visit_while_loop(span, None, w),
            StatementKind::Loop(Loop::ForOf(f)) => self.visit_for_of_loop(span, None, f),
            StatementKind::Loop(Loop::ForIn(f)) => self.visit_for_in_loop(span, None, f),
            StatementKind::Loop(Loop::DoWhile(d)) => self.visit_do_while_loop(span, None, d),
            StatementKind::Return(r) => self.visit_return_statement(span, r),
            StatementKind::Try(t) => self.visit_try_catch(span, t),
            StatementKind::Throw(t) => self.visit_throw(span, t),
            StatementKind::Import(i) => self.visit_import_statement(span, i),
            StatementKind::Export(e) => self.visit_export_statement(span, e),
            StatementKind::Class(c) => self.visit_class_declaration(span, c),
            StatementKind::Continue(sym) => self.visit_continue(span, sym),
            StatementKind::Break(sym) => self.visit_break(span, sym),
            StatementKind::Debugger => self.visit_debugger(span),
            StatementKind::Empty => self.visit_empty_statement(),
            StatementKind::Switch(s) => self.visit_switch_statement(span, s),
            StatementKind::Labelled(label, stmt) => self.visit_labelled(span, label, stmt),
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
            ExprKind::Class(e) => self.visit_class_expr(span, e),
            ExprKind::Array(e) => self.visit_array_literal(span, e),
            ExprKind::Object(e) => self.visit_object_literal(span, e),
            ExprKind::Compiled(mut buf) => {
                self.current_function_mut().buf.append(&mut buf);
                Ok(())
            }
            ExprKind::YieldStar(e) => self.visit_yield_star(span, e),
            ExprKind::Empty => self.visit_empty_expr(),
        }
    }

    fn visit_binary_expression(
        &mut self,
        span: Span,
        BinaryExpr { left, right, operator }: BinaryExpr,
    ) -> Result<(), Error> {
        let left_type = expr_ty(self, &left);
        let right_type = expr_ty(self, &right);

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
        let mut ib = InstructionBuilder::new(self);
        let res = match expr {
            LiteralExpr::Boolean(b) => ib.build_boolean_constant(b),
            LiteralExpr::Number(n) => ib.build_number_constant(n),
            LiteralExpr::String(s) => ib.build_string_constant(s),
            LiteralExpr::Identifier(_) => unreachable!("identifiers are handled in visit_identifier_expression"),
            LiteralExpr::Regex(regex, flags, sym) => ib.build_regex_constant(regex, flags, sym),
            LiteralExpr::Null => ib.build_null_constant(),
            LiteralExpr::Undefined => ib.build_undefined_constant(),
        };
        res.map_err(|_| Error::ConstantPoolLimitExceeded(span))
    }

    fn visit_identifier_expression(&mut self, span: Span, ident: Symbol) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        match ident {
            sym::this => ib.build_this(),
            // super() handled specifically in call visitor
            sym::super_ => unimplementedc!(span, "super keyword outside of a call"),
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

        // Typeof operator evaluates its operand differently if it's an identifier and there is no such local variable, so special case it here
        // `typeof x` does not throw an error if the variable does not exist,
        // `typeof x.a` does throw an error
        if let TokenType::Typeof = operator {
            if let ExprKind::Literal(LiteralExpr::Identifier(ident)) = expr.kind {
                if ib.find_local(ident).is_none() {
                    return ib.build_typeof_global_ident(span, ident);
                }
            }
        }

        // Delete operator works different from other unary operators
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
                            .add_symbol(ident)
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
                        .add_symbol(ident)
                        .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                    ib.build_static_delete(id);
                }
                _ => {
                    ib.build_boolean_constant(true)
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
                if !ib.current_function().is_async() {
                    return Err(Error::AwaitOutsideAsync { await_expr: span });
                }

                ib.build_await();
            }
            _ => unimplementedc!(span, "unary operator {:?}", operator),
        }

        Ok(())
    }

    fn visit_yield_star(&mut self, span: Span, right: Box<Expr>) -> Result<(), Error> {
        if !matches!(self.current_function().ty, FunctionKind::Generator) {
            return Err(Error::YieldOutsideGenerator { yield_expr: span });
        }

        // Desugar `yield* right` to:
        //
        // let _iterator = right;
        // let _item;
        // while (!(_item = _iterator.next()).done) {
        //     yield _item.value;
        // }
        // _item.value;

        let mut ib = InstructionBuilder::new(self);
        let mut fcx = ForEachDesugarCtxt::new(&mut ib, span)?;
        fcx.init_iterator(ForEachLoopKind::ForOf, *right)?;
        fcx.compile_loop(
            None,
            Box::new(Statement {
                span,
                kind: StatementKind::Expression(Expr {
                    span,
                    kind: ExprKind::unary(TokenType::Yield, fcx.step_value_expr()),
                }),
            }),
        )?;
        let step_value = fcx.step_value_expr();
        ib.accept_expr(step_value)?;

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
                    let slot = ib.find_local_from_binding(ident);

                    if let Some(expr) = value {
                        ib.accept_expr(expr)?;
                        ib.build_local_store(AssignKind::Assignment, slot, false);
                        ib.build_pop();
                    }
                }
                VariableDeclarationName::Pattern(ref pat) => {
                    let value = value.ok_or(Error::MissingInitializerInDestructuring(span))?;
                    compile_destructuring_pattern(&mut ib, value, pat, span)?;
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

    fn visit_block_statement(&mut self, _span: Span, BlockStatement(stmt, id): BlockStatement) -> Result<(), Error> {
        let old = self.current;
        self.current = id;

        // Note: No `?` here because we need to always exit the scope
        let re = self.accept_multiple(stmt);

        self.current = old;

        re
    }

    fn visit_function_declaration(&mut self, span: Span, fun: FunctionDeclaration) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);
        let var_id = fun.name.map(|name| ib.find_local_from_binding(name));

        ib.visit_function_expr(span, fun)?;
        if let Some(var_id) = var_id {
            ib.build_local_store(AssignKind::Assignment, var_id, false);
        }
        ib.build_pop();
        Ok(())
    }

    fn visit_while_loop(
        &mut self,
        _span: Span,
        label: Option<Symbol>,
        WhileLoop { condition, body }: WhileLoop,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        let loop_id = ib.current_function_mut().prepare_loop(label);

        ib.current_function_mut()
            .add_global_label(Label::LoopCondition { loop_id });
        ib.current_function_mut()
            .add_global_label(Label::LoopIterationEnd { loop_id });
        ib.accept_expr(condition)?;
        ib.build_jmpfalsep(Label::LoopEnd { loop_id }, false);

        ib.accept(*body)?;
        ib.build_jmp(Label::LoopCondition { loop_id }, false);

        ib.current_function_mut().add_global_label(Label::LoopEnd { loop_id });

        exit_breakable!(ib.current_function_mut(), Breakable::Loop { .. });

        Ok(())
    }

    fn visit_do_while_loop(
        &mut self,
        _span: Span,
        label: Option<Symbol>,
        DoWhileLoop { body, condition }: DoWhileLoop,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        let loop_id = ib.current_function_mut().prepare_loop(label);

        ib.current_function_mut()
            .add_global_label(Label::LoopCondition { loop_id });

        ib.accept(*body)?;

        ib.current_function_mut()
            .add_global_label(Label::LoopIterationEnd { loop_id });
        ib.accept_expr(condition)?;
        ib.build_jmptruep(Label::LoopCondition { loop_id }, false);

        ib.current_function_mut().add_global_label(Label::LoopEnd { loop_id });
        exit_breakable!(ib.current_function_mut(), Breakable::Loop { .. });

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
                        if let VariableDeclarationKind::Const = local.kind {
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
                _ => {
                    match &left.kind {
                        ExprKind::Object(ObjectLiteral(object)) => {
                            let mut rest = None;

                            for (kind, expr) in object {
                                if let ObjectMemberKind::Spread = kind {
                                    let Some(ident) = expr.kind.as_identifier() else {
                                        unimplementedc!(span, "rest binding must be an identifier")
                                    };

                                    let Some((local, _, false)) = ib.find_local(ident) else {
                                        unimplementedc!(span, "rest binding must be defined in the current function")
                                    };

                                    if rest.is_some() {
                                        unimplementedc!(span, "duplicate rest binding in object destructuring");
                                    }

                                    rest = Some(local);
                                }
                            }

                            let object_local = ib.add_unnameable_local(sym::empty).map_err(|_| Error::LocalLimitExceeded(span))?;
                            ib.accept_expr(*right)?;
                            ib.build_local_store(AssignKind::Assignment, object_local, false);

                            ib.build_objdestruct((object.len() - rest.is_some() as usize).try_into().map_err(|_| Error::DestructureLimitExceeded(span))?, rest);

                            for (kind, expr) in object {
                                let name = match kind {
                                    ObjectMemberKind::Spread => continue,
                                    ObjectMemberKind::Static(sym) => *sym,
                                    other => unimplementedc!(span, "invalid object member in destructuring: {:?}", other)
                                };
                                let Some(alias) = expr.kind.as_identifier() else {
                                    unimplementedc!(span, "binding must be an identifier")
                                };

                                let Some((local, _, false)) = ib.find_local(alias) else {
                                    unimplementedc!(span, "binding must be defined in the current function")
                                };

                                let NumberConstant(var_id) = ib
                                    .current_function_mut()
                                    .cp
                                    .add_number(local as f64)
                                    .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                                let SymbolConstant(ident_id) = ib
                                    .current_function_mut()
                                    .cp
                                    .add_symbol(name)
                                    .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                                ib.writew(var_id);
                                ib.writew(ident_id);
                            }

                            ib.build_local_load(object_local, false);
                        },
                        ExprKind::Array(ArrayLiteral(array)) => {
                            let array_local = ib.add_unnameable_local(sym::empty).map_err(|_| Error::LocalLimitExceeded(span))?;
                            ib.accept_expr(*right)?;
                            ib.build_local_store(AssignKind::Assignment, array_local, false);

                            ib.build_arraydestruct(array.len().try_into().map_err(|_| Error::LocalLimitExceeded(span))?);

                            for kind in array {
                                match kind {
                                    ArrayMemberKind::Empty => ib.write_bool(false),
                                    ArrayMemberKind::Item(expr) => {
                                        ib.write_bool(true);

                                        let Some(alias) = expr.kind.as_identifier() else {
                                            unimplementedc!(span, "binding must be an identifier")
                                        };

                                        let Some((local, _, false)) = ib.find_local(alias) else {
                                            unimplementedc!(span, "binding must be defined in the current function")
                                        };

                                        let NumberConstant(var_id) = ib
                                            .current_function_mut()
                                            .cp
                                            .add_number(local as f64)
                                            .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;
                                        ib.writew(var_id);
                                    }
                                    ArrayMemberKind::Spread(_) => unimplementedc!(span, "rest operator in array destructuring is unsupported"),
                                }
                            }

                            ib.build_local_load(array_local, false);
                        }
                        _ => unimplementedc!(span, "assignment to non-identifier")
                    }
                },
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

        if let ExprKind::Literal(LiteralExpr::Identifier(sym::super_)) = target.kind {
            // super() call lowering
            let super_id = ib
                .inner
                .scopes
                .add_unnameable_local(ib.current, sym::super_, None)
                .map_err(|_| Error::LocalLimitExceeded(span))?;

            // __super = new this.constructor.__proto__()
            let superclass_constructor_call = Expr {
                span: Span::COMPILER_GENERATED,
                kind: ExprKind::function_call(
                    Expr {
                        span: Span::COMPILER_GENERATED,
                        kind: ExprKind::property_access(
                            false,
                            Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::property_access(
                                    false,
                                    Expr {
                                        span: Span::COMPILER_GENERATED,
                                        kind: ExprKind::identifier(sym::this),
                                    },
                                    Expr {
                                        span: Span::COMPILER_GENERATED,
                                        kind: ExprKind::identifier(sym::constructor),
                                    },
                                ),
                            },
                            Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::identifier(sym::__proto__),
                            },
                        ),
                    },
                    arguments,
                    true,
                ),
            };

            ib.visit_expression_statement(Expr {
                span: Span::COMPILER_GENERATED,
                kind: ExprKind::assignment_local_space(super_id, superclass_constructor_call, TokenType::Assignment),
            })?;

            // __super.__proto__ = this.__proto__
            ib.visit_expression_statement(Expr {
                span: Span::COMPILER_GENERATED,
                kind: ExprKind::assignment(
                    Expr {
                        span: Span::COMPILER_GENERATED,
                        kind: ExprKind::property_access(
                            false,
                            Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::compiled(compile_local_load(super_id, false)),
                            },
                            Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::identifier(sym::__proto__),
                            },
                        ),
                    },
                    Expr {
                        span: Span::COMPILER_GENERATED,
                        kind: ExprKind::property_access(
                            false,
                            Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::Literal(LiteralExpr::Identifier(sym::this)),
                            },
                            Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::identifier(sym::__proto__),
                            },
                        ),
                    },
                    TokenType::Assignment,
                ),
            })?;

            // this.__proto__ = __super
            ib.visit_assignment_expression(
                Span::COMPILER_GENERATED,
                AssignmentExpr::new_expr_place(
                    Expr {
                        span: Span::COMPILER_GENERATED,
                        kind: ExprKind::property_access(
                            false,
                            Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::Literal(LiteralExpr::Identifier(sym::this)),
                            },
                            Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::identifier(sym::__proto__),
                            },
                        ),
                    },
                    Expr {
                        span: Span::COMPILER_GENERATED,
                        kind: ExprKind::compiled(compile_local_load(super_id, false)),
                    },
                    TokenType::Assignment,
                ),
            )?;
            // Assignment expression leaves `super` on the stack, as it is needed by expressions

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
        let mut ib = InstructionBuilder::new(self);
        let finally = ib.current_function().enclosing_finally();

        let tc_depth = ib.current_function().try_depth;
        ib.accept_expr(stmt)?;
        if let Some(finally) = finally {
            ib.write_instr(Instruction::DelayedReturn);
            ib.build_jmp(finally, false);
        } else {
            ib.build_ret(tc_depth);
        }

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
            ty_segment: _,
            constructor_initializers,
        }: FunctionDeclaration,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);
        ib.with_scope(id, |ib| {
            ib.function_stack.push(FunctionLocalState::new(ty, id));

            let mut rest_local = None;

            for (param, default, _ty) in &arguments {
                let id = match *param {
                    Parameter::Identifier(binding) | Parameter::SpreadIdentifier(binding) => {
                        ib.find_local_from_binding(binding)
                    }
                    Parameter::Pattern(id, _) | Parameter::SpreadPattern(id, _) => ib.decl_to_slot.slot_from_local(id),
                };

                if let Parameter::SpreadIdentifier(_) | Parameter::SpreadPattern(..) = param {
                    rest_local = Some(id);
                }

                if let Some(default) = default {
                    let mut sub_ib = InstructionBuilder::new(ib);
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

                if let Parameter::Pattern(_, pat) | Parameter::SpreadPattern(_, pat) = param {
                    compile_destructuring_pattern(
                        ib,
                        Expr {
                            span,
                            kind: ExprKind::Compiled(compile_local_load(id, false)),
                        },
                        pat,
                        span,
                    )?;
                }
            }

            transformations::hoist_declarations(id, &mut ib.inner.scope_counter, &mut ib.inner.scopes, &mut statements);
            transformations::ast_insert_implicit_return(&mut statements);

            // Insert initializers
            if let Some(members) = constructor_initializers {
                let members = compile_class_members(ib, span, members)?;
                ib.build_this();
                ib.build_object_member_like_instruction(span, members, Instruction::AssignProperties)?;
            }

            let res = statements.into_iter().try_for_each(|stmt| ib.accept(stmt));

            let cmp = ib.function_stack.pop().expect("Missing function state");
            res?; // Cannot early return error in the loop as we need to pop the function state in any case
            let locals = ib.scopes[id].expect_function().locals.len();

            let function = Function {
                buffer: Buffer(Cell::new(cmp.buf.into())),
                constants: cmp.cp,
                locals,
                name: name.map(|binding| binding.ident),
                ty,
                params: match arguments.last() {
                    Some((Parameter::SpreadPattern(..) | Parameter::SpreadIdentifier(_), ..)) => arguments.len() - 1,
                    _ => arguments.len(),
                },
                externals: cmp.externals.into(),
                rest_local,
                poison_ips: RefCell::new(HashSet::new()),
                debug_symbols: cmp.debug_symbols,
                source: Rc::clone(&ib.source),
                references_arguments: cmp.references_arguments.is_some(),
            };
            ib.build_function_constant(function)
                .map_err(|_| Error::ConstantPoolLimitExceeded(span))?;

            Ok(())
        })
    }

    fn visit_array_literal(&mut self, _: Span, ArrayLiteral(exprs): ArrayLiteral) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        let mut component_count = 0;
        let mut stack_values = 0;
        let mut kinds = Vec::with_capacity(exprs.len());
        let mut exprs = exprs.into_iter().peekable();

        while let Some(kind) = exprs.next() {
            kinds.push(dash_middle::compiler::ArrayMemberKind::from(&kind) as u8);

            match kind {
                ArrayMemberKind::Item(expr) => {
                    ib.accept_expr(expr)?;
                    stack_values += 1;
                }
                ArrayMemberKind::Spread(expr) => {
                    ib.accept_expr(expr)?;
                    stack_values += 1;
                }
                ArrayMemberKind::Empty => {
                    // merge consecutive holes
                    let mut holes = 1;
                    while exprs.next_if(|v| matches!(v, ArrayMemberKind::Empty)).is_some() {
                        holes += 1;
                    }

                    kinds.push(holes);
                }
            }
            component_count += 1;
        }

        ib.build_arraylit(component_count, stack_values);
        ib.write_all(&kinds);
        Ok(())
    }

    fn visit_object_literal(&mut self, span: Span, ObjectLiteral(exprs): ObjectLiteral) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        let members = compile_object_members(&mut ib, exprs.iter().cloned())?;
        ib.build_object_member_like_instruction(span, members, Instruction::ObjLit)?;

        Ok(())
    }

    fn visit_try_catch(&mut self, span: Span, TryCatch { try_, catch, finally }: TryCatch) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        let finally = finally.map(|f| (ib.current_function_mut().finally_counter.inc(), f));

        ib.build_try_block(catch.is_some(), finally.as_ref().map(|&(id, _)| id));

        ib.current_function_mut().try_depth += 1;
        ib.current_function_mut()
            .finally_labels
            .push(finally.as_ref().map(|&(finally_id, _)| Label::Finally { finally_id }));
        let res = ib.accept(*try_);
        ib.current_function_mut().try_depth -= 1;
        res?;

        ib.build_jmp(Label::TryEnd, true);

        if catch.is_none() && finally.is_none() {
            // FIXME: make it a real error
            unimplementedc!(span, "try block has no catch or finally");
        }

        if let Some(catch) = catch {
            ib.add_local_label(Label::Catch);

            if let Some(binding) = catch.binding {
                let id = ib.find_local_from_binding(binding);

                if id == u16::MAX {
                    // Max u16 value is reserved for "no binding"
                    return Err(Error::LocalLimitExceeded(span));
                }

                ib.writew(id);
            } else {
                ib.writew(u16::MAX);
            }

            ib.visit_block_statement(catch.body_span, catch.body)?;
        }
        ib.current_function_mut().finally_labels.pop();

        if let Some((finally_id, finally)) = finally {
            ib.current_function_mut()
                .add_global_label(Label::Finally { finally_id });
            ib.add_local_label(Label::TryEnd);
            ib.build_try_end();

            ib.accept(*finally)?;

            ib.write_instr(Instruction::FinallyEnd);
            ib.writew(ib.current_function().try_depth);
        } else {
            ib.add_local_label(Label::TryEnd);
            ib.build_try_end();
        }

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
        label: Option<Symbol>,
        ForLoop {
            init,
            condition,
            finalizer,
            body,
            scope,
        }: ForLoop,
    ) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);
        ib.with_scope(scope, |ib| {
            // Initialization
            if let Some(init) = init {
                ib.accept(*init)?;
            }

            let loop_id = ib.current_function_mut().prepare_loop(label);

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
                .add_global_label(Label::LoopIterationEnd { loop_id });
            if let Some(finalizer) = finalizer {
                ib.accept_expr(finalizer)?;
                ib.build_pop();
            }
            ib.build_jmp(Label::LoopCondition { loop_id }, false);

            ib.current_function_mut().add_global_label(Label::LoopEnd { loop_id });
            exit_breakable!(ib.current_function_mut(), Breakable::Loop { .. });
            Ok(())
        })
    }

    fn visit_for_of_loop(
        &mut self,
        span: Span,
        label: Option<Symbol>,
        ForOfLoop {
            binding,
            expr,
            body,
            scope,
        }: ForOfLoop,
    ) -> Result<(), Error> {
        ForEachDesugarCtxt::new(&mut InstructionBuilder::new(self), span)?.desugar_for_each_kinded_loop(
            ForEachLoopKind::ForOf,
            binding,
            expr,
            body,
            scope,
            label,
        )
    }

    fn visit_for_in_loop(
        &mut self,
        span: Span,
        label: Option<Symbol>,
        ForInLoop {
            binding,
            expr,
            body,
            scope,
        }: ForInLoop,
    ) -> Result<(), Error> {
        ForEachDesugarCtxt::new(&mut InstructionBuilder::new(self), span)?.desugar_for_each_kinded_loop(
            ForEachLoopKind::ForIn,
            binding,
            expr,
            body,
            scope,
            label,
        )
    }

    fn visit_import_statement(&mut self, span: Span, import: ImportKind) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        match import {
            ImportKind::Dynamic(ex) => {
                ib.accept_expr(ex)?;
                ib.build_dynamic_import();
            }
            ref kind @ (ImportKind::DefaultAs(SpecifierKind::Ident(sym), path)
            | ImportKind::AllAs(SpecifierKind::Ident(sym), path)) => {
                let local_id = ib.find_local_from_binding(sym);

                let path_id = ib
                    .current_function_mut()
                    .cp
                    .add_symbol(path)
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
                        .add_symbol(name)
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
                        VariableDeclarationName::Identifier(Binding { ident, .. }) => it.push(ident),
                        VariableDeclarationName::Pattern(Pattern::Array { ref fields, rest }) => {
                            it.extend(fields.iter().flatten().map(|(b, _)| b.ident));
                            it.extend(rest.map(|b| b.ident));
                        }
                        VariableDeclarationName::Pattern(Pattern::Object { ref fields, rest }) => {
                            it.extend(fields.iter().map(|&(_, name, ident, _)| ident.unwrap_or(name)));
                            it.extend(rest.map(|b| b.ident));
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

    fn visit_break(&mut self, span: Span, sym: Option<Symbol>) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        if ib.current_function().enclosing_finally().is_some() {
            unimplementedc!(span, "`break` in a try-finally block");
        }

        let breakable = ib
            .current_function_mut()
            .find_breakable(sym, BreakStmt::Break)
            .ok_or(Error::IllegalBreak(span))?;

        match breakable {
            Breakable::Loop { loop_id, label: _ } => {
                ib.build_jmp(Label::LoopEnd { loop_id }, false);
            }
            Breakable::Switch { switch_id } => {
                ib.build_jmp(Label::SwitchEnd { switch_id }, false);
            }
            Breakable::Named { sym: _, label_id } => {
                ib.build_jmp(Label::UserDefinedEnd { id: label_id }, false);
            }
        }
        Ok(())
    }

    fn visit_continue(&mut self, span: Span, sym: Option<Symbol>) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        if ib.current_function().enclosing_finally().is_some() {
            unimplementedc!(span, "`continue` in a try-finally block");
        }

        let breakable = ib
            .current_function_mut()
            .find_breakable(sym, BreakStmt::Continue)
            .ok_or(Error::IllegalBreak(span))?;

        match breakable {
            Breakable::Loop { loop_id, label: _ } => {
                ib.build_jmp(Label::LoopIterationEnd { loop_id }, false);
            }
            Breakable::Switch { .. } => {
                // TODO: make it possible to use `continue` in loops even if its used in a switch
                unimplementedc!(span, "`continue` used inside of a switch statement");
            }
            Breakable::Named { .. } => {
                unimplementedc!(span, "`continue` cannot target a non-iteration statement");
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

    fn visit_class_expr(&mut self, span: Span, class: Class) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);

        let load_super_class = match class.extends.as_deref() {
            Some(expr) => {
                let extend_id = ib
                    .add_unnameable_local(sym::DesugaredClass)
                    .map_err(|_| Error::LocalLimitExceeded(span))?;

                // __super = <class extend expression>
                ib.visit_expression_statement(Expr {
                    span: Span::COMPILER_GENERATED,
                    kind: ExprKind::Assignment(AssignmentExpr {
                        left: AssignmentTarget::LocalId(extend_id),
                        right: Box::new(expr.clone()),
                        operator: TokenType::Assignment,
                    }),
                })?;

                Some(Expr {
                    span: Span::COMPILER_GENERATED,
                    kind: ExprKind::Compiled(compile_local_load(extend_id, false)),
                })
            }
            None => None,
        };
        let constructor = class.constructor();

        let binding_id = match class.name {
            Some(name) => ib.find_local_from_binding(name),
            None => ib
                .add_unnameable_local(sym::DesugaredClass)
                .map_err(|_| Error::LocalLimitExceeded(span))?,
        };

        let (parameters, statements, id) = match constructor {
            Some(fun) => (fun.parameters, fun.statements, fun.id),
            None => {
                let parent = ib.current;
                let scope = ib
                    .inner
                    .scopes
                    .add_empty_function_scope(parent, &mut ib.inner.scope_counter);

                (Vec::new(), Vec::new(), scope)
            }
        };

        let fields = class
            .members
            .iter()
            .filter(|member| matches!(member.value, ClassMemberValue::Field(_)));

        let desugared_class = FunctionDeclaration {
            id,
            name: class.name,
            parameters,
            statements,
            ty: FunctionKind::Function(Asyncness::No),
            ty_segment: None,
            constructor_initializers: Some(fields.clone().filter(|member| !member.static_).cloned().collect()),
        };

        ib.visit_expression_statement(Expr {
            span: Span::COMPILER_GENERATED,
            kind: ExprKind::Assignment(AssignmentExpr::new_local_place(
                binding_id,
                Expr {
                    span,
                    kind: ExprKind::Function(desugared_class),
                },
                TokenType::Assignment,
            )),
        })?;
        let load_class_binding = Expr {
            span: Span::COMPILER_GENERATED,
            kind: ExprKind::Compiled(compile_local_load(binding_id, false)),
        };

        // Class.prototype
        let class_prototype = Expr {
            span: Span::COMPILER_GENERATED,
            kind: ExprKind::property_access(
                false,
                load_class_binding.clone(),
                Expr {
                    span: Span::COMPILER_GENERATED,
                    kind: ExprKind::identifier(sym::prototype),
                },
            ),
        };

        let methods = class.members.iter().filter(|member| {
            matches!(
                member.value,
                ClassMemberValue::Getter(_) | ClassMemberValue::Setter(_) | ClassMemberValue::Method(_)
            )
        });

        let static_m = compile_class_members(&mut ib, span, methods.clone().filter(|method| method.static_).cloned())?;
        ib.accept_expr(load_class_binding.clone())?;
        ib.build_object_member_like_instruction(span, static_m, Instruction::AssignProperties)?;

        let prototype_m = compile_class_members(&mut ib, span, methods.filter(|method| !method.static_).cloned())?;
        ib.accept_expr(class_prototype.clone())?;
        ib.build_object_member_like_instruction(span, prototype_m, Instruction::AssignProperties)?;

        let static_fields = compile_class_members(&mut ib, span, fields.filter(|member| member.static_).cloned())?;
        ib.accept_expr(load_class_binding.clone())?;
        ib.build_object_member_like_instruction(span, static_fields, Instruction::AssignProperties)?;

        if let Some(super_id) = load_super_class {
            // Add the superclass' prototype to our prototype chain
            // Class.prototype.__proto__ = Superclass.prototype

            ib.visit_expression_statement(Expr {
                span: Span::COMPILER_GENERATED,
                kind: ExprKind::assignment(
                    Expr {
                        span: Span::COMPILER_GENERATED,
                        kind: ExprKind::property_access(
                            false,
                            class_prototype.clone(),
                            Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::identifier(sym::__proto__),
                            },
                        ),
                    },
                    Expr {
                        span: Span::COMPILER_GENERATED,
                        kind: ExprKind::property_access(
                            false,
                            super_id.clone(),
                            Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::identifier(sym::prototype),
                            },
                        ),
                    },
                    TokenType::Assignment,
                ),
            })?;

            // Set the [[Prototype]] of this class to its superclass
            // Class.__proto__ = Superclass
            ib.visit_expression_statement(Expr {
                span: Span::COMPILER_GENERATED,
                kind: ExprKind::assignment(
                    Expr {
                        span: Span::COMPILER_GENERATED,
                        kind: ExprKind::property_access(
                            false,
                            load_class_binding.clone(),
                            Expr {
                                span: Span::COMPILER_GENERATED,
                                kind: ExprKind::identifier(sym::__proto__),
                            },
                        ),
                    },
                    super_id,
                    TokenType::Assignment,
                ),
            })?;
        }

        // Load it one last time since the `class` expression ultimately should evaluate to that class
        ib.accept_expr(load_class_binding)?;

        Ok(())
    }

    fn visit_class_declaration(&mut self, span: Span, class: Class) -> Result<(), Error> {
        self.visit_class_expr(span, class)?;
        InstructionBuilder::new(self).build_pop();
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
        exit_breakable!(ib.current_function_mut(), Breakable::Switch { .. });

        Ok(())
    }

    fn visit_labelled(&mut self, _: Span, label: Symbol, stmt: Box<Statement>) -> Result<(), Error> {
        let mut ib = InstructionBuilder::new(self);
        let label_id = ib.current_function_mut().user_label_counter.inc();
        ib.current_function_mut()
            .breakables
            .push(Breakable::Named { sym: label, label_id });

        if let StatementKind::Loop(lp) = stmt.kind {
            match lp {
                Loop::For(for_loop) => ib.visit_for_loop(stmt.span, Some(label), for_loop)?,
                Loop::ForOf(for_of_loop) => ib.visit_for_of_loop(stmt.span, Some(label), for_of_loop)?,
                Loop::ForIn(for_in_loop) => ib.visit_for_in_loop(stmt.span, Some(label), for_in_loop)?,
                Loop::While(while_loop) => ib.visit_while_loop(stmt.span, Some(label), while_loop)?,
                Loop::DoWhile(do_while_loop) => ib.visit_do_while_loop(stmt.span, Some(label), do_while_loop)?,
            }
        } else {
            ib.accept(*stmt)?;
        }

        ib.current_function_mut()
            .add_global_label(Label::UserDefinedEnd { id: label_id });
        exit_breakable!(ib.current_function_mut(), Breakable::Named { .. });
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
///     # if there's a break anywhere in here, jump to end of switch
///     jmp case_x
///
/// case_x_cond:
///     eq (ld_1 ld x)
///     jmpfalsep case_y_cond
///  case_x:
///     # code for case x
///     constant 'case 1'
///     ret
///     jmp case_y
///
/// case_x_code:
///     ...
/// case_y_cond:
///     eq (ld_1 ld y)
///     jmpfalsep default
/// case y:
///     ...
///     jmp default
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
        .add_unnameable_local(sym::switch_cond_desugar)
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

fn compile_object_members(
    ib: &mut InstructionBuilder<'_, '_>,
    iter: impl IntoIterator<Item = (ObjectMemberKind, Expr)>,
) -> Result<Vec<ObjectMemberKind>, Error> {
    let iter = iter.into_iter();

    let mut members = Vec::with_capacity(iter.size_hint().0);
    for (member, value) in iter {
        ib.accept_expr(value)?;

        let mut push_and_accept = |ct: fn(Expr) -> ObjectMemberKind, expr: Expr| {
            // TODO: no clone really needed, the `expr` is not needed in ib.build_objlit
            members.push(ct(expr.clone()));
            ib.accept_expr(expr)
        };

        match member {
            ObjectMemberKind::DynamicGetter(expr) => push_and_accept(ObjectMemberKind::DynamicGetter, expr)?,
            ObjectMemberKind::DynamicSetter(expr) => push_and_accept(ObjectMemberKind::DynamicSetter, expr)?,
            ObjectMemberKind::Dynamic(expr) => push_and_accept(ObjectMemberKind::Dynamic, expr)?,
            _ => members.push(member),
        }
    }

    Ok(members)
}

fn compile_destructuring_pattern(
    ib: &mut InstructionBuilder<'_, '_>,
    from: Expr,
    pat: &Pattern,
    at: Span,
) -> Result<(), Error> {
    match pat {
        Pattern::Object { fields, rest } => {
            let rest_id = rest.map(|rest| ib.find_local_from_binding(rest));

            let field_count = fields
                .len()
                .try_into()
                .map_err(|_| Error::DestructureLimitExceeded(at))?;

            for (.., default) in fields.iter().rev() {
                if let Some(default) = default {
                    ib.accept_expr(default.clone())?;
                }
            }

            ib.accept_expr(from)?;

            ib.build_objdestruct(field_count, rest_id);

            for &(local, name, alias, ref default) in fields {
                let name = alias.unwrap_or(name);
                let id = ib.find_local_from_binding(Binding { id: local, ident: name });

                let NumberConstant(var_id) = ib
                    .current_function_mut()
                    .cp
                    .add_number(id as f64)
                    .map_err(|_| Error::ConstantPoolLimitExceeded(at))?;
                let SymbolConstant(ident_id) = ib
                    .current_function_mut()
                    .cp
                    .add_symbol(name)
                    .map_err(|_| Error::ConstantPoolLimitExceeded(at))?;
                ib.write_bool(default.is_some());
                ib.writew(var_id);
                ib.writew(ident_id);
            }
        }
        Pattern::Array { fields, rest } => {
            if rest.is_some() {
                unimplementedc!(at, "rest operator in array destructuring");
            }

            let field_count = fields
                .len()
                .try_into()
                .map_err(|_| Error::DestructureLimitExceeded(at))?;

            #[expect(clippy::manual_flatten, reason = "pattern contains an inner Some()")]
            for field in fields.iter().rev() {
                if let Some((_, Some(default))) = field {
                    ib.accept_expr(default.clone())?;
                }
            }

            ib.accept_expr(from)?;

            ib.build_arraydestruct(field_count);

            for name in fields {
                ib.write_bool(name.is_some());
                if let Some((name, ref default)) = *name {
                    let id = ib.find_local_from_binding(name);

                    let NumberConstant(id) = ib
                        .current_function_mut()
                        .cp
                        .add_number(id as f64)
                        .map_err(|_| Error::ConstantPoolLimitExceeded(at))?;

                    ib.write_bool(default.is_some());
                    ib.writew(id);
                }
            }
        }
    }

    Ok(())
}

fn compile_class_members(
    ib: &mut InstructionBuilder<'_, '_>,
    span: Span,
    it: impl IntoIterator<Item = ClassMember>,
) -> Result<Vec<ObjectMemberKind>, Error> {
    let mk_fn = |f| Expr {
        span,
        kind: ExprKind::function(f),
    };

    compile_object_members(
        ib,
        it.into_iter().map(|member| {
            let (key, value) = match (member.key, member.value) {
                (ClassMemberKey::Computed(key), ClassMemberValue::Method(value)) => {
                    (ObjectMemberKind::Dynamic(key), mk_fn(value))
                }
                (ClassMemberKey::Computed(key), ClassMemberValue::Field(value)) => (
                    ObjectMemberKind::Dynamic(key),
                    value.unwrap_or_else(|| Expr {
                        span,
                        kind: ExprKind::undefined_literal(),
                    }),
                ),
                (ClassMemberKey::Computed(key), ClassMemberValue::Getter(value)) => {
                    (ObjectMemberKind::DynamicGetter(key), mk_fn(value))
                }
                (ClassMemberKey::Computed(key), ClassMemberValue::Setter(value)) => {
                    (ObjectMemberKind::DynamicSetter(key), mk_fn(value))
                }
                (ClassMemberKey::Named(key), ClassMemberValue::Method(value)) => {
                    (ObjectMemberKind::Static(key), mk_fn(value))
                }
                (ClassMemberKey::Named(key), ClassMemberValue::Field(value)) => (
                    ObjectMemberKind::Static(key),
                    value.unwrap_or_else(|| Expr {
                        span,
                        kind: ExprKind::undefined_literal(),
                    }),
                ),
                (ClassMemberKey::Named(key), ClassMemberValue::Getter(value)) => {
                    (ObjectMemberKind::Getter(key), mk_fn(value))
                }
                (ClassMemberKey::Named(key), ClassMemberValue::Setter(value)) => {
                    (ObjectMemberKind::Setter(key), mk_fn(value))
                }
            };

            (key, value)
        }),
    )
}

fn expr_ty(c: &FunctionCompiler<'_>, expr: &Expr) -> Option<CompileValueType> {
    let func_id = c.current_function().id;
    let mut tcx = TypeInferCtx::view(InferMode::View(&c.scopes), c.current, func_id);
    tcx.visit(expr)
}
