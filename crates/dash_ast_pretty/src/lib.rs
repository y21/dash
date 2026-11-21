use std::fmt::Write;

use dash_middle::interner::StringInterner;
use dash_middle::lexer::token::TokenType;
use dash_middle::parser::expr::{
    ArrayLiteral, ArrayMemberKind, AssignmentExpr, AssignmentTarget, BinaryExpr, CallArgumentKind, ConditionalExpr,
    DynamicImport, Expr, ExprKind, FunctionCall, GroupingExpr, LiteralExpr, ObjectLiteral, ObjectMemberKind,
    OptionalChainingComponent, OptionalChainingExpression, PropertyAccessExpr, UnaryExpr,
};
use dash_middle::parser::statement::{
    Asyncness, BlockStatement, Catch, Class, ClassMember, ClassMemberKey, ClassMemberValue, DoWhileLoop, ExportKind,
    ForInLoop, ForLoop, ForOfLoop, FunctionDeclaration, FunctionKind, IfStatement, ImportKind, Loop, Parameter,
    Pattern, ReturnStatement, SpecifierKind, Statement, StatementKind, SwitchStatement, TryCatch, VariableBinding,
    VariableDeclaration, VariableDeclarationKind, VariableDeclarationName, VariableDeclarations, WhileLoop,
};

pub struct PrettyCx<'a> {
    pub out: &'a mut String,
    pub interner: &'a StringInterner,
    pub depth: u32,
}

impl PrettyCx<'_> {
    pub fn linebreak(&mut self) {
        self.out.push('\n');
        let indent = self.depth as usize * 2;
        write!(&mut self.out, "{:indent$}", "").unwrap();
    }

    pub fn nest<R>(&mut self, f: impl FnOnce(&mut PrettyCx<'_>) -> R) -> R {
        let mut nested = PrettyCx {
            out: self.out,
            interner: self.interner,
            depth: self.depth + 1,
        };
        f(&mut nested)
    }
}

