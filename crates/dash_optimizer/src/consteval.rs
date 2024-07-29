use dash_log::debug;
use dash_middle::compiler::scope::ScopeGraph;
use dash_middle::interner::StringInterner;
use dash_middle::lexer::token::TokenType;
use dash_middle::parser::expr::{
    ArrayLiteral, ArrayMemberKind, AssignmentExpr, AssignmentTarget, BinaryExpr, CallArgumentKind, ConditionalExpr,
    Expr, ExprKind, FunctionCall, GroupingExpr, LiteralExpr, ObjectLiteral, ObjectMemberKind, PropertyAccessExpr,
    UnaryExpr,
};
use dash_middle::parser::statement::{
    BlockStatement, Class, ClassMemberValue, DoWhileLoop, ExportKind, ForInLoop, ForLoop, ForOfLoop,
    FunctionDeclaration, IfStatement, ImportKind, Loop, ReturnStatement, ScopeId, SpecifierKind, Statement,
    StatementKind, SwitchCase, SwitchStatement, TryCatch, VariableBinding, VariableDeclaration, VariableDeclarations,
    WhileLoop,
};

use crate::OptLevel;

#[derive(Debug)]
pub struct ConstFunctionEvalCtx<'b, 'interner> {
    _view: &'b ScopeGraph,
    current: ScopeId,
    interner: &'interner mut StringInterner,
    #[allow(unused)]
    opt_level: OptLevel,
}

impl<'b, 'interner> ConstFunctionEvalCtx<'b, 'interner> {
    pub fn new(view: &'b ScopeGraph, interner: &'interner mut StringInterner, opt_level: OptLevel) -> Self {
        Self {
            _view: view,
            current: ScopeId::ROOT,
            interner,
            opt_level,
        }
    }

    fn with_scope(&mut self, id: ScopeId, f: impl FnOnce(&mut Self)) {
        let old = self.current;
        self.current = id;
        f(self);
        self.current = old;
    }

    pub fn visit_statement(&mut self, statement: &mut Statement) {
        match &mut statement.kind {
            StatementKind::Block(block) => self.visit_block_statement(block),
            StatementKind::Expression(expr) => {
                self.visit(expr);
            }
            StatementKind::Variable(stmt) => self.visit_variable_declaration(stmt),
            StatementKind::If(stmt) => self.visit_if_statement(stmt),
            StatementKind::Function(expr) => {
                self.visit_function_expression(expr);
            }
            StatementKind::Loop(expr) => self.visit_loop_statement(expr),
            StatementKind::Return(stmt) => self.visit_return_statement(stmt),
            StatementKind::Try(stmt) => self.visit_try_statement(stmt),
            StatementKind::Throw(expr) => {
                self.visit(expr);
            }
            StatementKind::Import(ImportKind::AllAs(SpecifierKind::Ident(..), ..)) => {}
            StatementKind::Import(ImportKind::Dynamic(expr)) => {
                self.visit(expr);
            }
            StatementKind::Import(ImportKind::DefaultAs(SpecifierKind::Ident(..), ..)) => {}
            StatementKind::Export(ExportKind::Default(expr)) => {
                self.visit(expr);
            }
            StatementKind::Export(ExportKind::Named(..)) => {}
            StatementKind::Export(ExportKind::NamedVar(stmt)) => self.visit_variable_declaration(stmt),
            StatementKind::Class(stmt) => self.visit_class_statement(stmt),
            StatementKind::Switch(stmt) => self.visit_switch_statement(stmt),
            StatementKind::Continue(_) => {}
            StatementKind::Break(_) => {}
            StatementKind::Debugger => {}
            StatementKind::Empty => {}
            StatementKind::Labelled(_, stmt) => self.visit_statement(stmt),
        };

        if !stmt_has_side_effects(statement) {
            *statement = Statement::dummy_empty();
        }
    }

    pub fn visit_maybe_statement(&mut self, stmt: Option<&mut Statement>) {
        if let Some(stmt) = stmt {
            self.visit_statement(stmt);
        }
    }

