use dash_log::debug;
use dash_log::error;
use dash_middle::compiler::scope::CompileValueType;
use dash_middle::compiler::scope::Scope;
use dash_middle::compiler::scope::ScopeLocal;
use dash_middle::lexer::token::TokenType;
use dash_middle::parser::expr::ArrayLiteral;
use dash_middle::parser::expr::AssignmentExpr;
use dash_middle::parser::expr::AssignmentTarget;
use dash_middle::parser::expr::BinaryExpr;
use dash_middle::parser::expr::ConditionalExpr;
use dash_middle::parser::expr::Expr;
use dash_middle::parser::expr::FunctionCall;
use dash_middle::parser::expr::GroupingExpr;
use dash_middle::parser::expr::LiteralExpr;
use dash_middle::parser::expr::ObjectLiteral;
use dash_middle::parser::expr::ObjectMemberKind;
use dash_middle::parser::expr::PropertyAccessExpr;
use dash_middle::parser::expr::UnaryExpr;
use dash_middle::parser::statement::BlockStatement;
use dash_middle::parser::statement::Class;
use dash_middle::parser::statement::ClassMemberKind;
use dash_middle::parser::statement::ClassProperty;
use dash_middle::parser::statement::DoWhileLoop;
use dash_middle::parser::statement::ExportKind;
use dash_middle::parser::statement::ForInLoop;
use dash_middle::parser::statement::ForLoop;
use dash_middle::parser::statement::ForOfLoop;
use dash_middle::parser::statement::FuncId;
use dash_middle::parser::statement::FunctionDeclaration;
use dash_middle::parser::statement::IfStatement;
use dash_middle::parser::statement::ImportKind;
use dash_middle::parser::statement::Loop;
use dash_middle::parser::statement::Parameter;
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::SpecifierKind;
use dash_middle::parser::statement::Statement;
use dash_middle::parser::statement::SwitchCase;
use dash_middle::parser::statement::SwitchStatement;
use dash_middle::parser::statement::TryCatch;
use dash_middle::parser::statement::VariableBinding;
use dash_middle::parser::statement::VariableDeclaration;
use dash_middle::parser::statement::VariableDeclarationKind;
use dash_middle::parser::statement::VariableDeclarationName;
use dash_middle::parser::statement::VariableDeclarations;
use dash_middle::parser::statement::WhileLoop;
use dash_middle::tree::Tree;
use dash_middle::tree::TreeNode;
use dash_middle::util::Counter;

#[derive(Debug)]
pub struct TypeInferCtx<'a> {
    counter: Counter<FuncId>,
    scopes: Tree<Scope<'a>>,
}

impl<'a> TypeInferCtx<'a> {
    pub fn new(counter: Counter<FuncId>) -> Self {
        let scopes = (0..counter.len()).map(|_| TreeNode::new(Scope::new(), None)).collect();
        Self { scopes, counter }
    }

