#![feature(let_chains)]

use dash_middle::interner::StringInterner;
use dash_middle::parser::error::Error;
use dash_middle::parser::expr::Expr;
use dash_middle::parser::expr::ExprKind;
use dash_middle::parser::expr::LiteralExpr;
use dash_middle::parser::statement::Statement;
use dash_middle::parser::statement::StatementKind;
use dash_middle::parser::statement::VariableDeclaration;
use dash_middle::parser::statement::VariableDeclarations;
use dash_middle::parser::types::LiteralType;
use dash_middle::parser::types::TypeSegment;

pub struct TypeckCtxt<'i, 's> {
    pub interner: &'i mut StringInterner,
    pub source: &'s str,
    pub errors: Vec<Error>,
}

type InferenceResult = Option<TypeSegment>;

#[derive(Debug, Copy, Clone)]
enum Variance {
    Covariance,
    Contravariance,
    Invariance,
}

fn can_eq<'ast>(
    to_check: &'ast TypeSegment,
    check_against: &'ast TypeSegment,
    variance: Variance,
) -> (bool, InferenceResult, InferenceResult) {
    match (to_check, check_against) {
        (TypeSegment::Any | TypeSegment::Never, _) => (true, None, None),
        (_, TypeSegment::Any | TypeSegment::Never) => (true, None, None),
        (TypeSegment::Literal(left), TypeSegment::Literal(right)) if left == right => (true, None, None),
        (to_check, TypeSegment::Union(left, right)) => (
            can_eq(to_check, left, variance).0 || can_eq(to_check, right, variance).0,
            None,
            None,
        ),
        (TypeSegment::Union(left, right), check_against) => (
            can_eq(left, check_against, variance).0 || can_eq(right, check_against, variance).0,
            None,
            None,
        ),
        (TypeSegment::Literal(LiteralType::Number(_)), TypeSegment::Number) => (true, None, None),
        (TypeSegment::Number, TypeSegment::Literal(LiteralType::Number(_))) => (true, None, None),
        (TypeSegment::Number, TypeSegment::Number) => (true, None, None),
        (TypeSegment::String, TypeSegment::String) => (true, None, None),
        (TypeSegment::Boolean, TypeSegment::Boolean) => (true, None, None),
        _ => (false, None, None),
    }
}

fn type_of_literal(lit: &LiteralExpr) -> TypeSegment {
    match lit {
        LiteralExpr::Boolean(b) => TypeSegment::Literal(LiteralType::Boolean(*b)),
        LiteralExpr::Number(n) => TypeSegment::Literal(LiteralType::Number(*n)),
        LiteralExpr::String(s) => TypeSegment::Literal(LiteralType::Identifier(*s)),
        LiteralExpr::Identifier(ident) => TypeSegment::Literal(LiteralType::Identifier(*ident)),
        LiteralExpr::Null => todo!(),
        LiteralExpr::Undefined => todo!(),
        _ => todo!(),
    }
}

impl<'i, 's> TypeckCtxt<'i, 's> {
    pub fn check_expr(&mut self, expr: &Expr, expectation: &TypeSegment) {
        match &expr.kind {
            ExprKind::Literal(lit) => {
                let got = type_of_literal(lit);
                if !can_eq(&got, expectation, Variance::Contravariance).0 {
                    self.errors.push(Error::TypeMismatch {
                        span: expr.span,
                        got,
                        expected: expectation.clone(),
                    });
                }
            }
            _ => todo!(),
        }
    }
    pub fn check_stmt(&mut self, stmt: &Statement) {
        match &stmt.kind {
            StatementKind::Variable(VariableDeclarations(decls)) => {
                for VariableDeclaration { binding, value } in decls {
                    // TODO: store variable & type for later
                    if let Some(value) = value
                        && let Some(ty) = &binding.ty
                    {
                        self.check_expr(value, ty);
                    }
                }
            }
            _ => todo!(),
        }
    }
    pub fn check_stmts(&mut self, stmts: &[Statement]) {
        for stmt in stmts {
            self.check_stmt(stmt);
        }
    }
}
