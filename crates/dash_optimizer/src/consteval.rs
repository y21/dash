use dash_log::debug;
use dash_middle::interner::StringInterner;
use dash_middle::lexer::token::TokenType;
use dash_middle::parser::expr::{
    ArrayLiteral, ArrayMemberKind, AssignmentExpr, AssignmentTarget, BinaryExpr, CallArgumentKind, ConditionalExpr,
    Expr, ExprKind, FunctionCall, GroupingExpr, LiteralExpr, ObjectLiteral, ObjectMemberKind, PropertyAccessExpr,
    UnaryExpr,
};
use dash_middle::parser::statement::{
    BlockStatement, Class, ClassMemberKind, ClassProperty, DoWhileLoop, ExportKind, ForInLoop, ForLoop, ForOfLoop,
    FuncId, FunctionDeclaration, IfStatement, ImportKind, Loop, Parameter, ReturnStatement, SpecifierKind, Statement,
    StatementKind, SwitchCase, SwitchStatement, TryCatch, VariableBinding, VariableDeclaration,
    VariableDeclarationKind, VariableDeclarations, WhileLoop,
};

use crate::type_infer::TypeInferCtx;
use crate::OptLevel;

#[derive(Debug)]
pub struct ConstFunctionEvalCtx<'b, 'interner> {
    tcx: &'b mut TypeInferCtx,
    interner: &'interner mut StringInterner,
    #[allow(unused)]
    opt_level: OptLevel,
}

impl<'b, 'interner> ConstFunctionEvalCtx<'b, 'interner> {
    pub fn new(tcx: &'b mut TypeInferCtx, interner: &'interner mut StringInterner, opt_level: OptLevel) -> Self {
        Self {
            tcx,
            interner,
            opt_level,
        }
    }

    pub fn visit_statement(&mut self, statement: &mut Statement, func_id: FuncId) {
        match &mut statement.kind {
            StatementKind::Block(BlockStatement(stmt)) => {
                self.tcx.scope_mut(func_id).enter();
                for stmt in stmt {
                    self.visit_statement(stmt, func_id);
                }
                self.tcx.scope_mut(func_id).exit();
            }
            StatementKind::Expression(expr) => {
                self.visit(expr, func_id);
            }
            StatementKind::Variable(stmt) => self.visit_variable_declaration(stmt, func_id),
            StatementKind::If(stmt) => self.visit_if_statement(stmt, func_id),
            StatementKind::Function(expr) => {
                self.visit_function_expression(expr, func_id);
            }
            StatementKind::Loop(expr) => self.visit_loop_statement(expr, func_id),
            StatementKind::Return(stmt) => self.visit_return_statement(stmt, func_id),
            StatementKind::Try(stmt) => self.visit_try_statement(stmt, func_id),
            StatementKind::Throw(expr) => {
                self.visit(expr, func_id);
            }
            StatementKind::Import(ImportKind::AllAs(SpecifierKind::Ident(..), ..)) => {}
            StatementKind::Import(ImportKind::Dynamic(expr)) => {
                self.visit(expr, func_id);
            }
            StatementKind::Import(ImportKind::DefaultAs(SpecifierKind::Ident(..), ..)) => {}
            StatementKind::Export(ExportKind::Default(expr)) => {
                self.visit(expr, func_id);
            }
            StatementKind::Export(ExportKind::Named(..)) => {}
            StatementKind::Export(ExportKind::NamedVar(stmt)) => self.visit_variable_declaration(stmt, func_id),
            StatementKind::Class(stmt) => self.visit_class_statement(stmt, func_id),
            StatementKind::Switch(stmt) => self.visit_switch_statement(stmt, func_id),
            StatementKind::Continue => {}
            StatementKind::Break => {}
            StatementKind::Debugger => {}
            StatementKind::Empty => {}
        };

        if !stmt_has_side_effects(statement) {
            *statement = Statement::dummy_empty();
        }
    }

    pub fn visit_maybe_statement(&mut self, stmt: Option<&mut Statement>, func_id: FuncId) {
        if let Some(stmt) = stmt {
            self.visit_statement(stmt, func_id);
        }
    }

    pub fn visit_many_statements(&mut self, stmt: &mut [Statement], func_id: FuncId) {
        for stmt in stmt {
            self.visit_statement(stmt, func_id);
        }
    }

    pub fn visit_many_exprs(&mut self, expr: &mut [Expr], func_id: FuncId) {
        for expr in expr {
            self.visit(expr, func_id);
        }
    }

