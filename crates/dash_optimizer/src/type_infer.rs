use std::cell::RefCell;

use dash_log::debug;
use dash_middle::compiler::scope::{BlockScope, CompileValueType, FunctionScope, Local, ScopeGraph, ScopeKind};
use dash_middle::interner::{Symbol, sym};
use dash_middle::lexer::token::TokenType;
use dash_middle::parser::expr::{
    ArrayLiteral, ArrayMemberKind, AssignmentExpr, AssignmentTarget, BinaryExpr, CallArgumentKind, ConditionalExpr,
    Expr, ExprKind, FunctionCall, GroupingExpr, LiteralExpr, ObjectLiteral, ObjectMemberKind,
    OptionalChainingExpression, PropertyAccessExpr, UnaryExpr,
};
use dash_middle::parser::statement::{
    Binding, BlockStatement, Class, ClassMemberKey, ClassMemberValue, DoWhileLoop, ExportKind, ForInLoop, ForLoop,
    ForOfLoop, FunctionDeclaration, IfStatement, ImportKind, LocalId, Loop, Parameter, Pattern, ReturnStatement,
    ScopeId, SpecifierKind, Statement, StatementKind, SwitchCase, SwitchStatement, TryCatch, VariableBinding,
    VariableDeclaration, VariableDeclarationKind, VariableDeclarationName, VariableDeclarations, WhileLoop,
};

#[derive(Debug)]
pub struct LocalDeclToSlot(Vec<u16>);

impl LocalDeclToSlot {
    pub fn slot_from_local(&self, local: LocalId) -> u16 {
        self.0[local.0]
    }
}

#[derive(Debug)]
pub enum InferMode<'s> {
    /// Initial, main phase:
    /// discover local variables as well as scopes by populating the ScopeGraph
    Discover(&'s mut ScopeGraph, &'s mut LocalDeclToSlot),
    /// Don't add any more scopes or locals.
    /// In other words, visiting the same node(s) multiple times in this mode should not have any side effects
    /// with respect to the ScopeGraph
    View(&'s ScopeGraph),
}

#[derive(Debug)]
pub struct TypeInferCtx<'s> {
    mode: InferMode<'s>,
    current: ScopeId,
    current_function: ScopeId,
}

impl<'s> TypeInferCtx<'s> {
    pub fn new(mut mode: InferMode<'s>) -> Self {
        if let InferMode::Discover(scopes, _) = &mut mode {
            scopes[ScopeId::ROOT].kind = ScopeKind::Function(FunctionScope { locals: Vec::new() });
        }

        Self {
            mode,
            current: ScopeId::ROOT,
            current_function: ScopeId::ROOT,
        }
    }

    pub fn view(mode: InferMode<'s>, current: ScopeId, current_function: ScopeId) -> Self {
        Self {
            mode,
            current,
            current_function,
        }
    }

    fn with_block_scope(&mut self, id: ScopeId, f: impl FnOnce(&mut Self)) {
        let parent = self.current;
        self.current = id;

        match &mut self.mode {
            InferMode::Discover(scopes, _) => {
                let scope = &mut scopes[id];
                scope.parent = Some(parent);
                scope.kind = ScopeKind::Block(BlockScope {
                    enclosing_function: self.current_function,
                });
                f(self);
            }
            InferMode::View(_) => f(self),
        }

        self.current = parent;
    }
    fn with_function_scope(&mut self, id: ScopeId, f: impl FnOnce(&mut Self)) {
        let parent = self.current;
        let parent_func = self.current_function;
        self.current = id;
        self.current_function = id;

        match &mut self.mode {
            InferMode::Discover(scopes, _) => {
                let scope = &mut scopes[id];
                scope.parent = Some(parent);
                scope.kind = ScopeKind::Function(FunctionScope { locals: Vec::new() });

                f(self);
            }
            InferMode::View(_) => f(self),
        }

        self.current = parent;
        self.current_function = parent_func;
    }