fn pretty_expr_into(cx: &mut PrettyCx<'_>, expr: &Expr) {
    match &expr.kind {
        ExprKind::Binary(BinaryExpr { left, right, operator }) => {
            pretty_expr_into(cx, left);
            cx.out.push(' ');
            write!(&mut cx.out, "{operator}").unwrap();
            cx.out.push(' ');
            pretty_expr_into(cx, right);
        }
        ExprKind::Grouping(GroupingExpr(exprs)) => {
            cx.out.push('(');
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    cx.out.push_str(", ");
                }
                pretty_expr_into(cx, expr);
            }
            cx.out.push(')');
        }
        ExprKind::Literal(literal) => pretty_literal_into(cx, literal),
        ExprKind::Unary(UnaryExpr {
            operator,
            expr: operand,
        }) => {
            write!(&mut cx.out, "{operator}").unwrap();
            if matches!(
                *operator,
                TokenType::Await | TokenType::Delete | TokenType::Void | TokenType::Typeof | TokenType::Yield
            ) {
                cx.out.push(' ');
            }
            pretty_expr_into(cx, operand);
        }
        ExprKind::YieldStar(expr) => {
            cx.out.push_str("yield* ");
            pretty_expr_into(cx, expr);
        }
        ExprKind::Assignment(AssignmentExpr { left, right, operator }) => {
            let target = match left {
                AssignmentTarget::Expr(expr) => expr,
                AssignmentTarget::LocalId(id) => {
                    panic!("attempted to pretty print compiler-local assignment target {id:?}")
                }
            };
            pretty_expr_into(cx, target);
            cx.out.push(' ');
            write!(&mut cx.out, "{operator}").unwrap();
            cx.out.push(' ');
            pretty_expr_into(cx, right);
        }
        ExprKind::Call(FunctionCall {
            constructor_call,
            target,
            arguments,
        }) => {
            if *constructor_call {
                cx.out.push_str("new ");
            }
            pretty_expr_into(cx, target);
            cx.out.push('(');
            for (i, argument) in arguments.iter().enumerate() {
                if i > 0 {
                    cx.out.push_str(", ");
                }
                match argument {
                    CallArgumentKind::Normal(expr) => pretty_expr_into(cx, expr),
                    CallArgumentKind::Spread(expr) => {
                        cx.out.push_str("...");
                        pretty_expr_into(cx, expr);
                    }
                }
            }
            cx.out.push(')');
        }
        ExprKind::NewTarget => cx.out.push_str("new.target"),
        ExprKind::Conditional(ConditionalExpr { condition, then, el }) => {
            pretty_expr_into(cx, condition);
            cx.out.push_str(" ? ");
            pretty_expr_into(cx, then);
            cx.out.push_str(" : ");
            pretty_expr_into(cx, el);
        }
        ExprKind::PropertyAccess(PropertyAccessExpr {
            computed,
            target,
            property,
        }) => {
            pretty_expr_into(cx, target);
            if *computed {
                cx.out.push('[');
                pretty_expr_into(cx, property);
                cx.out.push(']');
            } else {
                cx.out.push('.');
                pretty_expr_into(cx, property);
            }
        }
        ExprKind::Sequence((left, right)) => {
            pretty_expr_into(cx, left);
            cx.out.push_str(", ");
            pretty_expr_into(cx, right);
        }
        ExprKind::Prefix((operator, expr)) => {
            write!(&mut cx.out, "{operator}").unwrap();
            pretty_expr_into(cx, expr);
        }
        ExprKind::Postfix((operator, expr)) => {
            pretty_expr_into(cx, expr);
            write!(&mut cx.out, "{operator}").unwrap();
        }
        ExprKind::Function(function) => pretty_function_into(cx, function),
        ExprKind::Class(class) => pretty_class_into(cx, class),
        ExprKind::Array(ArrayLiteral { members, .. }) => {
            cx.out.push('[');
            for (i, member) in members.iter().enumerate() {
                if i > 0 {
                    cx.out.push_str(", ");
                }
                match member {
                    ArrayMemberKind::Item(expr) => pretty_expr_into(cx, expr),
                    ArrayMemberKind::Spread(expr) => {
                        cx.out.push_str("...");
                        pretty_expr_into(cx, expr);
                    }
                    ArrayMemberKind::Empty => {}
                }
            }
            cx.out.push(']');
        }
        ExprKind::Object(object) => pretty_object_into(cx, object),
        ExprKind::Chaining(OptionalChainingExpression { base, components }) => {
            pretty_expr_into(cx, base);
            for component in components {
                match component {
                    OptionalChainingComponent::Ident { property, .. } => {
                        cx.out.push_str("?.");
                        cx.out.push_str(cx.interner.resolve(*property));
                    }
                    OptionalChainingComponent::Dyn { property, .. } => {
                        cx.out.push_str("?.[");
                        pretty_expr_into(cx, property);
                        cx.out.push(']');
                    }
                    OptionalChainingComponent::Call(arguments) | OptionalChainingComponent::Construct(arguments) => {
                        cx.out.push_str("?.(");
                        for (i, argument) in arguments.iter().enumerate() {
                            if i > 0 {
                                cx.out.push_str(", ");
                            }
                            match argument {
                                CallArgumentKind::Normal(expr) => pretty_expr_into(cx, expr),
                                CallArgumentKind::Spread(expr) => {
                                    cx.out.push_str("...");
                                    pretty_expr_into(cx, expr);
                                }
                            }
                        }
                        cx.out.push(')');
                    }
                }
            }
        }
        ExprKind::Compiled(_) => panic!("attempted to pretty print compiled bytecode"),
        ExprKind::Import(DynamicImport { specifier, options }) => {
            cx.out.push_str("import(");
            pretty_expr_into(cx, specifier);
            if let Some(options) = options {
                cx.out.push_str(", ");
                pretty_expr_into(cx, options);
            }
            cx.out.push(')');
        }
        ExprKind::Empty => cx.out.push_str("void 0"),
    }
}

