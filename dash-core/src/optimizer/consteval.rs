use crate::parser::expr::BinaryExpr;
use crate::parser::expr::Expr;
use crate::parser::expr::GroupingExpr;
use crate::parser::expr::LiteralExpr;
use crate::parser::statement::Statement;
use crate::parser::token::TokenType;

pub enum OptLevel {
    None,
    Basic,
    Aggressive,
}

impl Default for OptLevel {
    fn default() -> Self {
        Self::Basic
    }
}

impl OptLevel {
    pub fn from_level(s: &str) -> Option<Self> {
        let l = s.parse::<u8>().ok()?;
        match l {
            0 => Some(Self::None),
            1 => Some(Self::Basic),
            2 => Some(Self::Aggressive),
            _ => None,
        }
    }
}

/// A trait for evaluating constant expressions.
pub trait Eval {
    /// Attempts to fold an expression or statement prior to execution
    fn fold(&mut self, can_remove: bool);
    /// Whether this item has side effects
    ///
    /// If this function returns true, it may be entirely removed when folded
    fn has_side_effect(&self) -> bool {
        true
    }
}

impl<'a> Eval for LiteralExpr<'a> {
    fn fold(&mut self, can_remove: bool) {}
    fn has_side_effect(&self) -> bool {
        // identifier might invoke a global getter
        matches!(self, Self::Identifier(_))
    }
}

impl<'a> Eval for BinaryExpr<'a> {
    fn fold(&mut self, can_remove: bool) {
        self.left.fold(can_remove);
        self.right.fold(can_remove);
    }

    fn has_side_effect(&self) -> bool {
        self.left.has_side_effect() || self.right.has_side_effect()
    }
}

impl<'a> Eval for Expr<'a> {
    fn fold(&mut self, can_remove: bool) {
        use Expr::*;
        use LiteralExpr::*;

        match self {
            Self::Binary(expr) => {
                expr.fold(can_remove);

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
            Grouping(e) => {
                e.0.fold(can_remove);
            }
            Sequence((a, b)) => {
                a.fold(can_remove);
                b.fold(can_remove);
            }
            Conditional(c) => {
                c.condition.fold(can_remove);
                c.el.fold(can_remove);
                c.then.fold(can_remove);

                match c.condition.is_truthy() {
                    Some(true) => {
                        *self = (*c.then).clone();
                    }
                    Some(false) => {
                        *self = (*c.el).clone();
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    }

    fn has_side_effect(&self) -> bool {
        match self {
            Self::Binary(b) => b.has_side_effect(),
            Self::Literal(l) => l.has_side_effect(),
            Self::Grouping(GroupingExpr(ve)) => ve.has_side_effect(),
            _ => true, // assume it does to prevent dead code elimination
        }
    }
}

impl<'a> Eval for Statement<'a> {
    fn fold(&mut self, can_remove: bool) {
        match self {
            Self::Expression(e) => e.fold(can_remove),
            Self::Return(r) => r.0.fold(can_remove),
            Self::If(i) => {
                match i.condition.is_truthy() {
                    Some(true) => {
                        *self = (*i.then).clone();
                    }
                    Some(false) => {
                        if let Some(el) = &i.el {
                            *self = (**el).clone();
                        } else {
                            *self = Statement::Empty;
                        }
                    }
                    _ => {}
                };
            }
            _ => {}
        };

        if can_remove && !self.has_side_effect() {
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
    fn fold(&mut self, can_remove: bool) {
        let len = self.len();
        for (id, stmt) in self.iter_mut().enumerate() {
            let is_last = id == len - 1;

            stmt.fold(can_remove && !is_last);
        }
    }

    fn has_side_effect(&self) -> bool {
        self.iter().any(|e| e.has_side_effect())
    }
}

impl<'a> Eval for [Expr<'a>] {
    fn fold(&mut self, can_remove: bool) {
        for stmt in self.iter_mut() {
            stmt.fold(can_remove);
        }
    }

    fn has_side_effect(&self) -> bool {
        self.iter().any(|e| e.has_side_effect())
    }
}