    fn add_local_in_scope(
        &mut self,
        at: ScopeId,
        binding: Binding,
        kind: VariableDeclarationKind,
        ty: Option<CompileValueType>,
    ) {
        if let InferMode::Discover(scopes, decls) = &mut self.mode {
            let enclosing_function = scopes.enclosing_function_of(at);
            if kind == VariableDeclarationKind::Var && at != enclosing_function {
                // `var` declarations are hoisted to the top
                self.add_local_in_scope(enclosing_function, binding, kind, ty);
                return;
            }

            if let Some((_, slot)) = scopes[at]
                .declarations
                .iter()
                .copied()
                .find(|decl| decl.0 == binding.ident)
            {
                // This should be a normal error: it happens for e.g.
                // let x = 1;
                // let x = 2;
                // assert!(
                //     kind == VariableDeclarationKind::Var
                //         && scopes[enclosing_function].expect_function().locals[slot as usize].kind
                //             == VariableDeclarationKind::Var
                // );
                decls.0[binding.id.0] = slot;
            } else {
                let local_id = scopes[enclosing_function]
                    .expect_function_mut()
                    .add_local(Local {
                        name: binding.ident,
                        kind,
                        inferred_type: RefCell::new(ty),
                    })
                    .unwrap();

                scopes[at].declarations.push((binding.ident, local_id));
                decls.0[binding.id.0] = local_id;
            }
        }
    }

    /// NOTE: in InferMode::View, this has no effect and is simply a no-op
    fn add_local(&mut self, binding: Binding, kind: VariableDeclarationKind, ty: Option<CompileValueType>) {
        self.add_local_in_scope(self.current, binding, kind, ty)
    }