fn pretty_literal_into(cx: &mut PrettyCx<'_>, literal: &LiteralExpr) {
    match literal {
        LiteralExpr::Boolean(boolean) => write!(&mut cx.out, "{boolean}").unwrap(),
        LiteralExpr::Identifier(identifier) => cx.out.push_str(cx.interner.resolve(*identifier)),
        LiteralExpr::Number(number) => {
            if number.is_nan() {
                cx.out.push_str("NaN");
            } else if *number == f64::INFINITY {
                cx.out.push_str("Infinity");
            } else if *number == f64::NEG_INFINITY {
                cx.out.push_str("-Infinity");
            } else {
                write!(&mut cx.out, "{number}").unwrap();
            }
        }
        LiteralExpr::String(symbol) => write!(&mut cx.out, "{:?}", cx.interner.resolve(*symbol)).unwrap(),
        LiteralExpr::Regex(_, source) => {
            cx.out.push('/');
            cx.out.push_str(cx.interner.resolve(*source));
            cx.out.push('/');
        }
        LiteralExpr::Null => cx.out.push_str("null"),
        LiteralExpr::This => cx.out.push_str("this"),
    }
}

fn object_key_prefix(cx: &mut PrettyCx<'_>, kind: &ObjectMemberKind, value: &Expr) {
    match kind {
        ObjectMemberKind::Getter(..) | ObjectMemberKind::DynamicGetter(..) => cx.out.push_str("get "),
        ObjectMemberKind::Setter(..) | ObjectMemberKind::DynamicSetter(..) => cx.out.push_str("set "),
        ObjectMemberKind::Static(..)
        | ObjectMemberKind::Spread
        | ObjectMemberKind::Dynamic(..)
        | ObjectMemberKind::Default(..) => {}
    }
    if let ExprKind::Function(function) = &value.kind {
        if matches!(function.ty, FunctionKind::Function(Asyncness::Yes)) {
            cx.out.push_str("async ");
        }
        if matches!(function.ty, FunctionKind::Generator) {
            cx.out.push('*');
        }
    }
}

fn pretty_object_value_into(cx: &mut PrettyCx<'_>, value: &Expr) {
    if let ExprKind::Function(function) = &value.kind
        && !matches!(function.ty, FunctionKind::Arrow)
    {
        pretty_function_sig_body_into(cx, function);
    } else {
        cx.out.push_str(": ");
        pretty_expr_into(cx, value);
    }
}

fn pretty_object_into(cx: &mut PrettyCx<'_>, object: &ObjectLiteral) {
    cx.out.push('{');
    for (i, (kind, value)) in object.members.iter().enumerate() {
        if i > 0 {
            cx.out.push_str(", ");
        }
        match kind {
            ObjectMemberKind::Spread => {
                cx.out.push_str("...");
                pretty_expr_into(cx, value);
            }
            ObjectMemberKind::Static(symbol) => {
                if let ExprKind::Literal(LiteralExpr::Identifier(id)) = &value.kind
                    && id == symbol
                {
                    cx.out.push_str(cx.interner.resolve(*symbol));
                } else {
                    object_key_prefix(cx, kind, value);
                    write!(&mut cx.out, "{:?}", cx.interner.resolve(*symbol)).unwrap();
                    pretty_object_value_into(cx, value);
                }
            }
            ObjectMemberKind::Dynamic(expr) => {
                object_key_prefix(cx, kind, value);
                cx.out.push('[');
                pretty_expr_into(cx, expr);
                cx.out.push(']');
                pretty_object_value_into(cx, value);
            }
            ObjectMemberKind::Getter(symbol) | ObjectMemberKind::Setter(symbol) => {
                object_key_prefix(cx, kind, value);
                cx.out.push_str(cx.interner.resolve(*symbol));
                pretty_object_value_into(cx, value);
            }
            ObjectMemberKind::DynamicGetter(expr) | ObjectMemberKind::DynamicSetter(expr) => {
                object_key_prefix(cx, kind, value);
                cx.out.push('[');
                pretty_expr_into(cx, expr);
                cx.out.push(']');
                pretty_object_value_into(cx, value);
            }
            ObjectMemberKind::Default(symbol) => {
                cx.out.push_str(cx.interner.resolve(*symbol));
                cx.out.push_str(" = ");
                pretty_expr_into(cx, value);
            }
        }
    }
    cx.out.push('}');
}

