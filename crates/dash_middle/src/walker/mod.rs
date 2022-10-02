use crate::parser::expr::ArrayLiteral;
use crate::parser::expr::AssignmentExpr;
use crate::parser::expr::BinaryExpr;
use crate::parser::expr::ConditionalExpr;
use crate::parser::expr::Expr;
use crate::parser::expr::FunctionCall;
use crate::parser::expr::GroupingExpr;
use crate::parser::expr::LiteralExpr;
use crate::parser::expr::ObjectLiteral;
use crate::parser::expr::ObjectMemberKind;
use crate::parser::expr::Postfix;
use crate::parser::expr::PropertyAccessExpr;
use crate::parser::expr::Seq;
use crate::parser::expr::UnaryExpr;
use crate::parser::statement::BlockStatement;
use crate::parser::statement::Class;
use crate::parser::statement::ClassMemberKind;
use crate::parser::statement::ClassProperty;
use crate::parser::statement::ExportKind;
use crate::parser::statement::ForInLoop;
use crate::parser::statement::ForLoop;
use crate::parser::statement::ForOfLoop;
use crate::parser::statement::FunctionDeclaration;
use crate::parser::statement::IfStatement;
use crate::parser::statement::ImportKind;
use crate::parser::statement::Loop;
use crate::parser::statement::ReturnStatement;
use crate::parser::statement::Statement;
use crate::parser::statement::SwitchStatement;
use crate::parser::statement::TryCatch;
use crate::parser::statement::VariableBinding;
use crate::parser::statement::VariableDeclaration;
use crate::parser::statement::WhileLoop;

pub mod function_local;