    pub fn visit_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Block(block) => self.visit_block_statement(block),
            StatementKind::Expression(expr) => drop(self.visit(expr)),
            StatementKind::Variable(stmt) => self.visit_variable_declaration(stmt),
            StatementKind::If(stmt) => self.visit_if_statement(stmt),
            StatementKind::Function(expr) => drop(self.visit_function_expression(expr)),
            StatementKind::Loop(expr) => self.visit_loop_statement(expr),
            StatementKind::Return(stmt) => self.visit_return_statement(stmt),
            StatementKind::Try(stmt) => self.visit_try_statement(stmt),
            StatementKind::Throw(expr) => drop(self.visit(expr)),
            StatementKind::Import(ImportKind::Dynamic(expr)) => drop(self.visit(expr)),
            StatementKind::Import(
                ImportKind::DefaultAs(SpecifierKind::Ident(sym), ..) | ImportKind::AllAs(SpecifierKind::Ident(sym), ..),
            ) => self.add_local(*sym, VariableDeclarationKind::Var, None),
            StatementKind::Export(ExportKind::Default(expr)) => drop(self.visit(expr)),
            StatementKind::Export(ExportKind::Named(..)) => {}
            StatementKind::Export(ExportKind::NamedVar(stmt)) => self.visit_variable_declaration(stmt),
            StatementKind::Class(stmt) => self.visit_class_statement(stmt),
            StatementKind::Switch(stmt) => self.visit_switch_statement(stmt),
            StatementKind::Continue(_) => {}
            StatementKind::Break(_) => {}
            StatementKind::Debugger => {}
            StatementKind::Empty => {}
            StatementKind::Labelled(_, s) => self.visit_statement(s),
        }
    }

    pub fn visit_maybe_statement(&mut self, stmt: Option<&Statement>) {
        if let Some(stmt) = stmt {
            self.visit_statement(stmt);
        }
    }

    pub fn visit_many_statements(&mut self, stmt: &[Statement]) {
        for stmt in stmt {
            self.visit_statement(stmt);
        }
    }

    pub fn visit_maybe_expr(&mut self, expr: Option<&Expr>) -> Option<CompileValueType> {
        if let Some(expr) = expr { self.visit(expr) } else { None }
    }

    pub fn visit_return_statement(&mut self, ReturnStatement(expr): &ReturnStatement) {
        self.visit(expr);
    }

    pub fn visit_block_statement(&mut self, BlockStatement(stmt, id): &BlockStatement) {
        self.with_block_scope(*id, |this| this.visit_many_statements(stmt));
    }

    pub fn visit_try_statement(&mut self, TryCatch { try_, catch, finally }: &TryCatch) {
        self.visit_statement(try_);
        if let Some(catch) = catch {
            self.with_block_scope(catch.body.1, |this| {
                if let Some(ident) = catch.binding {
                    this.add_local(ident, VariableDeclarationKind::Let, None);
                }
                this.visit_many_statements(&catch.body.0);
            });
        }
        self.visit_maybe_statement(finally.as_deref());
    }

    pub fn visit_class_statement(
        &mut self,
        Class {
            extends, members, name, ..
        }: &Class,
    ) {
        self.visit_maybe_expr(extends.as_deref());
        for member in members {
            if let ClassMemberKey::Computed(expr) = &member.key {
                self.visit(expr);
            }

            match &member.value {
                ClassMemberValue::Method(method)
                | ClassMemberValue::Getter(method)
                | ClassMemberValue::Setter(method) => drop(self.visit_function_expression(method)),
                ClassMemberValue::Field(field) => drop(self.visit_maybe_expr(field.as_ref())),
            }
        }

        if let Some(name) = name {
            self.visit_variable_binding(
                &VariableBinding {
                    name: VariableDeclarationName::Identifier(*name),
                    kind: VariableDeclarationKind::Var,
                    ty: None,
                },
                None,
            );
        }
    }

    pub fn visit_switch_statement(&mut self, SwitchStatement { expr, default, cases }: &SwitchStatement) {
        self.visit(expr);

        if let Some(default) = default {
            self.visit_many_statements(default);
        }

        for SwitchCase { value, body } in cases {
            self.visit(value);
            self.visit_many_statements(body);
        }
    }

    pub fn visit_loop_statement(&mut self, loop_: &Loop) {
        match loop_ {
            Loop::For(ForLoop {
                init,
                condition,
                finalizer,
                body,
                scope,
            }) => {
                self.with_block_scope(*scope, |this| {
                    this.visit_maybe_statement(init.as_deref());
                    this.visit_maybe_expr(condition.as_ref());
                    this.visit_maybe_expr(finalizer.as_ref());
                    this.visit_statement(body);
                });
            }
            Loop::ForOf(ForOfLoop {
                expr,
                body,
                binding,
                scope,
            }) => {
                self.with_block_scope(*scope, |this| {
                    this.visit_variable_binding(binding, None);
                    this.visit(expr);
                    this.visit_statement(body);
                });
            }
            Loop::ForIn(ForInLoop {
                expr,
                body,
                binding,
                scope,
            }) => {
                self.with_block_scope(*scope, |this| {
                    this.visit_variable_binding(binding, None);
                    this.visit(expr);
                    this.visit_statement(body);
                });
            }
            Loop::While(WhileLoop { condition, body }) => {
                self.visit(condition);
                self.visit_statement(body);
            }
            Loop::DoWhile(DoWhileLoop { body, condition }) => {
                self.visit(condition);
                self.visit_statement(body);
            }
        }
    }

    fn visit_pattern(&mut self, kind: VariableDeclarationKind, pat: &Pattern) {
        match *pat {
            Pattern::Object { ref fields, rest } => {
                for &(id, field, alias, ref default) in fields {
                    let name = alias.unwrap_or(field);
                    self.add_local(Binding { id, ident: name }, kind, None);
                    self.visit_maybe_expr(default.as_ref());
                }
                if let Some(rest) = rest {
                    self.add_local(rest, kind, None);
                }
            }
            Pattern::Array { ref fields, rest } => {
                for &(field, ref default) in fields.iter().flatten() {
                    self.add_local(field, kind, None);
                    self.visit_maybe_expr(default.as_ref());
                }
                if let Some(rest) = rest {
                    self.add_local(rest, kind, None);
                }
            }
        }
    }

    fn visit_variable_binding(&mut self, binding: &VariableBinding, value: Option<&Expr>) {
        let ty = match value {
            Some(expr) => self.visit(expr),
            None => Some(CompileValueType::Uninit),
        };
        debug!("discovered new variable(s) {binding:?} of type {:?}", ty);

        match binding.name {
            VariableDeclarationName::Identifier(name) => self.add_local(name, binding.kind, ty),
            VariableDeclarationName::Pattern(ref pat) => self.visit_pattern(binding.kind, pat),
        }
    }

    pub fn visit_variable_declaration(&mut self, VariableDeclarations(declarations): &VariableDeclarations) {
        for VariableDeclaration { binding, value } in declarations {
            self.visit_variable_binding(binding, value.as_ref());
        }
    }

    pub fn visit_if_statement(
        &mut self,
        IfStatement {
            condition,
            then,
            branches,
            el,
        }: &IfStatement,
    ) {
        self.visit(condition);
        self.visit_statement(then);
        if let Some(el) = el {
            self.visit_statement(el);
        }
        for branch in branches {
            self.visit_if_statement(branch);
        }
    }

    pub fn visit(&mut self, expression: &Expr) -> Option<CompileValueType> {
        match &expression.kind {
            ExprKind::Binary(expr) => self.visit_binary_expression(expr),
            ExprKind::Grouping(expr) => self.visit_grouping_expression(expr),
            ExprKind::Literal(expr) => self.visit_literal_expression(expr),
            ExprKind::Unary(expr) => self.visit_unary_expression(expr),
            ExprKind::Assignment(expr) => self.visit_assignment_expression(expr),
            ExprKind::Call(expr) => self.visit_call_expression(expr),
            ExprKind::Conditional(expr) => self.visit_conditional_expression(expr),
            ExprKind::PropertyAccess(expr) => self.visit_property_access_expression(expr),
            ExprKind::Sequence(..) => panic!("Unemitted expr type: Sequence"),
            ExprKind::Prefix((tt, expr)) => self.visit_prefix_expression(expr, *tt),
            ExprKind::Postfix((tt, expr)) => self.visit_postfix_expression(expr, *tt),
            ExprKind::Function(expr) => self.visit_function_expression(expr),
            ExprKind::Class(class) => self.visit_class_expression(class),
            ExprKind::Array(expr) => self.visit_array_expression(expr),
            ExprKind::Object(expr) => self.visit_object_expression(expr),
            ExprKind::Chaining(OptionalChainingExpression { base, components: _ }) => self.visit(base),
            ExprKind::Compiled(..) => None,
            ExprKind::Empty => None,
            ExprKind::NewTarget => None,
            ExprKind::YieldStar(e) => {
                self.visit(e);
                None
            }
        }
    }

    pub fn visit_binary_expression(
        &mut self,
        BinaryExpr { left, right, operator }: &BinaryExpr,
    ) -> Option<CompileValueType> {
        let left = self.visit(left);
        let right = self.visit(right);

        match (left, right, operator) {
            (Some(CompileValueType::String), _, TokenType::Plus) => Some(CompileValueType::String),
            (_, Some(CompileValueType::String), TokenType::Plus) => Some(CompileValueType::String),
            (_, _, TokenType::Greater) => Some(CompileValueType::Boolean),
            (_, _, TokenType::GreaterEqual) => Some(CompileValueType::Boolean),
            (_, _, TokenType::Less) => Some(CompileValueType::Boolean),
            (_, _, TokenType::LessEqual) => Some(CompileValueType::Boolean),
            (_, _, TokenType::Equality) => Some(CompileValueType::Boolean),
            (_, _, TokenType::Inequality) => Some(CompileValueType::Boolean),
            (_, _, TokenType::StrictEquality) => Some(CompileValueType::Boolean),
            (_, _, TokenType::StrictInequality) => Some(CompileValueType::Boolean),
            (Some(CompileValueType::Number), Some(CompileValueType::Number), _) => Some(CompileValueType::Number),
            (_, _, TokenType::Minus | TokenType::Star | TokenType::Slash) => Some(CompileValueType::Number),
            _ => None,
        }
    }

    fn visit_class_expression(&mut self, class: &Class) -> Option<CompileValueType> {
        self.visit_class_statement(class);
        None
    }

    pub fn visit_grouping_expression(&mut self, GroupingExpr(expression): &GroupingExpr) -> Option<CompileValueType> {
        let mut ty = None;
        for expression in expression {
            ty = self.visit(expression);
        }
        ty
    }

    pub fn find_local(&self, ident: Symbol) -> Option<&Local> {
        let scopes: &ScopeGraph = match &self.mode {
            InferMode::Discover(scopes, _) => scopes,
            InferMode::View(scopes) => scopes,
        };

        let res = scopes.find(self.current, ident)?;

        let local_enclosing_function = scopes.enclosing_function_of(res.scope);
        let local = &scopes[local_enclosing_function].expect_function().locals[res.slot as usize];

        if local_enclosing_function != self.current_function {
            let mut ty = local.inferred_type.borrow_mut();
            if ty.is_some() {
                *ty = Some(CompileValueType::Extern);
            }
        }

        Some(local)
    }

    pub fn visit_literal_expression(&mut self, expression: &LiteralExpr) -> Option<CompileValueType> {
        match expression {
            LiteralExpr::Boolean(..) => Some(CompileValueType::Boolean),
            LiteralExpr::Identifier(identifier) => match self.find_local(*identifier) {
                Some(local) => local.inferred_type().borrow().clone(),
                _ => None,
            },
            LiteralExpr::Number(..) => Some(CompileValueType::Number),
            LiteralExpr::String(..) => Some(CompileValueType::String),
            LiteralExpr::Regex(..) => None,
            LiteralExpr::Null => Some(CompileValueType::Null),
            LiteralExpr::Undefined => Some(CompileValueType::Undefined),
        }
    }

    pub fn visit_unary_expression(&mut self, UnaryExpr { expr, operator }: &UnaryExpr) -> Option<CompileValueType> {
        self.visit(expr);
        match operator {
            TokenType::Plus | TokenType::Minus => Some(CompileValueType::Number),
            TokenType::Typeof => Some(CompileValueType::String),
            _ => None,
        }
    }

    pub fn visit_assignment_expression(
        &mut self,
        AssignmentExpr { left, right, .. }: &AssignmentExpr,
    ) -> Option<CompileValueType> {
        let AssignmentTarget::Expr(left) = left else {
            panic!("Cannot infer type for assignment place LocalId");
        };

        let update_ty = |left_ty: &RefCell<Option<CompileValueType>>, ty: Option<CompileValueType>| {
            let mut left_ty = left_ty.borrow_mut();
            if !matches!(*left_ty, Some(CompileValueType::Extern)) {
                *left_ty = ty;
            }
        };

        self.visit(left);
        let right_type = self.visit(right);

        // Also propagate assignment to target
        if let ExprKind::Literal(LiteralExpr::Identifier(ident)) = &left.kind {
            if let Some(local) = self.find_local(*ident) {
                let left_type = local.inferred_type();
                let left_type_ref = left_type.borrow();

                if left_type_ref.as_ref() == right_type.as_ref() {
                    // Assign value is the same, no change.
                } else {
                    debug!(
                        "variable {} changed type {:?} -> {:?}",
                        ident, left_type_ref, right_type
                    );

                    match (left_type_ref.as_ref(), right_type.as_ref()) {
                        (Some(left), Some(right)) => {
                            let left = left.clone();
                            let right = right.clone();
                            drop(left_type_ref);
                            update_ty(
                                left_type,
                                Some(CompileValueType::Either(Box::new(left), Box::new(right))),
                            );
                        }
                        (_, Some(right)) => {
                            drop(left_type_ref);
                            update_ty(left_type, Some(CompileValueType::Maybe(Box::new(right.clone()))));
                        }
                        (_, _) => {
                            drop(left_type_ref);
                            update_ty(left_type, None);
                        }
                    }
                }
            }
        }

        right_type
    }

    pub fn visit_call_expression(
        &mut self,
        FunctionCall { target, arguments, .. }: &FunctionCall,
    ) -> Option<CompileValueType> {
        self.visit(target);
        for argument in arguments {
            match argument {
                CallArgumentKind::Normal(expr) => drop(self.visit(expr)),
                CallArgumentKind::Spread(expr) => drop(self.visit(expr)),
            }
        }
        None
    }

    pub fn visit_conditional_expression(
        &mut self,
        ConditionalExpr { then, el, condition }: &ConditionalExpr,
    ) -> Option<CompileValueType> {
        self.visit(condition);
        let then_ty = self.visit(then);
        let else_ty = self.visit(el);

        if then_ty == else_ty {
            then_ty
        } else if let (Some(then_ty), Some(else_ty)) = (then_ty, else_ty) {
            Some(CompileValueType::Either(Box::new(then_ty), Box::new(else_ty)))
        } else {
            None
        }
    }

    pub fn visit_property_access_expression(
        &mut self,
        PropertyAccessExpr { target, property, .. }: &PropertyAccessExpr,
    ) -> Option<CompileValueType> {
        self.visit(target);
        self.visit(property);
        None
    }

    pub fn visit_prefix_expression(&mut self, expression: &Expr, _: TokenType) -> Option<CompileValueType> {
        self.visit(expression);
        Some(CompileValueType::Number)
    }

    pub fn visit_postfix_expression(&mut self, expression: &Expr, _: TokenType) -> Option<CompileValueType> {
        self.visit(expression);
        Some(CompileValueType::Number)
    }

    pub fn visit_function_expression(
        &mut self,
        FunctionDeclaration {
            parameters,
            statements,
            id,
            name,
            ..
        }: &FunctionDeclaration,
    ) -> Option<CompileValueType> {
        let sub_func_id = *id;

        if let Some(name) = *name {
            debug!("visit function {name}");

            self.add_local(name, VariableDeclarationKind::Var, None);
        }
        self.with_function_scope(sub_func_id, |this| {
            for (param, expr, _) in parameters {
                match *param {
                    Parameter::Identifier(binding) | Parameter::SpreadIdentifier(binding) => {
                        this.add_local(binding, VariableDeclarationKind::Var, None)
                    }
                    Parameter::Pattern(local_id, _) | Parameter::SpreadPattern(local_id, _) => {
                        // Actual patterns bindings are visited in a second pass so that the actual parameters locals get their ids first
                        this.add_local(
                            Binding {
                                ident: sym::empty,
                                id: local_id,
                            },
                            VariableDeclarationKind::Unnameable,
                            None,
                        );
                    }
                }

                if let Some(expr) = expr {
                    this.visit(expr);
                }
            }

            for (param, _, _) in parameters {
                if let Parameter::Pattern(_, pat) | Parameter::SpreadPattern(_, pat) = param {
                    this.visit_pattern(VariableDeclarationKind::Var, pat);
                }
            }

            for stmt in statements {
                this.visit_statement(stmt);
            }
        });

        None
    }

    pub fn visit_array_expression(&mut self, ArrayLiteral(expr): &ArrayLiteral) -> Option<CompileValueType> {
        for kind in expr {
            match kind {
                ArrayMemberKind::Spread(expr) => {
                    self.visit(expr);
                }
                ArrayMemberKind::Item(expr) => {
                    self.visit(expr);
                }
                ArrayMemberKind::Empty => {}
            }
        }
        Some(CompileValueType::Array)
    }

    pub fn visit_object_expression(&mut self, ObjectLiteral(expr): &ObjectLiteral) -> Option<CompileValueType> {
        for (kind, expr) in expr {
            if let ObjectMemberKind::Dynamic(expr) = kind {
                self.visit(expr);
            }
            self.visit(expr);
        }
        None
    }
}

pub struct NameResolutionResults {
    pub scopes: ScopeGraph,
    pub decl_to_slot: LocalDeclToSlot,
}

pub fn name_res(ast: &[Statement], scope_count: usize, local_count: usize) -> NameResolutionResults {
    let mut scopes = ScopeGraph::new(scope_count);
    let mut locals = LocalDeclToSlot(vec![0; local_count]);
    let mut tcx = TypeInferCtx::new(InferMode::Discover(&mut scopes, &mut locals));
    tcx.visit_many_statements(ast);
    NameResolutionResults {
        scopes,
        decl_to_slot: locals,
    }
}