fn pretty_params_into(cx: &mut PrettyCx<'_>, function: &FunctionDeclaration) {
    cx.out.push('(');
    for (i, (parameter, default, _)) in function.parameters.iter().enumerate() {
        if i > 0 {
            cx.out.push_str(", ");
        }
        match parameter {
            Parameter::Identifier(binding) => cx.out.push_str(cx.interner.resolve(binding.ident)),
            Parameter::SpreadIdentifier(binding) => {
                cx.out.push_str("...");
                cx.out.push_str(cx.interner.resolve(binding.ident));
            }
            Parameter::Pattern(_, pattern) => pretty_pattern_into(cx, pattern),
            Parameter::SpreadPattern(_, pattern) => {
                cx.out.push_str("...");
                pretty_pattern_into(cx, pattern);
            }
        }
        if let Some(default) = default {
            cx.out.push_str(" = ");
            pretty_expr_into(cx, default);
        }
    }
    cx.out.push(')');
}

fn pretty_stmts_braced_into(cx: &mut PrettyCx<'_>, statements: &[Statement]) {
    cx.out.push_str(" {");
    cx.nest(|cx| pretty_stmts_into(cx, statements));
    cx.linebreak();
    cx.out.push('}');
}

fn pretty_function_sig_body_into(cx: &mut PrettyCx<'_>, function: &FunctionDeclaration) {
    pretty_params_into(cx, function);
    pretty_stmts_braced_into(cx, &function.statements);
}

fn pretty_function_into(cx: &mut PrettyCx<'_>, function: &FunctionDeclaration) {
    match function.ty {
        FunctionKind::Function(Asyncness::Yes) => cx.out.push_str("async "),
        FunctionKind::Function(Asyncness::No) | FunctionKind::Generator => {}
        FunctionKind::Arrow => {
            pretty_params_into(cx, function);
            cx.out.push_str(" => ");
            pretty_stmts_braced_into(cx, &function.statements);
            return;
        }
    }
    cx.out.push_str("function");
    if matches!(function.ty, FunctionKind::Generator) {
        cx.out.push('*');
    }
    cx.out.push(' ');
    if let Some(name) = function.name {
        cx.out.push_str(cx.interner.resolve(name.ident));
    }
    pretty_function_sig_body_into(cx, function);
}

fn pretty_class_into(cx: &mut PrettyCx<'_>, class: &Class) {
    cx.out.push_str("class");
    if let Some(name) = class.name {
        cx.out.push(' ');
        cx.out.push_str(cx.interner.resolve(name.ident));
    }
    if let Some(extends) = &class.extends {
        cx.out.push_str(" extends ");
        pretty_expr_into(cx, extends);
    }
    cx.out.push_str(" {");
    cx.nest(|cx| {
        for member in &class.members {
            cx.linebreak();
            pretty_class_member_into(cx, member);
        }
    });
    cx.linebreak();
    cx.out.push('}');
}

fn pretty_class_member_into(cx: &mut PrettyCx<'_>, member: &ClassMember) {
    if member.static_ {
        cx.out.push_str("static ");
    }
    match &member.value {
        ClassMemberValue::Getter(..) => cx.out.push_str("get "),
        ClassMemberValue::Setter(..) => cx.out.push_str("set "),
        ClassMemberValue::Method(function) => {
            if matches!(function.ty, FunctionKind::Function(Asyncness::Yes)) {
                cx.out.push_str("async ");
            }
            if matches!(function.ty, FunctionKind::Generator) {
                cx.out.push('*');
            }
        }
        ClassMemberValue::Field(..) => {}
    }

    if member.private {
        cx.out.push('#');
    }

    match &member.key {
        ClassMemberKey::Named(name, _) => cx.out.push_str(cx.interner.resolve(*name)),
        ClassMemberKey::Computed(expr) => {
            cx.out.push('[');
            pretty_expr_into(cx, expr);
            cx.out.push(']');
        }
    }
    match &member.value {
        ClassMemberValue::Method(function)
        | ClassMemberValue::Getter(function)
        | ClassMemberValue::Setter(function) => pretty_function_sig_body_into(cx, function),
        ClassMemberValue::Field(None) => cx.out.push(';'),
        ClassMemberValue::Field(Some(expr)) => {
            cx.out.push_str(" = ");
            pretty_expr_into(cx, expr);
            cx.out.push(';');
        }
    }
}

