use dash_log::debug;
use dash_middle::lexer::token::TokenType;
use dash_middle::parser::expr::ArrayLiteral;
use dash_middle::parser::expr::ArrayMemberKind;
use dash_middle::parser::expr::AssignmentExpr;
use dash_middle::parser::expr::AssignmentTarget;
use dash_middle::parser::expr::BinaryExpr;
use dash_middle::parser::expr::CallArgumentKind;
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
use dash_middle::parser::statement::VariableDeclarations;
use dash_middle::parser::statement::WhileLoop;

use crate::type_infer::TypeInferCtx;
use crate::OptLevel;

#[derive(Debug)]
pub struct ConstFunctionEvalCtx<'a, 'b> {
    tcx: &'b mut TypeInferCtx<'a>,
    #[allow(unused)]
    opt_level: OptLevel,
}

impl<'a, 'b> ConstFunctionEvalCtx<'a, 'b> {
    pub fn new(tcx: &'b mut TypeInferCtx<'a>, opt_level: OptLevel) -> Self {
        Self { tcx, opt_level }
    }

    pub fn visit_statement(&mut self, statement: &mut Statement<'a>, func_id: FuncId) {
        match statement {
            Statement::Block(BlockStatement(stmt)) => {
                self.tcx.scope_mut(func_id).enter();
                for stmt in stmt {
                    self.visit_statement(stmt, func_id);
                }
                self.tcx.scope_mut(func_id).exit();
            }
            Statement::Expression(expr) => {
                self.visit(expr, func_id);
            }
            Statement::Variable(stmt) => self.visit_variable_declaration(stmt, func_id),
            Statement::If(stmt) => self.visit_if_statement(stmt, func_id),
            Statement::Function(expr) => {
                self.visit_function_expression(expr, func_id);
            }
            Statement::Loop(expr) => self.visit_loop_statement(expr, func_id),
            Statement::Return(stmt) => self.visit_return_statement(stmt, func_id),
            Statement::Try(stmt) => self.visit_try_statement(stmt, func_id),
            Statement::Throw(expr) => {
                self.visit(expr, func_id);
            }
            Statement::Import(ImportKind::AllAs(SpecifierKind::Ident(..), ..)) => {}
            Statement::Import(ImportKind::Dynamic(expr)) => {
                self.visit(expr, func_id);
            }
            Statement::Import(ImportKind::DefaultAs(SpecifierKind::Ident(..), ..)) => {}
            Statement::Export(ExportKind::Default(expr)) => {
                self.visit(expr, func_id);
            }
            Statement::Export(ExportKind::Named(..)) => {}
            Statement::Export(ExportKind::NamedVar(stmt)) => self.visit_variable_declaration(stmt, func_id),
            Statement::Class(stmt) => self.visit_class_statement(stmt, func_id),
            Statement::Switch(stmt) => self.visit_switch_statement(stmt, func_id),
            Statement::Continue => {}
            Statement::Break => {}
            Statement::Debugger => {}
            Statement::Empty => {}
        };

        if !stmt_has_side_effects(statement) {
            *statement = Statement::Empty;
        }
    }

    pub fn visit_maybe_statement(&mut self, stmt: Option<&mut Statement<'a>>, func_id: FuncId) {
        if let Some(stmt) = stmt {
            self.visit_statement(stmt, func_id);
        }
    }