    pub fn visit_many_statements(&mut self, stmt: &mut [Statement]) {
        for stmt in stmt {
            self.visit_statement(stmt);
        }
    }

    pub fn visit_many_exprs(&mut self, expr: &mut [Expr]) {
        for expr in expr {
            self.visit(expr);
        }
    }

    pub fn visit_maybe_expr(&mut self, expr: Option<&mut Expr>) {
        if let Some(expr) = expr {
            self.visit(expr);
        }
    }

    pub fn visit_return_statement(&mut self, ReturnStatement(expr): &mut ReturnStatement) {
        self.visit(expr);
    }

    pub fn visit_block_statement(&mut self, BlockStatement(stmt, id): &mut BlockStatement) {
        self.with_scope(*id, |this| this.visit_many_statements(stmt));
    }

    pub fn visit_try_statement(&mut self, TryCatch { try_, catch, finally }: &mut TryCatch) {
        self.visit_statement(try_);
        if let Some(catch) = catch {
            self.visit_block_statement(&mut catch.body);
        }
        self.visit_maybe_statement(finally.as_deref_mut());
    }

    pub fn visit_class_statement(&mut self, Class { extends, members, .. }: &mut Class) {
        self.visit_maybe_expr(extends.as_deref_mut());
        for member in members {
            match &mut member.value {
                ClassMemberValue::Method(method)
                | ClassMemberValue::Getter(method)
                | ClassMemberValue::Setter(method) => {
                    self.visit_function_expression(method);
                }
                ClassMemberValue::Field(field) => {
                    self.visit_maybe_expr(field.as_mut());
                }
            }
        }
    }

    pub fn visit_switch_statement(&mut self, SwitchStatement { expr, default, cases }: &mut SwitchStatement) {
        self.visit(expr);

        if let Some(default) = default {
            self.visit_many_statements(default);
        }

        for SwitchCase { value, body } in cases {
            self.visit(value);
            self.visit_many_statements(body);
        }
    }

