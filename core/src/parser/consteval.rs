use super::{
    expr::{BinaryExpr, Expr, LiteralExpr},
    statement::Statement,
    token::TokenType,
};

/// A trait for evaluating constant expressions.
pub trait Eval {
    /// Attempts to fold an expression or statement prior to execution
    fn fold(&mut self);
    /// Whether this item has side effects
    ///
    /// If this function returns true, it may be entirely removed when folded
    fn has_side_effect(&self) -> bool {
        true
    }
}

impl<'a> Eval for LiteralExpr<'a> {
    fn fold(&mut self) {}
    fn has_side_effect(&self) -> bool {
        // identifier might invoke a global getter
        matches!(self, Self::Identifier(_))
    }
}

impl<'a> Eval for BinaryExpr<'a> {
    fn fold(&mut self) {
        self.left.fold();
        self.right.fold();
    }

    fn has_side_effect(&self) -> bool {
        self.left.has_side_effect() || self.right.has_side_effect()
    }
}

impl<'a> Eval for Expr<'a> {
    fn fold(&mut self) {
        use Expr::*;
        use LiteralExpr::*;

        match self {
            Self::Binary(expr) => {
                expr.fold();

                match (&*expr.left, &*expr.right) {
                    (Literal(Number(l)), Literal(Number(r))) => match expr.operator {
                        TokenType::Plus => *self = Literal(Number(l + r)),
                        TokenType::Minus => *self = Literal(Number(l - r)),
                        TokenType::Star => *self = Literal(Number(l * r)),
                        TokenType::Slash => *self = Literal(Number(l / r)),
                        TokenType::Remainder => *self = Literal(Number(l % r)),
                        TokenType::Exponentiation => *self = Literal(Number(l.powf(*r))),
                        TokenType::Greater => *self = Literal(Boolean(l > r)),
                        TokenType::GreaterEqual => *self = Literal(Boolean(l >= r)),
                        TokenType::Less => *self = Literal(Boolean(l < r)),
                        TokenType::LessEqual => *self = Literal(Boolean(l <= r)),
                        TokenType::Equality => *self = Literal(Boolean(l == r)),
                        TokenType::Inequality => *self = Literal(Boolean(l != r)),
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn has_side_effect(&self) -> bool {
        match self {
            Self::Binary(b) => b.has_side_effect(),
            Self::Literal(l) => l.has_side_effect(),
            _ => true, // assume it does to prevent dead code elimination
        }
    }
}

impl<'a> Eval for Statement<'a> {
    fn fold(&mut self) {
        match self {
            Self::Expression(e) => e.fold(),
            Self::Return(r) => r.0.fold(),
            _ => {}
        };

        if !self.has_side_effect() {
            *self = Statement::Empty;
        }
    }

    fn has_side_effect(&self) -> bool {
        match self {
            Self::Expression(expr) => expr.has_side_effect(),
            _ => true, // assume it does to prevent dead code elimination
        }
    }
}

impl<'a> Eval for [Statement<'a>] {
    fn fold(&mut self) {
        for stmt in self.iter_mut() {
            stmt.fold();
        }
    }
}