    pub fn visit_maybe_expr(&mut self, expr: Option<&mut Expr>, func_id: FuncId) {
        if let Some(expr) = expr {
            self.visit(expr, func_id);
        }
    }

    pub fn visit_return_statement(&mut self, ReturnStatement(expr): &mut ReturnStatement, func_id: FuncId) {
        self.visit(expr, func_id);
    }

    pub fn visit_try_statement(&mut self, TryCatch { try_, catch, finally }: &mut TryCatch, func_id: FuncId) {
        self.visit_statement(try_, func_id);
        self.visit_statement(&mut catch.body, func_id);
        self.visit_maybe_statement(finally.as_deref_mut(), func_id);
    }

    pub fn visit_class_statement(&mut self, Class { extends, members, .. }: &mut Class, func_id: FuncId) {
        self.visit_maybe_expr(extends.as_mut(), func_id);
        for member in members {
            match &mut member.kind {
                ClassMemberKind::Method(method) => {
                    self.visit_function_expression(method, func_id);
                }
                ClassMemberKind::Property(ClassProperty { value, .. }) => {
                    self.visit_maybe_expr(value.as_mut(), func_id);
                }
            }
        }
    }

    pub fn visit_switch_statement(
        &mut self,
        SwitchStatement { expr, default, cases }: &mut SwitchStatement,
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

    pub fn visit_loop_statement(&mut self, loop_: &mut Loop, func_id: FuncId) {
        match loop_ {
            Loop::For(ForLoop {
                init,
                condition,
                finalizer,
                body,
            }) => {
                self.visit_maybe_statement(init.as_deref_mut(), func_id);
                self.visit_maybe_expr(condition.as_mut(), func_id);
                self.visit_maybe_expr(finalizer.as_mut(), func_id);
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

    fn visit_variable_binding(&mut self, _binding: &VariableBinding, value: Option<&mut Expr>, func_id: FuncId) {
        if let Some(value) = value {
            self.visit(value, func_id);
        }
    }

    pub fn visit_variable_declaration(
        &mut self,
        VariableDeclarations(declarations): &mut VariableDeclarations,
        func_id: FuncId,
    ) {
        for VariableDeclaration { binding, value } in declarations {
            self.visit_variable_binding(binding, value.as_mut(), func_id);
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
        func_id: FuncId,
    ) {
        self.visit(condition, func_id);
        self.visit_statement(then, func_id);
        if let Some(el) = el {
            self.visit_statement(el, func_id);
        }
        for branch in branches {
            self.visit_if_statement(branch, func_id);
        }
    }

    pub fn visit(&mut self, expression: &mut Expr, func_id: FuncId) {
        match &mut expression.kind {
            ExprKind::Binary(..) => self.visit_binary_expression(expression, func_id),
            ExprKind::Grouping(GroupingExpr(expr)) => expr.iter_mut().for_each(|e| self.visit(e, func_id)),
            ExprKind::Literal(..) => {}
            ExprKind::Unary(..) => self.visit_unary_expression(expression, func_id),
            ExprKind::Assignment(..) => self.visit_assignment_expression(expression, func_id),
            ExprKind::Call(..) => self.visit_call_expression(expression, func_id),
            ExprKind::Conditional(..) => self.visit_conditional_expression(expression, func_id),
            ExprKind::PropertyAccess(..) => self.visit_property_access_expression(expression, func_id),
            ExprKind::Sequence(..) => self.visit_seq_expression(expression, func_id),
            ExprKind::Prefix(..) => self.visit_prefix_expression(expression, func_id),
            ExprKind::Postfix(..) => self.visit_postfix_expression(expression, func_id),
            ExprKind::Function(expr) => self.visit_function_expression(expr, func_id),
            ExprKind::Array(..) => self.visit_array_expression(expression, func_id),
            ExprKind::Object(..) => self.visit_object_expression(expression, func_id),
            ExprKind::Compiled(..) => {}
            ExprKind::Empty => {}
        }
    }

    fn visit_array_expression(&mut self, array_expr: &mut Expr, func_id: FuncId) {
        let ExprKind::Array(ArrayLiteral(array)) = &mut array_expr.kind else {
            unreachable!()
        };

        for expr in array {
            match expr {
                ArrayMemberKind::Spread(expr) => self.visit(expr, func_id),
                ArrayMemberKind::Item(expr) => self.visit(expr, func_id),
                ArrayMemberKind::Empty => {}
            }
        }
    }

    fn visit_object_expression(&mut self, object_expr: &mut Expr, func_id: FuncId) {
        let ExprKind::Object(ObjectLiteral(object)) = &mut object_expr.kind else {
            unreachable!()
        };

        for (kind, expr) in object {
            if let ObjectMemberKind::Dynamic(expr) = kind {
                self.visit(expr, func_id);
            }
            self.visit(expr, func_id);
        }
    }

    fn visit_postfix_expression(&mut self, postfix_expr: &mut Expr, func_id: FuncId) {
        let ExprKind::Postfix((_, expr)) = &mut postfix_expr.kind else {
            unreachable!()
        };

        self.visit(expr, func_id);
    }

    fn visit_prefix_expression(&mut self, prefix_expr: &mut Expr, func_id: FuncId) {
        let ExprKind::Prefix((_, expr)) = &mut prefix_expr.kind else {
            unreachable!()
        };

        self.visit(expr, func_id);
    }

    fn visit_seq_expression(&mut self, seq_expr: &mut Expr, func_id: FuncId) {
        let ExprKind::Sequence((left, right)) = &mut seq_expr.kind else {
            unreachable!()
        };

        self.visit(left, func_id);
        self.visit(right, func_id);
    }

    fn visit_property_access_expression(&mut self, property_access_expr: &mut Expr, func_id: FuncId) {
        let ExprKind::PropertyAccess(PropertyAccessExpr { target, property, .. }) = &mut property_access_expr.kind
        else {
            unreachable!()
        };

        self.visit(target, func_id);
        self.visit(property, func_id);
    }

    fn visit_conditional_expression(&mut self, conditional_expr: &mut Expr, func_id: FuncId) {
        let ExprKind::Conditional(ConditionalExpr { condition, then, el }) = &mut conditional_expr.kind else {
            unreachable!()
        };
        debug!("reduce conditional {:?}", condition);
        self.visit(condition, func_id);
        self.visit(then, func_id);
        self.visit(el, func_id);

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

    fn visit_call_expression(&mut self, call_expr: &mut Expr, func_id: FuncId) {
        let ExprKind::Call(FunctionCall { target, arguments, .. }) = &mut call_expr.kind else {
            unreachable!()
        };

        self.visit(target, func_id);
        for expr in arguments {
            match expr {
                CallArgumentKind::Normal(expr) => self.visit(expr, func_id),
                CallArgumentKind::Spread(expr) => self.visit(expr, func_id),
            }
        }
    }

    fn visit_assignment_expression(&mut self, assignment_expr: &mut Expr, func_id: FuncId) {
        let ExprKind::Assignment(AssignmentExpr { left, right, .. }) = &mut assignment_expr.kind else {
            unreachable!()
        };

        if let AssignmentTarget::Expr(left) = left {
            self.visit(left, func_id);
        }
        self.visit(right, func_id);
    }

    fn visit_unary_expression(&mut self, unary_expr: &mut Expr, func_id: FuncId) {
        let ExprKind::Unary(UnaryExpr { operator, expr }) = &mut unary_expr.kind else {
            unreachable!()
        };

        self.visit(expr, func_id);

        use ExprKind::*;
        use LiteralExpr::*;
        use TokenType::*;

        match (operator, &expr.kind) {
            (Minus, &Literal(Number(n))) => unary_expr.kind = Literal(Number(-n)),
            (Plus, &Literal(Number(n))) => unary_expr.kind = Literal(Number(n)),
            _ => {}
        }
    }

    fn visit_binary_expression(&mut self, binary_expr: &mut Expr, func_id: FuncId) {
        let ExprKind::Binary(BinaryExpr { left, right, operator }) = &mut binary_expr.kind else {
            unreachable!()
        };
        debug!("reduce binary: {:?} {:?}", left, right);
        self.visit(left, func_id);
        self.visit(right, func_id);
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
        _func_id: FuncId,
    ) {
        let sub_func_id = *id;

        for (param, expr, _) in parameters {
            match param {
                Parameter::Identifier(ident) | Parameter::Spread(ident) => {
                    // TODO: handle this error, somehow
                    let _ = self
                        .tcx
                        .scope_mut(sub_func_id)
                        .add_local(*ident, VariableDeclarationKind::Var, None);
                }
            }

            if let Some(expr) = expr {
                self.visit(expr, sub_func_id);
            }
        }

        for stmt in statements {
            self.visit_statement(stmt, sub_func_id);
        }
    }
}

fn stmt_has_side_effects(stmt: &Statement) -> bool {
    match &stmt.kind {
        StatementKind::Block(BlockStatement(block)) => block.iter().any(stmt_has_side_effects),
        StatementKind::Break => true,
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
