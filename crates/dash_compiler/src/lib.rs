use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;
use std::{convert::TryInto, usize};

use dash_log::{debug, span, Level};
use dash_middle::compiler::constant::{Buffer, Constant, Function};
use dash_middle::compiler::instruction::{AssignKind, IntrinsicOperation};
use dash_middle::compiler::scope::ScopeLocal;
use dash_middle::compiler::scope::{CompileValueType, Scope};
use dash_middle::compiler::{constant::ConstantPool, external::External};
use dash_middle::compiler::{CompileResult, FunctionCallMetadata, StaticImportKind};
use dash_middle::lexer::token::TokenType;
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
use dash_middle::parser::expr::{ArrayLiteral, ObjectMemberKind};
use dash_middle::parser::expr::{ArrayMemberKind, BinaryExpr};
use dash_middle::parser::expr::{AssignmentExpr, AssignmentTarget};
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::SpecifierKind;
use dash_middle::parser::statement::Statement;
use dash_middle::parser::statement::TryCatch;
use dash_middle::parser::statement::VariableBinding;
use dash_middle::parser::statement::VariableDeclaration;
use dash_middle::parser::statement::VariableDeclarationKind;
use dash_middle::parser::statement::WhileLoop;
use dash_middle::parser::statement::{BlockStatement, Loop};
use dash_middle::parser::statement::{Class, Parameter};
use dash_middle::parser::statement::{ClassMemberKind, ExportKind};
use dash_middle::parser::statement::{DoWhileLoop, ForLoop};
use dash_middle::parser::statement::{ForInLoop, ForOfLoop};
use dash_middle::parser::statement::{FuncId, ImportKind};
use dash_middle::parser::statement::{FunctionDeclaration, SwitchStatement};
use dash_middle::parser::statement::{FunctionKind, VariableDeclarationName};
use dash_middle::parser::statement::{IfStatement, VariableDeclarations};
use dash_middle::visitor::Visitor;
use dash_optimizer::consteval::ConstFunctionEvalCtx;
use dash_optimizer::type_infer::TypeInferCtx;
use dash_optimizer::OptLevel;
use instruction::compile_local_load;
use jump_container::JumpContainer;

use crate::builder::{InstructionBuilder, Label};

use self::{error::CompileError, instruction::NamedExportKind};

pub mod builder;
pub mod error;
#[cfg(feature = "from_string")]
pub mod from_string;
pub mod instruction;
pub mod transformations;
// #[cfg(test)]
// mod test;
mod jump_container;