    pub fn visit_loop_statement(&mut self, loop_: &mut Loop) {
        match loop_ {
            Loop::For(ForLoop {
                init,
                condition,
                finalizer,
                body,
                scope,
            }) => {
                self.with_scope(*scope, |this| {
                    this.visit_maybe_statement(init.as_deref_mut());
                    this.visit_maybe_expr(condition.as_mut());
                    this.visit_maybe_expr(finalizer.as_mut());
                    this.visit_statement(body);
                });
            }
            Loop::ForOf(ForOfLoop {
                expr,
                body,
                binding,
                scope,
            }) => {
                self.with_scope(*scope, |this| {
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
                self.with_scope(*scope, |this| {
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

    fn visit_variable_binding(&mut self, _binding: &VariableBinding, value: Option<&mut Expr>) {
        if let Some(value) = value {
            self.visit(value);
        }
    }

    pub fn visit_variable_declaration(&mut self, VariableDeclarations(declarations): &mut VariableDeclarations) {
        for VariableDeclaration { binding, value } in declarations {
            self.visit_variable_binding(binding, value.as_mut());
        }
    }

    pub fn visit_if_statement(
        &mut self,
        IfStatement {
            condition,
            then,
            branches,
            el,
        }: &mut IfStatement,
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

    pub fn visit(&mut self, expression: &mut Expr) {
        match &mut expression.kind {
            ExprKind::Binary(..) => self.visit_binary_expression(expression),
            ExprKind::Grouping(GroupingExpr(expr)) => expr.iter_mut().for_each(|e| self.visit(e)),
            ExprKind::Literal(..) => {}
            ExprKind::Unary(..) => self.visit_unary_expression(expression),
            ExprKind::Assignment(..) => self.visit_assignment_expression(expression),
            ExprKind::Call(..) => self.visit_call_expression(expression),
            ExprKind::Conditional(..) => self.visit_conditional_expression(expression),
            ExprKind::PropertyAccess(..) => self.visit_property_access_expression(expression),
            ExprKind::Sequence(..) => self.visit_seq_expression(expression),
            ExprKind::Prefix(..) => self.visit_prefix_expression(expression),
            ExprKind::Postfix(..) => self.visit_postfix_expression(expression),
            ExprKind::Function(expr) => self.visit_function_expression(expr),
            ExprKind::Class(class) => self.visit_class_statement(class),
            ExprKind::Array(..) => self.visit_array_expression(expression),
            ExprKind::Object(..) => self.visit_object_expression(expression),
            ExprKind::Compiled(..) => {}
            ExprKind::Empty => {}
        }
    }

    fn visit_array_expression(&mut self, array_expr: &mut Expr) {
        let ExprKind::Array(ArrayLiteral(array)) = &mut array_expr.kind else {
            unreachable!()
        };

        for expr in array {
            match expr {
                ArrayMemberKind::Spread(expr) => self.visit(expr),
                ArrayMemberKind::Item(expr) => self.visit(expr),
                ArrayMemberKind::Empty => {}
            }
        }
    }

    fn visit_object_expression(&mut self, object_expr: &mut Expr) {
        let ExprKind::Object(ObjectLiteral(object)) = &mut object_expr.kind else {
            unreachable!()
        };

        for (kind, expr) in object {
            if let ObjectMemberKind::Dynamic(expr) = kind {
                self.visit(expr);
            }
            self.visit(expr);
        }
    }

    fn visit_postfix_expression(&mut self, postfix_expr: &mut Expr) {
        let ExprKind::Postfix((_, expr)) = &mut postfix_expr.kind else {
            unreachable!()
        };

        self.visit(expr);
    }

    fn visit_prefix_expression(&mut self, prefix_expr: &mut Expr) {
        let ExprKind::Prefix((_, expr)) = &mut prefix_expr.kind else {
            unreachable!()
        };

        self.visit(expr);
    }

    fn visit_seq_expression(&mut self, seq_expr: &mut Expr) {
        let ExprKind::Sequence((left, right)) = &mut seq_expr.kind else {
            unreachable!()
        };

        self.visit(left);
        self.visit(right);
    }

    fn visit_property_access_expression(&mut self, property_access_expr: &mut Expr) {
        let ExprKind::PropertyAccess(PropertyAccessExpr { target, property, .. }) = &mut property_access_expr.kind
        else {
            unreachable!()
        };

        self.visit(target);
        self.visit(property);
    }

    fn visit_conditional_expression(&mut self, conditional_expr: &mut Expr) {
        let ExprKind::Conditional(ConditionalExpr { condition, then, el }) = &mut conditional_expr.kind else {
            unreachable!()
        };
        debug!("reduce conditional {:?}", condition);
        self.visit(condition);
        self.visit(then);
        self.visit(el);

        use ExprKind::Literal;
        use LiteralExpr::Boolean;

        match &condition.kind {
            Literal(Boolean(true)) => {
                debug!("reduced condition to true");
                *conditional_expr = (**then).clone();
            }
            Literal(Boolean(false)) => {
                debug!("reduced condition to false");
                *conditional_expr = (**el).clone()
            }
            _ => {}
        }
    }

    fn visit_call_expression(&mut self, call_expr: &mut Expr) {
        let ExprKind::Call(FunctionCall { target, arguments, .. }) = &mut call_expr.kind else {
            unreachable!()
        };

        self.visit(target);
        for expr in arguments {
            match expr {
                CallArgumentKind::Normal(expr) => self.visit(expr),
                CallArgumentKind::Spread(expr) => self.visit(expr),
            }
        }
    }

    fn visit_assignment_expression(&mut self, assignment_expr: &mut Expr) {
        let ExprKind::Assignment(AssignmentExpr { left, right, .. }) = &mut assignment_expr.kind else {
            unreachable!()
        };

        if let AssignmentTarget::Expr(left) = left {
            self.visit(left);
        }
        self.visit(right);
    }

    fn visit_unary_expression(&mut self, unary_expr: &mut Expr) {
        let ExprKind::Unary(UnaryExpr { operator, expr }) = &mut unary_expr.kind else {
            unreachable!()
        };

        self.visit(expr);

        use ExprKind::*;
        use LiteralExpr::*;
        use TokenType::*;

        match (operator, &expr.kind) {
            (Minus, &Literal(Number(n))) => unary_expr.kind = Literal(Number(-n)),
            (Plus, &Literal(Number(n))) => unary_expr.kind = Literal(Number(n)),
            _ => {}
        }
    }

    fn visit_binary_expression(&mut self, binary_expr: &mut Expr) {
        let ExprKind::Binary(BinaryExpr { left, right, operator }) = &mut binary_expr.kind else {
            unreachable!()
        };
        debug!("reduce binary: {:?} {:?}", left, right);
        self.visit(left);
        self.visit(right);
        debug!("reduced binary to: {:?} {:?}", left, right);

        use ExprKind::*;
        use LiteralExpr::*;
        use TokenType::*;

        macro_rules! f64_opt {
            ($left:ident $t:tt $right:ident) => {{
                binary_expr.kind = Literal(Number($left $t $right));
            }};
        }
        macro_rules! float_opt_to_bool {
            ($left:ident $t:tt $right:ident) => {{
                binary_expr.kind = Literal(Boolean($left $t $right));
            }};
        }

        macro_rules! float_fopt {
            ($fun:expr, $left:ident, $right:ident) => {{
                binary_expr.kind = Literal(Number($fun($left, $right)));
            }};
        }

        macro_rules! i64_op {
            ($left:ident $t:tt $right:ident) => {
                binary_expr.kind = Literal(Number((($left as i64 as i32) $t ($right as i64 as i32)) as f64))
            };
        }

        fn truthy_f64(n: f64) -> bool {
            !n.is_nan() && n != 0.0
        }

        match (&left.kind, &right.kind, operator) {
            (&Literal(Number(left)), &Literal(Number(right)), Plus) => f64_opt!(left + right),
            (&Literal(Number(left)), &Literal(Number(right)), Minus) => f64_opt!(left - right),
            (&Literal(Number(left)), &Literal(Number(right)), Star) => f64_opt!(left * right),
            (&Literal(Number(left)), &Literal(Number(right)), Slash) => f64_opt!(left / right),
            (&Literal(Number(left)), &Literal(Number(right)), Remainder) => f64_opt!(left % right),
            (&Literal(Number(left)), &Literal(Number(right)), Exponentiation) => float_fopt!(f64::powf, left, right),
            (&Literal(Number(left)), &Literal(Number(right)), Greater) => float_opt_to_bool!(left > right),
            (&Literal(Number(left)), &Literal(Number(right)), GreaterEqual) => float_opt_to_bool!(left >= right),
            (&Literal(Number(left)), &Literal(Number(right)), Less) => float_opt_to_bool!(left < right),
            (&Literal(Number(left)), &Literal(Number(right)), LessEqual) => float_opt_to_bool!(left <= right),
            (&Literal(Number(left)), &Literal(Number(right)), Equality) => float_opt_to_bool!(left == right),
            (&Literal(Number(left)), &Literal(Number(right)), Inequality) => float_opt_to_bool!(left != right),
            (&Literal(Number(left)), &Literal(Number(right)), StrictEquality) => float_opt_to_bool!(left == right),
            (&Literal(Number(left)), &Literal(Number(right)), StrictInequality) => float_opt_to_bool!(left != right),
            (&Literal(Number(left)), &Literal(Number(right)), BitwiseOr) => i64_op!(left | right),
            (&Literal(Number(left)), &Literal(Number(right)), BitwiseAnd) => i64_op!(left & right),
            (&Literal(Number(left)), &Literal(Number(right)), BitwiseXor) => i64_op!(left ^ right),
            (&Literal(Number(left)), &Literal(Number(right)), LeftShift) => i64_op!(left << right),
            (&Literal(Number(left)), &Literal(Number(right)), RightShift) => i64_op!(left >> right),
            (&Literal(Number(left)), &Literal(Number(right)), LogicalOr) => {
                binary_expr.kind = Literal(Number(match truthy_f64(left) {
                    true => left,
                    false => right,
                }))
            }
            (&Literal(Number(left)), &Literal(Number(right)), LogicalAnd) => {
                binary_expr.kind = Literal(Number(match truthy_f64(left) {
                    true => right,
                    false => left,
                }))
            }
            (&Literal(LiteralExpr::String(left)), &Literal(LiteralExpr::String(right)), Equality) => {
                binary_expr.kind = Literal(Boolean(left == right));
            }
            (&Literal(LiteralExpr::String(left)), &Literal(LiteralExpr::String(right)), Inequality) => {
                binary_expr.kind = Literal(Boolean(left != right));
            }
            (&Literal(LiteralExpr::String(left)), &Literal(LiteralExpr::String(right)), Plus) => {
                let mut left = self.interner.resolve(left).to_string();
                left += self.interner.resolve(right);
                binary_expr.kind = Literal(LiteralExpr::String(self.interner.intern(left)));
            }
            _ => {}
        }
    }

    pub fn visit_function_expression(
        &mut self,
        FunctionDeclaration {
            parameters,
            statements,
            id,
            ..
        }: &mut FunctionDeclaration,
    ) {
        self.with_scope(*id, |this| {
            for (_, expr, _) in parameters {
                if let Some(expr) = expr {
                    this.visit(expr);
                }
            }

            for stmt in statements {
                this.visit_statement(stmt);
            }
        });
    }
}

fn stmt_has_side_effects(stmt: &Statement) -> bool {
    match &stmt.kind {
        StatementKind::Block(BlockStatement(block, _)) => block.iter().any(stmt_has_side_effects),
        StatementKind::Break(_) => true,
        StatementKind::Class(Class { .. }) => true, // TODO: can possibly be SE-free
        StatementKind::Empty => false,
        StatementKind::Expression(expr) => expr_has_side_effects(expr),
        StatementKind::Function(FunctionDeclaration { name, .. }) => {
            // Only considered to have side-effects if it's an actual declaration
            name.is_some()
        }
        StatementKind::If(IfStatement { .. }) => true,
        _ => true,
    }
}

fn expr_has_side_effects(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Array(ArrayLiteral(array)) => array.iter().any(|k| match k {
            ArrayMemberKind::Item(e) => expr_has_side_effects(e),
            ArrayMemberKind::Spread(e) => expr_has_side_effects(e),
            ArrayMemberKind::Empty => false,
        }),
        ExprKind::Binary(BinaryExpr { left, right, .. }) => expr_has_side_effects(left) || expr_has_side_effects(right),
        ExprKind::Conditional(ConditionalExpr { condition, then, el }) => {
            expr_has_side_effects(condition) || expr_has_side_effects(then) || expr_has_side_effects(el)
        }
        ExprKind::Empty => false,
        ExprKind::Function(..) => false,
        ExprKind::Grouping(GroupingExpr(grouping)) => grouping.iter().any(expr_has_side_effects),
        ExprKind::Literal(LiteralExpr::Boolean(..)) => false,
        ExprKind::Literal(LiteralExpr::Identifier(..)) => true, // might invoke a global getter
        ExprKind::Literal(LiteralExpr::Null) => false,
        ExprKind::Literal(LiteralExpr::Undefined) => false,
        ExprKind::Literal(LiteralExpr::Number(..)) => false,
        ExprKind::Literal(LiteralExpr::Regex(..)) => false,
        ExprKind::Literal(LiteralExpr::String(..)) => false,
        ExprKind::Object(ObjectLiteral(object)) => object.iter().any(|(kind, expr)| {
            if let ObjectMemberKind::Dynamic(dynamic) = kind {
                if expr_has_side_effects(dynamic) {
                    return true;
                }
            };
            expr_has_side_effects(expr)
        }),
        ExprKind::Postfix((_, expr)) => expr_has_side_effects(expr),
        ExprKind::Prefix((_, expr)) => expr_has_side_effects(expr),
        ExprKind::PropertyAccess(PropertyAccessExpr { target, property, .. }) => {
            expr_has_side_effects(target) || expr_has_side_effects(property)
        }
        ExprKind::Sequence((left, right)) => expr_has_side_effects(left) || expr_has_side_effects(right),
        ExprKind::Unary(UnaryExpr { .. }) => true, // TODO: can special case +- literal
        _ => true,
    }
}