pub trait AstWalker<'a> {
    fn accept(&mut self, e: Statement<'a>) {
        accept_default(self, e)
    }

    fn accept_expr(&mut self, e: Expr<'a>) {
        accept_expr_default(self, e)
    }

    fn visit_expression_statement(&mut self, _e: Expr<'a>) {}

    fn visit_binary_expression(&mut self, e: BinaryExpr<'a>) {
        self.accept_expr(*e.left);
        self.accept_expr(*e.right);
    }

    fn visit_grouping_expression(&mut self, e: GroupingExpr<'a>) {
        accept_expr_many(self, e.0)
    }

    fn visit_literal_expression(&mut self, _e: LiteralExpr<'a>) {}

    fn visit_identifier_expression(&mut self, _i: &str) {}

    fn visit_binding_expression(&mut self, _b: VariableBinding<'a>) {}

    fn visit_unary_expression(&mut self, e: UnaryExpr<'a>) {
        self.accept_expr(*e.expr);
    }

    fn visit_variable_declaration(&mut self, v: VariableDeclaration<'a>) {
        if let Some(expr) = v.value {
            self.accept_expr(expr);
        }
    }

    fn visit_if_statement(&mut self, i: IfStatement<'a>) -> () {
        self.accept_expr(i.condition);
        let branches = std::mem::take(&mut *i.branches.borrow_mut());
        for branch in branches {
            self.accept(Statement::If(branch));
        }
        accept_maybe_box(self, i.el);
        self.accept(*i.then);
    }

    fn visit_block_statement(&mut self, b: BlockStatement<'a>) {
        accept_many(self, b.0)
    }

    fn visit_function_declaration(&mut self, f: FunctionDeclaration<'a>) {
        accept_many(self, f.statements)
    }

    fn visit_while_loop(&mut self, l: WhileLoop<'a>) -> () {
        self.accept(*l.body);
        self.accept_expr(l.condition);
    }

    fn visit_assignment_expression(&mut self, e: AssignmentExpr<'a>) -> () {
        self.accept_expr(*e.left);
        self.accept_expr(*e.right);
    }

    fn visit_function_call(&mut self, c: FunctionCall<'a>) -> () {
        self.accept_expr(*c.target);
        accept_expr_many(self, c.arguments);
    }

    fn visit_return_statement(&mut self, s: ReturnStatement<'a>) -> () {
        self.accept_expr(s.0);
    }

    fn visit_conditional_expr(&mut self, c: ConditionalExpr<'a>) -> () {
        self.accept_expr(*c.condition);
        self.accept_expr(*c.el);
        self.accept_expr(*c.then);
    }

    fn visit_property_access_expr(&mut self, e: PropertyAccessExpr<'a>, _preserve_this: bool) -> () {
        self.accept_expr(*e.property);
        self.accept_expr(*e.target);
    }

    fn visit_sequence_expr(&mut self, s: Seq<'a>) -> () {
        self.accept_expr(*s.0);
        self.accept_expr(*s.1);
    }

    fn visit_postfix_expr(&mut self, p: Postfix<'a>) -> () {
        self.accept_expr(*p.1);
    }

    fn visit_function_expr(&mut self, f: FunctionDeclaration<'a>) -> () {
        self.accept(Statement::Function(f));
    }

    fn visit_array_literal(&mut self, a: ArrayLiteral<'a>) -> () {
        accept_expr_many(self, a.0);
    }

    fn visit_object_literal(&mut self, o: ObjectLiteral<'a>) -> () {
        for (kind, expr) in o.0 {
            self.accept_expr(expr);

            if let ObjectMemberKind::Dynamic(d) = kind {
                self.accept_expr(d);
            }
        }
    }

    fn visit_try_catch(&mut self, t: TryCatch<'a>) -> () {
        self.accept(*t.try_);
        self.accept(*t.catch.body);
        accept_maybe_box(self, t.finally);
    }

    fn visit_throw(&mut self, e: Expr<'a>) -> () {
        self.accept_expr(e);
    }

    fn visit_for_loop(&mut self, f: ForLoop<'a>) -> () {
        accept_maybe_box(self, f.init);
        accept_maybe_expr(self, f.condition);
        accept_maybe_expr(self, f.finalizer);
        self.accept(*f.body);
    }

    fn visit_for_of_loop(&mut self, f: ForOfLoop<'a>) -> () {
        self.accept_expr(f.expr);
        self.accept(*f.body);
    }

    fn visit_for_in_loop(&mut self, f: ForInLoop<'a>) -> () {
        self.accept_expr(f.expr);
        self.accept(*f.body);
    }

    fn visit_import_statement(&mut self, i: ImportKind<'a>) -> () {
        if let ImportKind::Dynamic(dy) = i {
            self.accept_expr(dy);
        }
    }

    fn visit_export_statement(&mut self, e: ExportKind<'a>) -> () {
        match e {
            ExportKind::Default(e) => self.accept_expr(e),
            ExportKind::NamedVar(v) => {
                for decl in v {
                    self.accept(Statement::Variable(decl));
                }
            }
            ExportKind::Named(..) => {}
        }
    }

    fn visit_empty_statement(&mut self) -> () {}

    fn visit_break(&mut self) -> () {}

    fn visit_continue(&mut self) -> () {}

    fn visit_debugger(&mut self) -> () {}

    fn visit_empty_expr(&mut self) -> () {}

    fn visit_class_declaration(&mut self, c: Class<'a>) -> () {
        accept_maybe_expr(self, c.extends);
        for member in c.members {
            match member.kind {
                ClassMemberKind::Method(m) => self.accept(Statement::Function(m)),
                ClassMemberKind::Property(ClassProperty { value, .. }) => accept_maybe_expr(self, value),
            }
        }
    }

    fn visit_switch_statement(&mut self, s: SwitchStatement<'a>) -> () {
        self.accept_expr(s.expr);
        if let Some(default) = s.default {
            accept_many(self, default);
        }
        for case in s.cases {
            self.accept_expr(case.value);
            accept_many(self, case.body);
        }
    }
}

pub fn accept_default<'a, V: AstWalker<'a> + ?Sized>(this: &mut V, s: Statement<'a>) {
    match s {
        Statement::Expression(e) => this.visit_expression_statement(e),
        Statement::Variable(v) => this.visit_variable_declaration(v),
        Statement::If(i) => this.visit_if_statement(i),
        Statement::Block(b) => this.visit_block_statement(b),
        Statement::Function(f) => this.visit_function_declaration(f),
        Statement::Loop(Loop::For(f)) => this.visit_for_loop(f),
        Statement::Loop(Loop::While(w)) => this.visit_while_loop(w),
        Statement::Loop(Loop::ForOf(f)) => this.visit_for_of_loop(f),
        Statement::Loop(Loop::ForIn(f)) => this.visit_for_in_loop(f),
        Statement::Return(r) => this.visit_return_statement(r),
        Statement::Try(t) => this.visit_try_catch(t),
        Statement::Throw(t) => this.visit_throw(t),
        Statement::Import(i) => this.visit_import_statement(i),
        Statement::Export(e) => this.visit_export_statement(e),
        Statement::Class(c) => this.visit_class_declaration(c),
        Statement::Continue => this.visit_continue(),
        Statement::Break => this.visit_break(),
        Statement::Debugger => this.visit_debugger(),
        Statement::Empty => this.visit_empty_statement(),
        Statement::Switch(s) => this.visit_switch_statement(s),
    }
}

pub fn accept_expr_default<'a, V: AstWalker<'a> + ?Sized>(this: &mut V, e: Expr<'a>) {
    match e {
        Expr::Binary(e) => this.visit_binary_expression(e),
        Expr::Assignment(e) => this.visit_assignment_expression(e),
        Expr::Grouping(e) => this.visit_grouping_expression(e),
        Expr::Literal(LiteralExpr::Binding(b)) => this.visit_binding_expression(b),
        Expr::Literal(LiteralExpr::Identifier(i)) => this.visit_identifier_expression(&i),
        Expr::Literal(l) => this.visit_literal_expression(l),
        Expr::Unary(e) => this.visit_unary_expression(e),
        Expr::Call(e) => this.visit_function_call(e),
        Expr::Conditional(e) => this.visit_conditional_expr(e),
        Expr::PropertyAccess(e) => this.visit_property_access_expr(e, false),
        Expr::Sequence(e) => this.visit_sequence_expr(e),
        Expr::Postfix(e) => this.visit_postfix_expr(e),
        Expr::Function(e) => this.visit_function_expr(e),
        Expr::Array(e) => this.visit_array_literal(e),
        Expr::Object(e) => this.visit_object_literal(e),
        Expr::Compiled(..) => (),
        Expr::Empty => this.visit_empty_expr(),
    }
}

pub fn accept_many<'a, V, I>(this: &mut V, stmt: I)
where
    I: IntoIterator<Item = Statement<'a>>,
    V: AstWalker<'a> + ?Sized,
{
    for s in stmt {
        accept_default(this, s);
    }
}
pub fn accept_expr_many<'a, V, I>(this: &mut V, stmt: I)
where
    I: IntoIterator<Item = Expr<'a>>,
    V: AstWalker<'a> + ?Sized,
{
    for s in stmt {
        accept_expr_default(this, s);
    }
}

pub fn accept_maybe<'a, V: AstWalker<'a> + ?Sized>(this: &mut V, stmt: Option<Statement<'a>>) {
    if let Some(s) = stmt {
        accept_default(this, s);
    }
}

pub fn accept_maybe_expr<'a, V: AstWalker<'a> + ?Sized>(this: &mut V, stmt: Option<Expr<'a>>) {
    if let Some(s) = stmt {
        accept_expr_default(this, s);
    }
}

pub fn accept_maybe_box<'a, V: AstWalker<'a> + ?Sized>(this: &mut V, stmt: Option<Box<Statement<'a>>>) {
    accept_maybe(this, stmt.map(|x| *x))
}