macro_rules! unimplementedc {
    ($($what:expr),*) => {
        return Err(CompileError::Unimplemented(format_args!($($what),*).to_string()))
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
pub struct FunctionCompiler<'a> {
    function_stack: Vec<FunctionLocalState>,
    tcx: TypeInferCtx<'a>,
    /// Optimization level
    #[allow(unused)]
    opt_level: OptLevel,
}

impl<'a> FunctionCompiler<'a> {
    pub fn new(opt_level: OptLevel, tcx: TypeInferCtx<'a>) -> Self {
        Self {
            opt_level,
            tcx,
            function_stack: Vec::new(),
        }
    }

    pub fn compile_ast(
        mut self,
        mut ast: Vec<Statement<'a>>,
        implicit_return: bool,
    ) -> Result<CompileResult, CompileError> {
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
                let mut cfx = ConstFunctionEvalCtx::new(&mut self.tcx, opt_level);

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
        let root_scope = self.tcx.scope(root.id);
        let locals = root_scope.locals().len();
        let externals = root_scope.externals().to_owned();

        Ok(CompileResult {
            instructions: root.buf,
            cp: root.cp,
            locals,
            externals,
        })
    }

    pub fn accept_multiple(&mut self, stmts: Vec<Statement<'a>>) -> Result<(), CompileError> {
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

    fn current_scope(&self) -> &Scope<'a> {
        let id = self.current_function().id;
        self.tcx.scope(id)
    }

    fn current_scope_mut(&mut self) -> &mut Scope<'a> {
        let id = self.current_function().id;
        self.tcx.scope_mut(id)
    }

    /// Adds an external to the current [`FunctionLocalState`] if it's not already present
    /// and returns its ID
    fn add_external_to_func(&mut self, func_id: FuncId, external_id: u16, is_nested_external: bool) -> usize {
        let externals = self.tcx.scope_mut(func_id).externals_mut();
        let id = externals
            .iter()
            .position(|External { id, is_external }| *id == external_id && *is_external == is_nested_external);

        match id {
            Some(id) => id,
            None => {
                externals.push(External {
                    id: external_id,
                    is_external: is_nested_external,
                });
                externals.len() - 1
            }
        }
    }

    fn find_local_in_scope(&mut self, ident: &str, func_id: FuncId) -> Option<(u16, ScopeLocal<'a>, bool)> {
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
    pub fn find_local(&mut self, ident: &str) -> Option<(u16, ScopeLocal<'a>, bool)> {
        let func_id = self.current_function().id;
        self.find_local_in_scope(ident, func_id)
    }

    fn visit_for_each_kinded_loop(
        &mut self,
        kind: ForEachLoopKind,
        binding: VariableBinding<'a>,
        expr: Expr<'a>,
        mut body: Box<Statement<'a>>,
    ) -> Result<(), CompileError> {
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
        let for_of_iter_id =
            ib.current_scope_mut()
                .add_local("for_of_iter", VariableDeclarationKind::Unnameable, None)?;

        let for_of_gen_step_id =
            ib.current_scope_mut()
                .add_local("for_of_gen_step", VariableDeclarationKind::Unnameable, None)?;

        ib.accept_expr(expr)?;
        match kind {
            ForEachLoopKind::ForOf => ib.build_symbol_iterator(),
            ForEachLoopKind::ForIn => ib.build_for_in_iterator(),
        }
        ib.build_local_store(AssignKind::Assignment, for_of_iter_id, false);
        ib.build_pop();

        // Prepend variable assignment to body
        if !matches!(&*body, Statement::Block(..)) {
            let old_body = std::mem::replace(&mut *body, Statement::Empty);

            *body = Statement::Block(BlockStatement(vec![old_body]));
        }

        // Assign iterator value to binding at the very start of the for loop body
        match &mut *body {
            Statement::Block(BlockStatement(stmts)) => {
                let gen_step = compile_local_load(for_of_gen_step_id, false);

                let var = Statement::Variable(VariableDeclarations(vec![VariableDeclaration::new(
                    binding,
                    Some(Expr::property_access(
                        false,
                        Expr::compiled(gen_step),
                        Expr::identifier(Cow::Borrowed("value")),
                    )),
                )]));

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
        ib.visit_while_loop(WhileLoop {
            condition: Expr::unary(
                TokenType::LogicalNot,
                Expr::property_access(
                    false,
                    Expr::assignment_local_space(
                        for_of_gen_step_id,
                        Expr::function_call(
                            Expr::property_access(
                                false,
                                Expr::compiled(for_of_iter_binding_bc),
                                Expr::identifier(Cow::Borrowed("next")),
                            ),
                            Vec::new(),
                            false,
                        ),
                        TokenType::Assignment,
                    ),
                    Expr::identifier(Cow::Borrowed("done")),
                ),
            ),
            body,
        })?;

        Ok(())
    }
}

enum ForEachLoopKind {
    ForOf,
    ForIn,
}

impl<'a> Visitor<'a, Result<(), CompileError>> for FunctionCompiler<'a> {
    fn accept(&mut self, stmt: Statement<'a>) -> Result<(), CompileError> {
        match stmt {
            Statement::Expression(e) => self.visit_expression_statement(e),
            Statement::Variable(v) => self.visit_variable_declaration(v),
            Statement::If(i) => self.visit_if_statement(i),
            Statement::Block(b) => self.visit_block_statement(b),
            Statement::Function(f) => self.visit_function_declaration(f),
            Statement::Loop(Loop::For(f)) => self.visit_for_loop(f),
            Statement::Loop(Loop::While(w)) => self.visit_while_loop(w),
            Statement::Loop(Loop::ForOf(f)) => self.visit_for_of_loop(f),
            Statement::Loop(Loop::ForIn(f)) => self.visit_for_in_loop(f),
            Statement::Loop(Loop::DoWhile(d)) => self.visit_do_while_loop(d),
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
            Expr::Prefix(e) => self.visit_prefix_expr(e),
            Expr::Function(e) => self.visit_function_expr(e),
            Expr::Array(e) => self.visit_array_literal(e),
            Expr::Object(e) => self.visit_object_literal(e),
            Expr::Compiled(mut buf) => {
                self.current_function_mut().buf.append(&mut buf);
                Ok(())
            }
            Expr::Empty => self.visit_empty_expr(),
        }
    }

    fn visit_binary_expression(
        &mut self,
        BinaryExpr { left, right, operator }: BinaryExpr<'a>,
    ) -> Result<(), CompileError> {
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
                        fn try_const_spec<'a>(ib: &mut InstructionBuilder<'_, 'a>, right: &Expr<'a>) -> bool {
                            if let Expr::Literal(LiteralExpr::Number(n)) = right {
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

                        if !try_const_spec(&mut ib, &right) {
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
            "Infinity" => ib.build_infinity(),
            "NaN" => ib.build_nan(),
            ident => match ib.find_local(ident) {
                Some((index, _, is_extern)) => ib.build_local_load(index, is_extern),
                _ => ib.build_global_load(ident)?,
            },
        };

        Ok(())
    }

    fn visit_unary_expression(&mut self, UnaryExpr { operator, expr }: UnaryExpr<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        // Special case delete operator, as it works different from other unary operators
        if let TokenType::Delete = operator {
            match *expr {
                Expr::PropertyAccess(PropertyAccessExpr {
                    computed,
                    property,
                    target,
                }) => match (*property, computed) {
                    (Expr::Literal(lit), false) => {
                        ib.accept_expr(*target)?;
                        let ident = lit.to_identifier();
                        let id = ib.current_function_mut().cp.add(Constant::Identifier(ident.into()))?;
                        ib.build_static_delete(id);
                    }
                    (expr, _) => {
                        ib.accept_expr(expr)?;
                        ib.accept_expr(*target)?;
                        ib.build_dynamic_delete();
                    }
                },
                Expr::Literal(lit) => {
                    ib.build_global();
                    let ident = lit.to_identifier();
                    let id = ib.current_function_mut().cp.add(Constant::Identifier(ident.into()))?;
                    ib.build_static_delete(id);
                }
                _ => {
                    ib.build_constant(Constant::Boolean(true))?;
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
                    return Err(CompileError::YieldOutsideGenerator);
                }

                ib.build_yield();
            }
            TokenType::Await => {
                if !ib.current_function().r#async {
                    return Err(CompileError::AwaitOutsideAsync);
                }

                ib.build_await();
            }
            _ => unimplementedc!("Unary operator {:?}", operator),
        }

        Ok(())
    }

    fn visit_variable_declaration(
        &mut self,
        VariableDeclarations(declarations): VariableDeclarations<'a>,
    ) -> Result<(), CompileError> {
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
                        unimplementedc!("Rest operator in object destructuring");
                    }

                    let field_count = fields
                        .len()
                        .try_into()
                        .map_err(|_| CompileError::DestructureLimitExceeded)?;

                    // Unwrap ok; checked at parse time
                    let value = value.ok_or(CompileError::MissingInitializerInDestructuring)?;
                    ib.accept_expr(value)?;

                    ib.build_objdestruct(field_count);

                    for (name, alias) in fields {
                        let name = alias.unwrap_or(name);
                        let id = ib.current_scope_mut().add_local(name, binding.kind, None)?;

                        let var_id = ib.current_function_mut().cp.add(Constant::Number(id as f64))?;
                        let ident_id = ib.current_function_mut().cp.add(Constant::Identifier(name.into()))?;
                        ib.writew(var_id);
                        ib.writew(ident_id);
                    }
                }
                VariableDeclarationName::ArrayDestructuring { fields, rest } => {
                    if rest.is_some() {
                        unimplementedc!("Rest operator in array destructuring");
                    }

                    let field_count = fields
                        .len()
                        .try_into()
                        .map_err(|_| CompileError::DestructureLimitExceeded)?;

                    // Unwrap ok; checked at parse time
                    let value = value.expect("Array destructuring requires a value");
                    ib.accept_expr(value)?;

                    ib.build_arraydestruct(field_count);

                    for name in fields {
                        let id = ib.current_scope_mut().add_local(name, binding.kind, None)?;

                        let var_id = ib.current_function_mut().cp.add(Constant::Number(id as f64))?;
                        ib.writew(var_id);
                    }
                }
            }
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

    fn visit_block_statement(&mut self, BlockStatement(stmt): BlockStatement<'a>) -> Result<(), CompileError> {
        self.current_scope_mut().enter();
        // Note: No `?` here because we need to always exit the scope
        let re = self.accept_multiple(stmt);
        self.current_scope_mut().exit();
        re
    }

    fn visit_function_declaration(&mut self, fun: FunctionDeclaration<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);
        let var_id = match fun.name {
            Some(name) => Some(
                ib.current_scope_mut()
                    .add_local(name, VariableDeclarationKind::Var, None)?,
            ),
            None => None,
        };

        ib.visit_function_expr(fun)?;
        if let Some(var_id) = var_id {
            ib.build_local_store(AssignKind::Assignment, var_id, false);
        }
        ib.build_pop();
        Ok(())
    }

    fn visit_while_loop(&mut self, WhileLoop { condition, body }: WhileLoop<'a>) -> Result<(), CompileError> {
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

    fn visit_do_while_loop(&mut self, DoWhileLoop { body, condition }: DoWhileLoop<'a>) -> Result<(), CompileError> {
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

    fn visit_assignment_expression(
        &mut self,
        AssignmentExpr { left, right, operator }: AssignmentExpr<'a>,
    ) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        match left {
            AssignmentTarget::Expr(left) => match *left {
                Expr::Literal(lit) => {
                    let ident = lit.to_identifier();
                    let local = ib.find_local(&ident);

                    if let Some((id, local, is_extern)) = local {
                        if matches!(local.binding().kind, VariableDeclarationKind::Const) {
                            return Err(CompileError::ConstAssignment);
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
                            _ => unimplementedc!("Unknown operator"),
                        }
                    } else {
                        macro_rules! assign {
                            ($kind:expr) => {{
                                ib.accept_expr(*right)?;
                                ib.build_global_store($kind, &ident)?;
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
                            _ => unimplementedc!("Unknown operator"),
                        }
                    }
                }
                Expr::PropertyAccess(prop) => {
                    ib.accept_expr(*prop.target)?;

                    macro_rules! staticassign {
                        ($lit:expr, $kind:expr) => {{
                            ib.accept_expr(*right)?;
                            let ident = $lit.to_identifier();
                            ib.build_static_prop_assign($kind, &ident)?;
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
                        (Expr::Literal(lit), false, TokenType::Assignment) => {
                            staticassign!(lit, AssignKind::Assignment)
                        }
                        (Expr::Literal(lit), false, TokenType::AdditionAssignment) => {
                            staticassign!(lit, AssignKind::AddAssignment)
                        }
                        (Expr::Literal(lit), false, TokenType::SubtractionAssignment) => {
                            staticassign!(lit, AssignKind::SubAssignment)
                        }
                        (Expr::Literal(lit), false, TokenType::MultiplicationAssignment) => {
                            staticassign!(lit, AssignKind::MulAssignment)
                        }
                        (Expr::Literal(lit), false, TokenType::DivisionAssignment) => {
                            staticassign!(lit, AssignKind::DivAssignment)
                        }
                        (Expr::Literal(lit), false, TokenType::RemainderAssignment) => {
                            staticassign!(lit, AssignKind::RemAssignment)
                        }
                        (Expr::Literal(lit), false, TokenType::ExponentiationAssignment) => {
                            staticassign!(lit, AssignKind::PowAssignment)
                        }
                        (Expr::Literal(lit), false, TokenType::LeftShiftAssignment) => {
                            staticassign!(lit, AssignKind::ShlAssignment)
                        }
                        (Expr::Literal(lit), false, TokenType::RightShiftAssignment) => {
                            staticassign!(lit, AssignKind::ShrAssignment)
                        }
                        (Expr::Literal(lit), false, TokenType::UnsignedRightShiftAssignment) => {
                            staticassign!(lit, AssignKind::UshrAssignment)
                        }
                        (Expr::Literal(lit), false, TokenType::BitwiseAndAssignment) => {
                            staticassign!(lit, AssignKind::BitAndAssignment)
                        }
                        (Expr::Literal(lit), false, TokenType::BitwiseOrAssignment) => {
                            staticassign!(lit, AssignKind::BitOrAssignment)
                        }
                        (Expr::Literal(lit), false, TokenType::BitwiseXorAssignment) => {
                            staticassign!(lit, AssignKind::BitXorAssignment)
                        }
                        (prop, true, TokenType::Assignment) => dynamicassign!(prop, AssignKind::Assignment),
                        (prop, true, TokenType::AdditionAssignment) => {
                            dynamicassign!(prop, AssignKind::AddAssignment)
                        }
                        (prop, true, TokenType::SubtractionAssignment) => {
                            dynamicassign!(prop, AssignKind::SubAssignment)
                        }
                        (prop, true, TokenType::MultiplicationAssignment) => {
                            dynamicassign!(prop, AssignKind::MulAssignment)
                        }
                        (prop, true, TokenType::DivisionAssignment) => {
                            dynamicassign!(prop, AssignKind::DivAssignment)
                        }
                        (prop, true, TokenType::RemainderAssignment) => {
                            dynamicassign!(prop, AssignKind::RemAssignment)
                        }
                        (prop, true, TokenType::ExponentiationAssignment) => {
                            dynamicassign!(prop, AssignKind::PowAssignment)
                        }
                        (prop, true, TokenType::LeftShiftAssignment) => {
                            dynamicassign!(prop, AssignKind::ShlAssignment)
                        }
                        (prop, true, TokenType::RightShiftAssignment) => {
                            dynamicassign!(prop, AssignKind::ShrAssignment)
                        }
                        (prop, true, TokenType::UnsignedRightShiftAssignment) => {
                            dynamicassign!(prop, AssignKind::UshrAssignment)
                        }
                        (prop, true, TokenType::BitwiseAndAssignment) => {
                            dynamicassign!(prop, AssignKind::BitAndAssignment)
                        }
                        (prop, true, TokenType::BitwiseOrAssignment) => {
                            dynamicassign!(prop, AssignKind::BitOrAssignment)
                        }
                        (prop, true, TokenType::BitwiseXorAssignment) => {
                            dynamicassign!(prop, AssignKind::BitXorAssignment)
                        }
                        _ => unimplementedc!("Assignment to computed property"),
                    }
                }
                _ => unimplementedc!("Assignment to non-identifier"),
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
        FunctionCall {
            constructor_call,
            target,
            arguments,
        }: FunctionCall<'a>,
    ) -> Result<(), CompileError> {
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
        fn try_spec_function_call<'a>(
            ib: &mut InstructionBuilder<'_, 'a>,
            target: &Expr<'a>,
            arguments: &[Expr<'a>],
        ) -> Result<bool, CompileError> {
            if let Expr::PropertyAccess(PropertyAccessExpr { target, property, .. }) = target {
                let Some(target) = target.as_identifier() else {
                    return Ok(false);
                };

                let Some(property) = property.as_identifier() else {
                    return Ok(false);
                };

                let Ok(arg_len) = u8::try_from(arguments.len()) else {
                    return Ok(false);
                };

                macro_rules! emit_spec {
                    ($spec:expr) => {{
                        for arg in arguments {
                            // TODO: we dont actually need to clone, we could take mem::take, if worth it
                            ib.accept_expr(arg.clone())?;
                        }
                        $spec(ib, arg_len);
                        return Ok(true);
                    }};
                }

                match (target, property) {
                    ("Math", "exp") => emit_spec!(InstructionBuilder::build_exp),
                    ("Math", "log2") => emit_spec!(InstructionBuilder::build_log2),
                    ("Math", "expm1") => emit_spec!(InstructionBuilder::build_expm1),
                    ("Math", "cbrt") => emit_spec!(InstructionBuilder::build_cbrt),
                    ("Math", "clz32") => emit_spec!(InstructionBuilder::build_clz32),
                    ("Math", "atanh") => emit_spec!(InstructionBuilder::build_atanh),
                    ("Math", "atan2") => emit_spec!(InstructionBuilder::build_atanh2),
                    ("Math", "round") => emit_spec!(InstructionBuilder::build_round),
                    ("Math", "acosh") => emit_spec!(InstructionBuilder::build_acosh),
                    ("Math", "abs") => emit_spec!(InstructionBuilder::build_abs),
                    ("Math", "sinh") => emit_spec!(InstructionBuilder::build_sinh),
                    ("Math", "sin") => emit_spec!(InstructionBuilder::build_sin),
                    ("Math", "ceil") => emit_spec!(InstructionBuilder::build_ceil),
                    ("Math", "tan") => emit_spec!(InstructionBuilder::build_tan),
                    ("Math", "trunc") => emit_spec!(InstructionBuilder::build_trunc),
                    ("Math", "asinh") => emit_spec!(InstructionBuilder::build_asinh),
                    ("Math", "log10") => emit_spec!(InstructionBuilder::build_log10),
                    ("Math", "asin") => emit_spec!(InstructionBuilder::build_asin),
                    ("Math", "random") => emit_spec!(InstructionBuilder::build_random),
                    ("Math", "log1p") => emit_spec!(InstructionBuilder::build_log1p),
                    ("Math", "sqrt") => emit_spec!(InstructionBuilder::build_sqrt),
                    ("Math", "atan") => emit_spec!(InstructionBuilder::build_atan),
                    ("Math", "log") => emit_spec!(InstructionBuilder::build_log),
                    ("Math", "floor") => emit_spec!(InstructionBuilder::build_floor),
                    ("Math", "cosh") => emit_spec!(InstructionBuilder::build_cosh),
                    ("Math", "acos") => emit_spec!(InstructionBuilder::build_acos),
                    ("Math", "cos") => emit_spec!(InstructionBuilder::build_cos),
                    _ => {}
                }
            }
            Ok(false)
        }

        if try_spec_function_call(&mut ib, &target, &arguments)? {
            return Ok(());
        }

        let has_this = if let Expr::PropertyAccess(p) = *target {
            ib.visit_property_access_expr(p, true)?;
            true
        } else {
            ib.accept_expr(*target)?;
            false
        };

        let argc = arguments
            .len()
            .try_into()
            .map_err(|_| CompileError::ParameterLimitExceeded)?;

        for arg in arguments {
            ib.accept_expr(arg)?;
        }

        let meta = FunctionCallMetadata::new_checked(argc, constructor_call, has_this)
            .ok_or(CompileError::ParameterLimitExceeded)?;

        ib.build_call(meta);

        Ok(())
    }

    fn visit_return_statement(&mut self, ReturnStatement(stmt): ReturnStatement<'a>) -> Result<(), CompileError> {
        let tc_depth = self.current_function().try_catch_depth;
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

        match *expr {
            Expr::Literal(lit) => {
                let ident = lit.to_identifier();

                if let Some((id, loc, is_extern)) = ib.find_local(&ident) {
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
                        TokenType::Increment => ib.build_global_store(AssignKind::PostfixIncrement, &ident)?,
                        TokenType::Decrement => ib.build_global_store(AssignKind::PostfixDecrement, &ident)?,
                        _ => unreachable!("Token never emitted"),
                    }
                }
            }
            Expr::PropertyAccess(prop) => {
                ib.accept_expr(*prop.target)?;

                match (*prop.property, prop.computed) {
                    (Expr::Literal(lit), false) => {
                        let ident = lit.to_identifier();
                        match tt {
                            TokenType::Increment => {
                                ib.build_static_prop_assign(AssignKind::PostfixIncrement, &ident)?
                            }
                            TokenType::Decrement => {
                                ib.build_static_prop_assign(AssignKind::PostfixDecrement, &ident)?
                            }
                            _ => unreachable!("Token never emitted"),
                        }
                    }
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
            _ => unimplementedc!("Non-identifier postfix expression"),
        }

        Ok(())
    }

    fn visit_prefix_expr(&mut self, (tt, expr): Postfix<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        match *expr {
            Expr::Literal(lit) => {
                let ident = lit.to_identifier();

                if let Some((id, loc, is_extern)) = ib.find_local(&ident) {
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
                        TokenType::Increment => ib.build_global_store(AssignKind::PrefixIncrement, &ident)?,
                        TokenType::Decrement => ib.build_global_store(AssignKind::PrefixDecrement, &ident)?,
                        _ => unreachable!("Token never emitted"),
                    }
                }
            }
            Expr::PropertyAccess(prop) => {
                ib.accept_expr(*prop.target)?;

                match (*prop.property, prop.computed) {
                    (Expr::Literal(lit), false) => {
                        let ident = lit.to_identifier();
                        match tt {
                            TokenType::Increment => ib.build_static_prop_assign(AssignKind::PrefixIncrement, &ident)?,
                            TokenType::Decrement => ib.build_static_prop_assign(AssignKind::PrefixDecrement, &ident)?,
                            _ => unreachable!("Token never emitted"),
                        }
                    }
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
            _ => unimplementedc!("Non-identifier postfix expression"),
        }

        Ok(())
    }

    fn visit_function_expr(
        &mut self,
        FunctionDeclaration {
            id,
            name,
            parameters: arguments,
            mut statements,
            ty,
            r#async,
            ty_segment: _,
        }: FunctionDeclaration<'a>,
    ) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);
        ib.function_stack.push(FunctionLocalState::new(ty, id, r#async));

        let mut rest_local = None;

        for (param, default, _ty) in &arguments {
            let name = match param {
                Parameter::Identifier(ident) => ident,
                Parameter::Spread(ident) => ident,
            };

            let id = ib
                .tcx
                .scope_mut(id)
                .add_local(name, VariableDeclarationKind::Var, None)?;

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
        for stmt in statements {
            ib.accept(stmt)?;
        }

        let cmp = ib.function_stack.pop().expect("Missing function state");
        let scope = ib.tcx.scope(id);
        let externals = scope.externals();
        let locals = scope.locals().len();

        let function = Function {
            buffer: Buffer(Cell::new(cmp.buf.into())),
            constants: cmp.cp.into_vec().into(),
            locals,
            name: name.map(ToOwned::to_owned),
            ty,
            params: match arguments.last() {
                Some((Parameter::Spread(..), ..)) => arguments.len() - 1,
                _ => arguments.len(),
            },
            externals: externals.into(),
            r#async,
            rest_local,
            poison_ips: RefCell::new(HashSet::new()),
        };
        ib.build_constant(Constant::Function(Rc::new(function)))?;

        Ok(())
    }

    fn visit_array_literal(&mut self, ArrayLiteral(exprs): ArrayLiteral<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        let len = exprs
            .len()
            .try_into()
            .map_err(|_| CompileError::ArrayLitLimitExceeded)?;

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

    fn visit_object_literal(&mut self, ObjectLiteral(exprs): ObjectLiteral<'a>) -> Result<(), CompileError> {
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

        ib.build_objlit(members)?;
        Ok(())
    }

    fn visit_try_catch(&mut self, TryCatch { try_, catch, .. }: TryCatch<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        ib.build_try_block();

        ib.current_function_mut().try_catch_depth += 1;
        ib.current_scope_mut().enter();
        ib.accept(*try_)?;
        ib.current_scope_mut().exit();
        ib.current_function_mut().try_catch_depth -= 1;

        ib.build_jmp(Label::TryEnd, true);

        ib.add_local_label(Label::Catch);

        ib.current_scope_mut().enter();

        if let Some(ident) = catch.ident {
            let id = ib
                .current_scope_mut()
                .add_local(ident, VariableDeclarationKind::Var, None)?;

            if id == u16::MAX {
                // Max u16 value is reserved for "no binding"
                return Err(CompileError::LocalLimitExceeded);
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

    fn visit_for_of_loop(&mut self, ForOfLoop { binding, expr, body }: ForOfLoop<'a>) -> Result<(), CompileError> {
        self.visit_for_each_kinded_loop(ForEachLoopKind::ForOf, binding, expr, body)
    }

    fn visit_for_in_loop(&mut self, ForInLoop { binding, expr, body }: ForInLoop<'a>) -> Result<(), CompileError> {
        self.visit_for_each_kinded_loop(ForEachLoopKind::ForIn, binding, expr, body)
    }

    fn visit_import_statement(&mut self, import: ImportKind<'a>) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        match import {
            ImportKind::Dynamic(ex) => {
                ib.accept_expr(ex)?;
                ib.build_dynamic_import();
            }
            ref kind @ (ImportKind::DefaultAs(ref spec, ref path) | ImportKind::AllAs(ref spec, ref path)) => {
                let local_id = ib.current_scope_mut().add_local(
                    match spec {
                        SpecifierKind::Ident(id) => id,
                    },
                    VariableDeclarationKind::Var,
                    None,
                )?;

                let path_id = ib
                    .current_function_mut()
                    .cp
                    .add(Constant::String(path.as_ref().into()))?;

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
                    let ident_id = ib.current_function_mut().cp.add(Constant::Identifier(name.into()))?;

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

                ib.build_named_export(&it)?;
            }
            ExportKind::NamedVar(VariableDeclarations(vars)) => {
                let mut it: Vec<&'a str> = Vec::with_capacity(vars.len());

                for var in &vars {
                    match &var.binding.name {
                        VariableDeclarationName::Identifier(ident) => it.push(ident),
                        VariableDeclarationName::ArrayDestructuring { fields, rest } => {
                            it.extend(fields.iter());
                            it.extend(rest);
                        }
                        VariableDeclarationName::ObjectDestructuring { fields, rest } => {
                            it.extend(fields.iter().map(|(name, ident)| ident.unwrap_or(name)));
                            it.extend(rest);
                        }
                    }
                }

                self.visit_variable_declaration(VariableDeclarations(vars))?;
                self.visit_export_statement(ExportKind::Named(it))?;
            }
        };
        Ok(())
    }

    fn visit_empty_statement(&mut self) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_break(&mut self) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);
        let breakable = *ib
            .current_function_mut()
            .breakables
            .last()
            .ok_or(CompileError::IllegalBreak)?;
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

    fn visit_continue(&mut self) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);
        let breakable = *ib
            .current_function_mut()
            .breakables
            .last()
            .ok_or(CompileError::IllegalBreak)?;
        match breakable {
            Breakable::Loop { loop_id } => {
                ib.build_jmp(Label::LoopIncrement { loop_id }, false);
            }
            Breakable::Switch { .. } => {
                // TODO: make it possible to use `continue` in loops even if its used in a switch
                unimplementedc!("`continue` used inside of a switch statement");
            }
        }
        Ok(())
    }

    fn visit_debugger(&mut self) -> Result<(), CompileError> {
        InstructionBuilder::new(self).build_debugger();
        Ok(())
    }

    fn visit_empty_expr(&mut self) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_class_declaration(&mut self, class: Class<'a>) -> Result<(), CompileError> {
        if class.extends.is_some() {
            unimplementedc!("Extending class");
        }

        let mut ib = InstructionBuilder::new(self);

        let constructor = class.members.iter().find_map(|member| {
            if let ClassMemberKind::Method(method) = &member.kind {
                if method.name == Some("constructor") {
                    return Some(method.clone());
                }
            }

            None
        });

        let binding_id = match class.name {
            Some(name) => ib
                .current_scope_mut()
                .add_local(name, VariableDeclarationKind::Var, None)?,
            None => ib
                .current_scope_mut()
                .add_local("DesugaredClass", VariableDeclarationKind::Unnameable, None)?,
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

        ib.visit_assignment_expression(AssignmentExpr::new_local_place(
            binding_id,
            Expr::Function(desugared_class),
            TokenType::Assignment,
        ))?;
        let load_class_binding = Expr::Compiled(compile_local_load(binding_id, false));

        for member in class.members {
            if let ClassMemberKind::Method(method) = member.kind {
                let name = method.name.expect("Class method did not have a name");

                ib.accept(Statement::Expression(Expr::Assignment(AssignmentExpr {
                    left: AssignmentTarget::Expr(match member.static_ {
                        true => Box::new(Expr::PropertyAccess(PropertyAccessExpr {
                            computed: false,
                            property: Box::new(Expr::string_literal(Cow::Borrowed(name))),
                            target: Box::new(load_class_binding.clone()),
                        })),
                        false => Box::new(Expr::PropertyAccess(PropertyAccessExpr {
                            computed: false,
                            property: Box::new(Expr::string_literal(Cow::Borrowed(name))),
                            target: Box::new(Expr::PropertyAccess(PropertyAccessExpr {
                                computed: false,
                                property: Box::new(Expr::string_literal(Cow::Borrowed("prototype"))),
                                target: Box::new(load_class_binding.clone()),
                            })),
                        })),
                    }),
                    operator: TokenType::Assignment,
                    right: Box::new(Expr::Function(method)),
                })))?
            }
        }

        Ok(())
    }

    fn visit_switch_statement(
        &mut self,
        SwitchStatement { expr, cases, default }: SwitchStatement<'a>,
    ) -> Result<(), CompileError> {
        let mut ib = InstructionBuilder::new(self);

        let switch_id = ib.current_function_mut().prepare_switch();
        let has_default = default.is_some();
        let case_count = cases.len();

        // First, compile all case expressions in reverse order, so that the first pop() in the vm
        // will be the first case value
        for case in cases.iter().rev() {
            ib.accept_expr(case.value.clone())?;
        }

        // Then, compile switch expression
        ib.accept_expr(expr)?;

        // Write switch metadata (case count, has default case)
        let case_count = case_count
            .try_into()
            .map_err(|_| CompileError::SwitchCaseLimitExceeded)?;

        ib.build_switch(case_count, has_default);

        // Then, build jump headers for every case
        for (case_id, ..) in cases.iter().enumerate() {
            ib.build_jmp_header(
                Label::SwitchCase {
                    case_id: case_id as u16,
                },
                true,
            );
        }
        if has_default {
            ib.build_jmp_header(Label::SwitchCase { case_id: case_count }, true);
        }

        // If no case matches, then jump to SwitchEnd
        ib.build_jmp(Label::SwitchEnd { switch_id }, false);

        // Finally, compile case bodies
        // All of the case bodies must be adjacent because of fallthrough
        for (case_id, case) in cases.into_iter().enumerate() {
            ib.add_local_label(Label::SwitchCase {
                case_id: case_id as u16,
            });
            ib.accept_multiple(case.body)?;
        }
        if let Some(default) = default {
            ib.add_local_label(Label::SwitchCase { case_id: case_count });
            ib.accept_multiple(default)?;
        }

        ib.current_function_mut()
            .add_global_label(Label::SwitchEnd { switch_id });
        ib.current_function_mut().exit_switch();
        Ok(())
    }
}