fn pretty_pattern_into(cx: &mut PrettyCx<'_>, pattern: &Pattern) {
    match pattern {
        Pattern::Object { fields, rest } => {
            cx.out.push('{');
            for (i, &(_, field, alias, ref default)) in fields.iter().enumerate() {
                if i > 0 {
                    cx.out.push_str(", ");
                }
                cx.out.push_str(cx.interner.resolve(field));
                if let Some(alias) = alias {
                    cx.out.push_str(": ");
                    cx.out.push_str(cx.interner.resolve(alias));
                }
                if let Some(default) = default {
                    cx.out.push_str(" = ");
                    pretty_expr_into(cx, default);
                }
            }
            if let Some(rest) = rest {
                if !fields.is_empty() {
                    cx.out.push_str(", ");
                }
                cx.out.push_str("...");
                cx.out.push_str(cx.interner.resolve(rest.ident));
            }
            cx.out.push('}');
        }
        Pattern::Array { fields, rest } => {
            cx.out.push('[');
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    cx.out.push_str(", ");
                }
                if let Some((binding, default)) = field {
                    cx.out.push_str(cx.interner.resolve(binding.ident));
                    if let Some(default) = default {
                        cx.out.push_str(" = ");
                        pretty_expr_into(cx, default);
                    }
                }
            }
            if let Some(rest) = rest {
                if !fields.is_empty() {
                    cx.out.push_str(", ");
                }
                cx.out.push_str("...");
                cx.out.push_str(cx.interner.resolve(rest.ident));
            }
            cx.out.push(']');
        }
    }
}

fn pretty_binding_kind_into(cx: &mut PrettyCx<'_>, kind: &VariableDeclarationKind) {
    match kind {
        VariableDeclarationKind::Var => cx.out.push_str("var"),
        VariableDeclarationKind::Let => cx.out.push_str("let"),
        VariableDeclarationKind::Const => cx.out.push_str("const"),
        VariableDeclarationKind::Arguments | VariableDeclarationKind::Unnameable => {
            panic!("attempted to pretty print {:?} binding", kind)
        }
    }
}

fn pretty_var_declarations_into(cx: &mut PrettyCx<'_>, declarations: &VariableDeclarations) {
    if let Some(decl) = declarations.0.first() {
        pretty_binding_kind_into(cx, &decl.binding.kind);
    }

    for (i, VariableDeclaration { binding, value }) in declarations.0.iter().enumerate() {
        if i > 0 {
            cx.out.push_str(", ");
        }
        pretty_binding_into(cx, binding);
        if let Some(value) = value {
            cx.out.push_str(" = ");
            pretty_expr_into(cx, value);
        }
    }
}

fn pretty_binding_into(cx: &mut PrettyCx<'_>, binding: &VariableBinding) {
    cx.out.push(' ');

    match &binding.name {
        VariableDeclarationName::Identifier(binding) => cx.out.push_str(cx.interner.resolve(binding.ident)),
        VariableDeclarationName::Pattern(pattern) => pretty_pattern_into(cx, pattern),
    }
}

fn pretty_stmt_inlined_block_into(cx: &mut PrettyCx<'_>, statement: &Statement) {
    if let StatementKind::Block(block) = &statement.kind {
        cx.out.push(' ');
        pretty_block_into(cx, block);
    } else {
        cx.nest(|cx| {
            cx.linebreak();
            pretty_stmt_into(cx, statement);
        });
    }
}

fn pretty_block_into(cx: &mut PrettyCx<'_>, BlockStatement(statements, _): &BlockStatement) {
    cx.out.push('{');
    cx.nest(|cx| pretty_stmts_into(cx, statements));
    cx.linebreak();
    cx.out.push('}');
}