    pub fn visit_many_statements(&mut self, stmt: &mut [Statement<'a>], func_id: FuncId) {
        for stmt in stmt {
            self.visit_statement(stmt, func_id);
        }
    }

    pub fn visit_many_exprs(&mut self, expr: &mut [Expr<'a>], func_id: FuncId) {
        for expr in expr {
            self.visit(expr, func_id);
        }
    }

    pub fn visit_maybe_expr(&mut self, expr: Option<&mut Expr<'a>>, func_id: FuncId) {
        if let Some(expr) = expr {
            self.visit(expr, func_id);
        }
    }

    pub fn visit_return_statement(&mut self, ReturnStatement(expr): &mut ReturnStatement<'a>, func_id: FuncId) {
        self.visit(expr, func_id);
    }

    pub fn visit_try_statement(&mut self, TryCatch { try_, catch, finally }: &mut TryCatch<'a>, func_id: FuncId) {
        self.visit_statement(try_, func_id);
        self.visit_statement(&mut catch.body, func_id);
        self.visit_maybe_statement(finally.as_deref_mut(), func_id);
    }

    pub fn visit_class_statement(&mut self, Class { extends, members, .. }: &mut Class<'a>, func_id: FuncId) {
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
        SwitchStatement { expr, default, cases }: &mut SwitchStatement<'a>,
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

    pub fn visit_loop_statement(&mut self, loop_: &mut Loop<'a>, func_id: FuncId) {
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

    fn visit_variable_binding(
        &mut self,
        _binding: &VariableBinding<'a>,
        value: Option<&mut Expr<'a>>,
        func_id: FuncId,
    ) {
        if let Some(value) = value {
            self.visit(value, func_id);
        }
    }

    pub fn visit_variable_declaration(
        &mut self,
        VariableDeclarations(declarations): &mut VariableDeclarations<'a>,
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
        }: &mut IfStatement<'a>,
        func_id: FuncId,
    ) {
        self.visit(condition, func_id);
        self.visit_statement(then, func_id);
        if let Some(el) = el {
            self.visit_statement(el, func_id);
        }
        let mut branches = branches.borrow_mut();
        for branch in branches.iter_mut() {
            self.visit_if_statement(branch, func_id);
        }
        drop(branches);
    }

    pub fn visit(&mut self, expression: &mut Expr<'a>, func_id: FuncId) {
        match expression {
            Expr::Binary(..) => self.visit_binary_expression(expression, func_id),
            Expr::Grouping(GroupingExpr(expr)) => expr.iter_mut().for_each(|e| self.visit(e, func_id)),
            Expr::Literal(..) => {}
            Expr::Unary(..) => self.visit_unary_expression(expression, func_id),
            Expr::Assignment(..) => self.visit_assignment_expression(expression, func_id),
            Expr::Call(..) => self.visit_call_expression(expression, func_id),
            Expr::Conditional(..) => self.visit_conditional_expression(expression, func_id),
            Expr::PropertyAccess(..) => self.visit_property_access_expression(expression, func_id),
            Expr::Sequence(..) => self.visit_seq_expression(expression, func_id),
            Expr::Prefix(..) => self.visit_prefix_expression(expression, func_id),
            Expr::Postfix(..) => self.visit_postfix_expression(expression, func_id),
            Expr::Function(expr) => self.visit_function_expression(expr, func_id),
            Expr::Array(..) => self.visit_array_expression(expression, func_id),
            Expr::Object(..) => self.visit_object_expression(expression, func_id),
            Expr::Compiled(..) => {}
            Expr::Empty => {}
        }
    }

    fn visit_array_expression(&mut self, array_expr: &mut Expr<'a>, func_id: FuncId) {
        let Expr::Array(ArrayLiteral(array)) = array_expr else {
            unreachable!()
        };

        for expr in array {
            match expr {
                ArrayMemberKind::Spread(expr) => self.visit(expr, func_id),
                ArrayMemberKind::Item(expr) => self.visit(expr, func_id),
            }
        }
    }

    fn visit_object_expression(&mut self, object_expr: &mut Expr<'a>, func_id: FuncId) {
        let Expr::Object(ObjectLiteral(object)) = object_expr else {
            unreachable!()
        };

        for (kind, expr) in object {
            if let ObjectMemberKind::Dynamic(expr) = kind {
                self.visit(expr, func_id);
            }
            self.visit(expr, func_id);
        }
    }

    fn visit_postfix_expression(&mut self, postfix_expr: &mut Expr<'a>, func_id: FuncId) {
        let Expr::Postfix((_, expr)) = postfix_expr else {
            unreachable!()
        };

        self.visit(expr, func_id);
    }

    fn visit_prefix_expression(&mut self, prefix_expr: &mut Expr<'a>, func_id: FuncId) {
        let Expr::Prefix((_, expr)) = prefix_expr else {
            unreachable!()
        };

        self.visit(expr, func_id);
    }

    fn visit_seq_expression(&mut self, seq_expr: &mut Expr<'a>, func_id: FuncId) {
        let Expr::Sequence((left, right)) = seq_expr else {
            unreachable!()
        };

        self.visit(left, func_id);
        self.visit(right, func_id);
    }

    fn visit_property_access_expression(&mut self, property_access_expr: &mut Expr<'a>, func_id: FuncId) {
        let Expr::PropertyAccess(PropertyAccessExpr { target, property, .. }) = property_access_expr else {
            unreachable!()
        };

        self.visit(target, func_id);
        self.visit(property, func_id);
    }

    fn visit_conditional_expression(&mut self, conditional_expr: &mut Expr<'a>, func_id: FuncId) {
        let Expr::Conditional(ConditionalExpr { condition, then, el }) = conditional_expr else {
            unreachable!()
        };
        debug!("reduce conditional {:?}", condition);
        self.visit(condition, func_id);
        self.visit(then, func_id);
        self.visit(el, func_id);

        use Expr::Literal;
        use LiteralExpr::Boolean;

        match &**condition {
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

    fn visit_call_expression(&mut self, call_expr: &mut Expr<'a>, func_id: FuncId) {
        let Expr::Call(FunctionCall { target, arguments, .. }) = call_expr else {
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

    fn visit_assignment_expression(&mut self, assignment_expr: &mut Expr<'a>, func_id: FuncId) {
        let Expr::Assignment(AssignmentExpr { left, right, .. }) = assignment_expr else {
            unreachable!()
        };

        if let AssignmentTarget::Expr(left) = left {
            self.visit(left, func_id);
        }
        self.visit(right, func_id);
    }

    fn visit_unary_expression(&mut self, unary_expr: &mut Expr<'a>, func_id: FuncId) {
        let Expr::Unary(UnaryExpr { operator, expr }) = unary_expr else {
            unreachable!()
        };

        self.visit(expr, func_id);

        use Expr::*;
        use LiteralExpr::*;
        use TokenType::*;

        match (operator, &**expr) {
            (Minus, Literal(Number(n))) => *unary_expr = Literal(Number(-*n)),
            (Plus, Literal(Number(n))) => *unary_expr = Literal(Number(*n)),
            _ => {}
        }
    }

    fn visit_binary_expression(&mut self, binary_expr: &mut Expr<'a>, func_id: FuncId) {
        let Expr::Binary(BinaryExpr { left, right, operator }) = binary_expr else {
            unreachable!()
        };
        debug!("reduce binary: {:?} {:?}", left, right);
        self.visit(left, func_id);
        self.visit(right, func_id);
        debug!("reduced binary to: {:?} {:?}", left, right);

        use Expr::*;
        use LiteralExpr::*;
        use TokenType::*;

        macro_rules! f64_opt {
            ($left:ident $t:tt $right:ident) => {{
                *binary_expr = Literal(Number(*$left $t *$right));
            }};
        }
        macro_rules! float_opt_to_bool {
            ($left:ident $t:tt $right:ident) => {{
                *binary_expr = Literal(Boolean(*$left $t *$right));
            }};
        }

        macro_rules! float_fopt {
            ($fun:expr, $left:ident, $right:ident) => {{
                *binary_expr = Literal(Number($fun(*$left, *$right)));
            }};
        }

        macro_rules! i64_op {
            ($left:ident $t:tt $right:ident) => {
                *binary_expr = Literal(Number(((*$left as i64 as i32) $t (*$right as i64 as i32)) as f64))
            };
        }

        fn truthy_f64(n: f64) -> bool {
            !n.is_nan() && n != 0.0
        }

        match (&**left, &**right, operator) {
            (Literal(Number(left)), Literal(Number(right)), Plus) => f64_opt!(left + right),
            (Literal(Number(left)), Literal(Number(right)), Minus) => f64_opt!(left - right),
            (Literal(Number(left)), Literal(Number(right)), Star) => f64_opt!(left * right),
            (Literal(Number(left)), Literal(Number(right)), Slash) => f64_opt!(left / right),
            (Literal(Number(left)), Literal(Number(right)), Remainder) => f64_opt!(left % right),
            (Literal(Number(left)), Literal(Number(right)), Exponentiation) => float_fopt!(f64::powf, left, right),
            (Literal(Number(left)), Literal(Number(right)), Greater) => float_opt_to_bool!(left > right),
            (Literal(Number(left)), Literal(Number(right)), GreaterEqual) => float_opt_to_bool!(left >= right),
            (Literal(Number(left)), Literal(Number(right)), Less) => float_opt_to_bool!(left < right),
            (Literal(Number(left)), Literal(Number(right)), LessEqual) => float_opt_to_bool!(left <= right),
            (Literal(Number(left)), Literal(Number(right)), Equality) => float_opt_to_bool!(left == right),
            (Literal(Number(left)), Literal(Number(right)), Inequality) => float_opt_to_bool!(left != right),
            (Literal(Number(left)), Literal(Number(right)), StrictEquality) => float_opt_to_bool!(left == right),
            (Literal(Number(left)), Literal(Number(right)), StrictInequality) => float_opt_to_bool!(left != right),
            (Literal(Number(left)), Literal(Number(right)), BitwiseOr) => i64_op!(left | right),
            (Literal(Number(left)), Literal(Number(right)), BitwiseAnd) => i64_op!(left & right),
            (Literal(Number(left)), Literal(Number(right)), BitwiseXor) => i64_op!(left ^ right),
            (Literal(Number(left)), Literal(Number(right)), LeftShift) => i64_op!(left << right),
            (Literal(Number(left)), Literal(Number(right)), RightShift) => i64_op!(left >> right),
            (Literal(Number(left)), Literal(Number(right)), LogicalOr) => {
                *binary_expr = Literal(Number(match truthy_f64(*left) {
                    true => *left,
                    false => *right,
                }))
            }
            (Literal(Number(left)), Literal(Number(right)), LogicalAnd) => {
                *binary_expr = Literal(Number(match truthy_f64(*left) {
                    true => *right,
                    false => *left,
                }))
            }
            (Literal(LiteralExpr::String(left)), Literal(LiteralExpr::String(right)), Equality) => {
                *binary_expr = Literal(Boolean(left == right));
            }
            (Literal(LiteralExpr::String(left)), Literal(LiteralExpr::String(right)), Inequality) => {
                *binary_expr = Literal(Boolean(left == right));
            }
            (Literal(LiteralExpr::String(left)), Literal(LiteralExpr::String(right)), Plus) => {
                let mut left = left.to_string();
                left.push_str(right);
                *binary_expr = Literal(LiteralExpr::String(left.into()));
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
        }: &mut FunctionDeclaration<'a>,
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
                        .add_local(ident, VariableDeclarationKind::Var, None);
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

fn stmt_has_side_effects(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::Block(BlockStatement(block)) => block.iter().any(stmt_has_side_effects),
        Statement::Break => true,
        Statement::Class(Class { .. }) => true, // TODO: can possibly be SE-free
        Statement::Empty => false,
        Statement::Expression(expr) => expr_has_side_effects(expr),
        Statement::Function(FunctionDeclaration { name, .. }) => {
            // Only considered to have side-effects if it's an actual declaration
            name.is_some()
        }
        Statement::If(IfStatement { .. }) => true,
        _ => true,
    }
}

fn expr_has_side_effects(expr: &Expr<'_>) -> bool {
    match expr {
        Expr::Array(ArrayLiteral(array)) => array.iter().any(|k| match k {
            ArrayMemberKind::Item(e) => expr_has_side_effects(e),
            ArrayMemberKind::Spread(e) => expr_has_side_effects(e),
        }),
        Expr::Binary(BinaryExpr { left, right, .. }) => expr_has_side_effects(left) || expr_has_side_effects(right),
        Expr::Conditional(ConditionalExpr { condition, then, el }) => {
            expr_has_side_effects(condition) || expr_has_side_effects(then) || expr_has_side_effects(el)
        }
        Expr::Empty => false,
        Expr::Function(..) => false,
        Expr::Grouping(GroupingExpr(grouping)) => grouping.iter().any(expr_has_side_effects),
        Expr::Literal(LiteralExpr::Boolean(..)) => false,
        Expr::Literal(LiteralExpr::Identifier(..)) => true, // might invoke a global getter
        Expr::Literal(LiteralExpr::Null) => false,
        Expr::Literal(LiteralExpr::Undefined) => false,
        Expr::Literal(LiteralExpr::Number(..)) => false,
        Expr::Literal(LiteralExpr::Regex(..)) => false,
        Expr::Literal(LiteralExpr::String(..)) => false,
        Expr::Object(ObjectLiteral(object)) => object.iter().any(|(kind, expr)| {
            if let ObjectMemberKind::Dynamic(dynamic) = kind {
                if expr_has_side_effects(dynamic) {
                    return true;
                }
            };
            expr_has_side_effects(expr)
        }),
        Expr::Postfix((_, expr)) => expr_has_side_effects(expr),
        Expr::Prefix((_, expr)) => expr_has_side_effects(expr),
        Expr::PropertyAccess(PropertyAccessExpr { target, property, .. }) => {
            expr_has_side_effects(target) || expr_has_side_effects(property)
        }
        Expr::Sequence((left, right)) => expr_has_side_effects(left) || expr_has_side_effects(right),
        Expr::Unary(UnaryExpr { .. }) => true, // TODO: can special case +- literal
        _ => true,
    }
}