    pub fn scope_mut(&mut self, func_id: FuncId) -> &mut Scope<'a> {
        // Scope not found implies a programmer error
        &mut self.scopes[func_id.into()]
    }

    pub fn scope(&self, func_id: FuncId) -> &Scope<'a> {
        // Scope not found implies a programmer error
        &self.scopes[func_id.into()]
    }

    pub fn scope_node(&self, func_id: FuncId) -> &TreeNode<Scope<'a>> {
        // Scope not found implies a programmer error
        &self.scopes[func_id.into()]
    }

    pub fn scope_node_mut(&mut self, func_id: FuncId) -> &mut TreeNode<Scope<'a>> {
        // Scope not found implies a programmer error
        &mut self.scopes[func_id.into()]
    }

    pub fn counter_mut(&mut self) -> &mut Counter<FuncId> {
        &mut self.counter
    }

    pub fn add_scope(&mut self, parent: Option<FuncId>) -> FuncId {
        self.scopes.push(parent.map(Into::into), Scope::new()).into()
    }

    pub fn visit_statement(&mut self, statement: &Statement<'a>, func_id: FuncId) {
        match statement {
            Statement::Block(BlockStatement(stmt)) => {
                self.scope_mut(func_id).enter();
                for stmt in stmt {
                    self.visit_statement(stmt, func_id);
                }
                self.scope_mut(func_id).exit();
            }
            Statement::Expression(expr) => drop(self.visit(expr, func_id)),
            Statement::Variable(stmt) => self.visit_variable_declaration(stmt, func_id),
            Statement::If(stmt) => self.visit_if_statement(stmt, func_id),
            Statement::Function(expr) => drop(self.visit_function_expression(expr, func_id)),
            Statement::Loop(expr) => self.visit_loop_statement(expr, func_id),
            Statement::Return(stmt) => self.visit_return_statement(stmt, func_id),
            Statement::Try(stmt) => self.visit_try_statement(stmt, func_id),
            Statement::Throw(expr) => drop(self.visit(expr, func_id)),
            Statement::Import(ImportKind::AllAs(SpecifierKind::Ident(..), ..)) => {}
            Statement::Import(ImportKind::Dynamic(expr)) => drop(self.visit(expr, func_id)),
            Statement::Import(ImportKind::DefaultAs(SpecifierKind::Ident(..), ..)) => {}
            Statement::Export(ExportKind::Default(expr)) => drop(self.visit(expr, func_id)),
            Statement::Export(ExportKind::Named(..)) => {}
            Statement::Export(ExportKind::NamedVar(stmt)) => self.visit_variable_declaration(stmt, func_id),
            Statement::Class(stmt) => self.visit_class_statement(stmt, func_id),
            Statement::Switch(stmt) => self.visit_switch_statement(stmt, func_id),
            Statement::Continue => {}
            Statement::Break => {}
            Statement::Debugger => {}
            Statement::Empty => {}
        }
    }

    pub fn visit_maybe_statement(&mut self, stmt: Option<&Statement<'a>>, func_id: FuncId) {
        if let Some(stmt) = stmt {
            self.visit_statement(stmt, func_id);
        }
    }

    pub fn visit_many_statements(&mut self, stmt: &[Statement<'a>], func_id: FuncId) {
        for stmt in stmt {
            self.visit_statement(stmt, func_id);
        }
    }

    pub fn visit_maybe_expr(&mut self, expr: Option<&Expr<'a>>, func_id: FuncId) -> Option<CompileValueType> {
        if let Some(expr) = expr {
            self.visit(expr, func_id)
        } else {
            None
        }
    }

    pub fn visit_return_statement(&mut self, ReturnStatement(expr): &ReturnStatement<'a>, func_id: FuncId) {
        self.visit(expr, func_id);
    }

    pub fn visit_try_statement(&mut self, TryCatch { try_, catch, finally }: &TryCatch<'a>, func_id: FuncId) {
        self.visit_statement(try_, func_id);
        self.visit_statement(&catch.body, func_id);
        self.visit_maybe_statement(finally.as_deref(), func_id);
    }

    pub fn visit_class_statement(
        &mut self,
        Class {
            extends, members, name, ..
        }: &Class<'a>,
        func_id: FuncId,
    ) {
        self.visit_maybe_expr(extends.as_ref(), func_id);
        for member in members {
            match &member.kind {
                ClassMemberKind::Method(method) => drop(self.visit_function_expression(method, func_id)),
                ClassMemberKind::Property(ClassProperty { value, .. }) => {
                    drop(self.visit_maybe_expr(value.as_ref(), func_id))
                }
            }
        }

        if let Some(name) = name {
            self.visit_variable_binding(
                &VariableBinding {
                    name: VariableDeclarationName::Identifier(name),
                    kind: VariableDeclarationKind::Var,
                    ty: None,
                },
                None,
                func_id,
            )
        }
    }

    pub fn visit_switch_statement(
        &mut self,
        SwitchStatement { expr, default, cases }: &SwitchStatement<'a>,
        func_id: FuncId,
    ) {
        self.visit(expr, func_id);

        if let Some(default) = default {
            self.visit_many_statements(default, func_id);
        }

        for SwitchCase { value, body } in cases {
            self.visit(value, func_id);
            self.visit_many_statements(body, func_id);
        }
    }

    pub fn visit_loop_statement(&mut self, loop_: &Loop<'a>, func_id: FuncId) {
        match loop_ {
            Loop::For(ForLoop {
                init,
                condition,
                finalizer,
                body,
            }) => {
                self.visit_maybe_statement(init.as_deref(), func_id);
                self.visit_maybe_expr(condition.as_ref(), func_id);
                self.visit_maybe_expr(finalizer.as_ref(), func_id);
                self.visit_statement(body, func_id);
            }
            Loop::ForOf(ForOfLoop { expr, body, binding }) => {
                self.visit_variable_binding(binding, None, func_id);
                self.visit(expr, func_id);
                self.visit_statement(body, func_id);
            }
            Loop::ForIn(ForInLoop { expr, body, binding }) => {
                self.visit_variable_binding(binding, None, func_id);
                self.visit(expr, func_id);
                self.visit_statement(body, func_id);
            }
            Loop::While(WhileLoop { condition, body }) => {
                self.visit(condition, func_id);
                self.visit_statement(body, func_id);
            }
            Loop::DoWhile(DoWhileLoop { body, condition }) => {
                self.visit(condition, func_id);
                self.visit_statement(body, func_id);
            }
        }
    }

    fn visit_variable_binding(&mut self, binding: &VariableBinding<'a>, value: Option<&Expr<'a>>, func_id: FuncId) {
        if let VariableDeclarationName::Identifier(ident) = binding.name {
            let ty = match value {
                Some(expr) => self.visit(expr, func_id),
                None => Some(CompileValueType::Uninit),
            };

            debug!("discovered new variable {} of type {:?}", ident, ty);

            if self.scope_mut(func_id).add_local(ident, binding.kind, ty).is_err() {
                error!("failed to add variable");
            }
        }
    }

    pub fn visit_variable_declaration(
        &mut self,
        VariableDeclarations(declarations): &VariableDeclarations<'a>,
        func_id: FuncId,
    ) {
        for VariableDeclaration { binding, value } in declarations {
            self.visit_variable_binding(binding, value.as_ref(), func_id);
        }
    }

    pub fn visit_if_statement(
        &mut self,
        IfStatement {
            condition,
            then,
            branches,
            el,
        }: &IfStatement<'a>,
        func_id: FuncId,
    ) {
        self.visit(condition, func_id);
        self.visit_statement(then, func_id);
        if let Some(el) = el {
            self.visit_statement(el, func_id);
        }
        let branches = branches.borrow();
        for branch in branches.iter() {
            self.visit_if_statement(branch, func_id);
        }
    }

    pub fn visit(&mut self, expression: &Expr<'a>, func_id: FuncId) -> Option<CompileValueType> {
        match expression {
            Expr::Binary(expr) => self.visit_binary_expression(expr, func_id),
            Expr::Grouping(expr) => self.visit_grouping_expression(expr, func_id),
            Expr::Literal(expr) => self.visit_literal_expression(expr, func_id),
            Expr::Unary(expr) => self.visit_unary_expression(expr, func_id),
            Expr::Assignment(expr) => self.visit_assignment_expression(expr, func_id),
            Expr::Call(expr) => self.visit_call_expression(expr, func_id),
            Expr::Conditional(expr) => self.visit_conditional_expression(expr, func_id),
            Expr::PropertyAccess(expr) => self.visit_property_access_expression(expr, func_id),
            Expr::Sequence(..) => panic!("Unemitted expr type: Sequence"),
            Expr::Prefix((tt, expr)) => self.visit_prefix_expression(expr, *tt, func_id),
            Expr::Postfix((tt, expr)) => self.visit_postfix_expression(expr, *tt, func_id),
            Expr::Function(expr) => self.visit_function_expression(expr, func_id),
            Expr::Array(expr) => self.visit_array_expression(expr, func_id),
            Expr::Object(expr) => self.visit_object_expression(expr, func_id),
            Expr::Compiled(..) => None,
            Expr::Empty => None,
        }
    }

    pub fn visit_binary_expression(
        &mut self,
        BinaryExpr { left, right, operator }: &BinaryExpr<'a>,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        let left = self.visit(left, func_id);
        let right = self.visit(right, func_id);

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

    pub fn visit_grouping_expression(
        &mut self,
        GroupingExpr(expression): &GroupingExpr<'a>,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        let mut ty = None;
        for expression in expression {
            ty = self.visit(expression, func_id);
        }
        ty
    }

    pub fn find_local(&self, ident: &str, func_id: FuncId) -> Option<&ScopeLocal<'a>> {
        if let Some((_, local)) = self.scope(func_id).find_local(ident) {
            Some(local)
        } else {
            let parent = self.scope_node(func_id).parent()?;
            let local = self.find_local(ident, parent.into())?;
            local.infer(CompileValueType::Extern);
            Some(local)
        }
    }

    pub fn visit_literal_expression(
        &mut self,
        expression: &LiteralExpr<'a>,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        match expression {
            LiteralExpr::Boolean(..) => Some(CompileValueType::Boolean),
            LiteralExpr::Identifier(identifier) => match self.find_local(identifier, func_id) {
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

    pub fn visit_unary_expression(
        &mut self,
        UnaryExpr { expr, operator }: &UnaryExpr<'a>,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        self.visit(expr, func_id);
        match operator {
            TokenType::Plus | TokenType::Minus => Some(CompileValueType::Number),
            TokenType::Typeof => Some(CompileValueType::String),
            _ => None,
        }
    }

    pub fn visit_assignment_expression(
        &mut self,
        AssignmentExpr { left, right, .. }: &AssignmentExpr<'a>,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        let AssignmentTarget::Expr(left) = left else {
            panic!("Cannot infer type for assignment place LocalId");
        };

        self.visit(left, func_id);
        let right_type = self.visit(right, func_id);

        // Also propagate assignment to target
        if let Expr::Literal(LiteralExpr::Identifier(ident)) = &**left {
            if let Some(local) = self.find_local(ident, func_id) {
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
                            *left_type.borrow_mut() = Some(CompileValueType::Either(Box::new(left), Box::new(right)));
                        }
                        (_, Some(right)) => {
                            drop(left_type_ref);
                            *left_type.borrow_mut() = Some(CompileValueType::Maybe(Box::new(right.clone())));
                        }
                        (_, _) => {
                            drop(left_type_ref);
                            *left_type.borrow_mut() = None;
                        }
                    }
                }
            }
        }

        right_type
    }

    pub fn visit_call_expression(
        &mut self,
        FunctionCall { target, arguments, .. }: &FunctionCall<'a>,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        self.visit(target, func_id);
        for argument in arguments {
            self.visit(argument, func_id);
        }
        None
    }

    pub fn visit_conditional_expression(
        &mut self,
        ConditionalExpr { then, el, condition }: &ConditionalExpr<'a>,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        self.visit(condition, func_id);
        let then_ty = self.visit(then, func_id);
        let else_ty = self.visit(el, func_id);

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
        PropertyAccessExpr { target, property, .. }: &PropertyAccessExpr<'a>,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        self.visit(target, func_id);
        self.visit(property, func_id);
        None
    }

    pub fn visit_prefix_expression(
        &mut self,
        expression: &Expr<'a>,
        _: TokenType,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        self.visit(expression, func_id);
        Some(CompileValueType::Number)
    }

    pub fn visit_postfix_expression(
        &mut self,
        expression: &Expr<'a>,
        _: TokenType,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        self.visit(expression, func_id);
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
        }: &FunctionDeclaration<'a>,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        let sub_func_id = *id;

        self.scope_node_mut(sub_func_id).set_parent(func_id.into());
        if let Some(name) = name {
            debug!("visit function {name}");

            if self
                .scope_mut(func_id)
                .add_local(name, VariableDeclarationKind::Var, None)
                .is_err()
            {
                error!("failed to reserve local space for function");
            }
        }

        for (param, expr, _) in parameters {
            match param {
                Parameter::Identifier(ident) | Parameter::Spread(ident) => {
                    if self
                        .scope_mut(sub_func_id)
                        .add_local(ident, VariableDeclarationKind::Var, None)
                        .is_err()
                    {
                        error!("failed to reserve space for parameter")
                    }
                }
            }

            if let Some(expr) = expr {
                self.visit(expr, sub_func_id);
            }
        }

        for stmt in statements {
            self.visit_statement(stmt, sub_func_id);
        }
        None
    }

    pub fn visit_array_expression(
        &mut self,
        ArrayLiteral(expr): &ArrayLiteral<'a>,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        for expr in expr {
            self.visit(expr, func_id);
        }
        Some(CompileValueType::Array)
    }

    pub fn visit_object_expression(
        &mut self,
        ObjectLiteral(expr): &ObjectLiteral<'a>,
        func_id: FuncId,
    ) -> Option<CompileValueType> {
        for (kind, expr) in expr {
            if let ObjectMemberKind::Dynamic(expr) = kind {
                self.visit(expr, func_id);
            }
            self.visit(expr, func_id);
        }
        None
    }
}