fn pretty_if_into(cx: &mut PrettyCx<'_>, if_stmt: &IfStatement) {
    cx.out.push_str("if (");
    pretty_expr_into(cx, &if_stmt.condition);
    cx.out.push(')');
    pretty_stmt_no_semi_into(cx, &if_stmt.then);

    for branch in &if_stmt.branches {
        cx.linebreak();
        cx.out.push_str("else if (");
        pretty_expr_into(cx, &branch.condition);
        cx.out.push(')');
        pretty_stmt_no_semi_into(cx, &branch.then);
    }

    if let Some(el) = &if_stmt.el {
        cx.linebreak();
        cx.out.push_str("else");
        pretty_stmt_inlined_block_into(cx, el);
    }
}

fn pretty_loop_into(cx: &mut PrettyCx<'_>, loop_: &Loop) {
    match loop_ {
        Loop::For(ForLoop {
            init,
            condition,
            finalizer,
            body,
            scope: _,
        }) => {
            cx.out.push_str("for (");
            if let Some(init) = init {
                pretty_stmt_no_semi_into(cx, init);
            }
            cx.out.push(';');
            if let Some(condition) = condition {
                cx.out.push(' ');
                pretty_expr_into(cx, condition);
            }
            cx.out.push_str("; ");
            if let Some(finalizer) = finalizer {
                pretty_expr_into(cx, finalizer);
            }
            cx.out.push(')');
            pretty_stmt_inlined_block_into(cx, body);
        }
        Loop::ForOf(ForOfLoop {
            binding,
            expr,
            body,
            scope: _,
        }) => {
            cx.out.push_str("for (");
            pretty_binding_kind_into(cx, &binding.kind);
            pretty_binding_into(cx, binding);
            cx.out.push_str(" of ");
            pretty_expr_into(cx, expr);
            cx.out.push(')');
            pretty_stmt_inlined_block_into(cx, body);
        }
        Loop::ForIn(ForInLoop {
            binding,
            expr,
            body,
            scope: _,
        }) => {
            cx.out.push_str("for (");
            pretty_binding_kind_into(cx, &binding.kind);
            pretty_binding_into(cx, binding);
            cx.out.push_str(" in ");
            pretty_expr_into(cx, expr);
            cx.out.push(')');
            pretty_stmt_inlined_block_into(cx, body);
        }
        Loop::While(WhileLoop { condition, body }) => {
            cx.out.push_str("while (");
            pretty_expr_into(cx, condition);
            cx.out.push(')');
            pretty_stmt_inlined_block_into(cx, body);
        }
        Loop::DoWhile(DoWhileLoop { body, condition }) => {
            cx.out.push_str("do");
            pretty_stmt_inlined_block_into(cx, body);
            cx.linebreak();
            cx.out.push_str("while (");
            pretty_expr_into(cx, condition);
            cx.out.push_str(")");
        }
    }
}

fn pretty_try_into(cx: &mut PrettyCx<'_>, try_catch: &TryCatch) {
    cx.out.push_str("try");
    pretty_stmt_no_semi_into(cx, &try_catch.try_);

    if let Some(Catch { body, binding, .. }) = &try_catch.catch {
        cx.linebreak();
        cx.out.push_str("catch");
        if let Some(binding) = binding {
            cx.out.push_str(" (");
            cx.out.push_str(cx.interner.resolve(binding.ident));
            cx.out.push(')');
        }
        cx.out.push(' ');
        pretty_block_into(cx, body);
    }

    if let Some(finally) = &try_catch.finally {
        cx.linebreak();
        cx.out.push_str("finally");
        pretty_stmt_no_semi_into(cx, finally);
    }
}

fn pretty_switch_into(cx: &mut PrettyCx<'_>, switch: &SwitchStatement) {
    cx.out.push_str("switch (");
    pretty_expr_into(cx, &switch.expr);
    cx.out.push_str(") {");
    cx.nest(|cx| {
        for case in &switch.cases {
            cx.linebreak();
            cx.out.push_str("case ");
            pretty_expr_into(cx, &case.value);
            cx.out.push(':');
            cx.nest(|cx| pretty_stmts_into(cx, &case.body));
        }
        if let Some(default) = &switch.default {
            cx.linebreak();
            cx.out.push_str("default:");
            cx.nest(|cx| pretty_stmts_into(cx, default));
        }
    });
    cx.linebreak();
    cx.out.push('}');
}

fn pretty_specifier_into(cx: &mut PrettyCx<'_>, specifier: &SpecifierKind) {
    match specifier {
        SpecifierKind::Ident(binding) => cx.out.push_str(cx.interner.resolve(binding.ident)),
    }
}

fn pretty_import_into(cx: &mut PrettyCx<'_>, import: &ImportKind) {
    cx.out.push_str("import ");
    match import {
        ImportKind::DefaultAs(specifier, source) => {
            pretty_specifier_into(cx, specifier);
            cx.out.push_str(" from ");
            write!(&mut cx.out, "{:?}", cx.interner.resolve(*source)).unwrap();
        }
        ImportKind::AllAs(specifier, source) => {
            cx.out.push_str("* as ");
            pretty_specifier_into(cx, specifier);
            cx.out.push_str(" from ");
            write!(&mut cx.out, "{:?}", cx.interner.resolve(*source)).unwrap();
        }
    }
}

fn pretty_export_into(cx: &mut PrettyCx<'_>, export: &ExportKind) {
    match export {
        ExportKind::Default(expr) => {
            cx.out.push_str("export default ");
            pretty_expr_into(cx, expr);
        }
        ExportKind::Named(names) => {
            cx.out.push_str("export {");
            for (i, name) in names.iter().enumerate() {
                if i > 0 {
                    cx.out.push_str(", ");
                }
                cx.out.push_str(cx.interner.resolve(*name));
            }
            cx.out.push_str("}");
        }
        ExportKind::NamedVar(declarations) => {
            cx.out.push_str("export ");
            pretty_var_declarations_into(cx, declarations);
        }
    }
}

pub fn pretty_stmt_into(cx: &mut PrettyCx<'_>, stmt: &Statement) {
    pretty_stmt_no_semi_into(cx, stmt);
    cx.out.push(';');
}

pub fn pretty_stmt_no_semi_into(cx: &mut PrettyCx<'_>, stmt: &Statement) {
    match &stmt.kind {
        StatementKind::Expression(expr) => {
            pretty_expr_into(cx, expr);
        }
        StatementKind::Variable(declarations) => {
            pretty_var_declarations_into(cx, declarations);
        }
        StatementKind::If(if_stmt) => pretty_if_into(cx, if_stmt),
        StatementKind::Block(block) => pretty_block_into(cx, block),
        StatementKind::Function(function) => pretty_function_into(cx, function),
        StatementKind::Loop(loop_) => pretty_loop_into(cx, loop_),
        StatementKind::Return(ReturnStatement(expr)) => {
            cx.out.push_str("return ");
            pretty_expr_into(cx, expr);
        }
        StatementKind::Try(try_catch) => pretty_try_into(cx, try_catch),
        StatementKind::Throw(expr) => {
            cx.out.push_str("throw ");
            pretty_expr_into(cx, expr);
        }
        StatementKind::Import(import) => pretty_import_into(cx, import),
        StatementKind::Export(export) => pretty_export_into(cx, export),
        StatementKind::Class(class) => pretty_class_into(cx, class),
        StatementKind::Switch(switch) => pretty_switch_into(cx, switch),
        StatementKind::Continue(label) => {
            cx.out.push_str("continue");
            if let Some(label) = label {
                cx.out.push(' ');
                cx.out.push_str(cx.interner.resolve(*label));
            }
        }
        StatementKind::Break(label) => {
            cx.out.push_str("break");
            if let Some(label) = label {
                cx.out.push(' ');
                cx.out.push_str(cx.interner.resolve(*label));
            }
        }
        StatementKind::Debugger => cx.out.push_str("debugger"),
        StatementKind::Labelled(label, statement) => {
            cx.out.push_str(cx.interner.resolve(*label));
            cx.out.push_str(": ");
            pretty_stmt_no_semi_into(cx, statement);
        }
        StatementKind::Empty => {}
    }
}

pub fn pretty_stmts_into(cx: &mut PrettyCx<'_>, stmts: &[Statement]) {
    for stmt in stmts {
        cx.linebreak();
        pretty_stmt_into(cx, stmt);
    }
}
