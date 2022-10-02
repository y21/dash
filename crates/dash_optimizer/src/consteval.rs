use dash_middle::lexer::token::TokenType;
use dash_middle::parser::expr::ArrayLiteral;
use dash_middle::parser::expr::AssignmentExpr;
use dash_middle::parser::expr::BinaryExpr;
use dash_middle::parser::expr::ConditionalExpr;
use dash_middle::parser::expr::Expr;
use dash_middle::parser::expr::FunctionCall;
use dash_middle::parser::expr::GroupingExpr;
use dash_middle::parser::expr::LiteralExpr;
use dash_middle::parser::expr::ObjectLiteral;
use dash_middle::parser::expr::PropertyAccessExpr;
use dash_middle::parser::expr::UnaryExpr;
use dash_middle::parser::statement::BlockStatement;
use dash_middle::parser::statement::Catch;
use dash_middle::parser::statement::ExportKind;
use dash_middle::parser::statement::ForLoop;
use dash_middle::parser::statement::FunctionDeclaration;
use dash_middle::parser::statement::IfStatement;
use dash_middle::parser::statement::Loop;
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::Statement;
use dash_middle::parser::statement::SwitchStatement;
use dash_middle::parser::statement::TryCatch;
use dash_middle::parser::statement::VariableDeclaration;
use dash_middle::parser::statement::WhileLoop;

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
    fn fold(&mut self, _can_remove: bool) {}
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

        macro_rules! u64op {
            ($l:ident $tok:tt $r:ident) => {
                Literal(Number(((*$l as u64) $tok (*$r as u64)) as f64))
            };
        }

        fn truthy_f64(n: f64) -> bool {
            !n.is_nan() && n != 0.0
        }

        match self {
            Binary(expr) => {
                expr.fold(can_remove);

                match (&*expr.left, &*expr.right) {
                    (Literal(Number(left)), Literal(Number(right))) => match expr.operator {
                        TokenType::Plus => *self = Literal(Number(left + right)),
                        TokenType::Minus => *self = Literal(Number(left - right)),
                        TokenType::Star => *self = Literal(Number(left * right)),
                        TokenType::Slash => *self = Literal(Number(left / right)),
                        TokenType::Remainder => *self = Literal(Number(left % right)),
                        TokenType::Exponentiation => *self = Literal(Number(left.powf(*right))),
                        TokenType::Greater => *self = Literal(Boolean(left > right)),
                        TokenType::GreaterEqual => *self = Literal(Boolean(left >= right)),
                        TokenType::Less => *self = Literal(Boolean(left < right)),
                        TokenType::LessEqual => *self = Literal(Boolean(left <= right)),
                        TokenType::Equality => *self = Literal(Boolean(left == right)),
                        TokenType::Inequality => *self = Literal(Boolean(left != right)),
                        TokenType::StrictEquality => *self = Literal(Boolean(left == right)),
                        TokenType::StrictInequality => *self = Literal(Boolean(left != right)),
                        TokenType::BitwiseOr => *self = u64op!(left | right),
                        TokenType::BitwiseAnd => *self = u64op!(left & right),
                        TokenType::BitwiseXor => *self = u64op!(left ^ right),
                        TokenType::LeftShift => *self = u64op!(left << right),
                        TokenType::RightShift => *self = u64op!(left >> right),
                        TokenType::LogicalOr => {
                            *self = Literal(Number(match truthy_f64(*left) {
                                true => *left,
                                false => *right,
                            }))
                        }
                        TokenType::LogicalAnd => {
                            *self = Literal(Number(match truthy_f64(*left) {
                                true => *right,
                                false => *left,
                            }))
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Grouping(GroupingExpr(expr)) => {
                expr.fold(can_remove);
            }
            Unary(UnaryExpr { operator, expr }) => {
                expr.fold(can_remove);

                match (operator, &**expr) {
                    (TokenType::LogicalNot, Literal(lit)) => match lit {
                        LiteralExpr::Number(n) => *self = Literal(Boolean(!truthy_f64(*n))),
                        LiteralExpr::Boolean(b) => *self = Literal(Boolean(!*b)),
                        LiteralExpr::String(s) => *self = Literal(Boolean(s.is_empty())),
                        LiteralExpr::Null | LiteralExpr::Undefined => *self = Literal(Boolean(true)),
                        _ => {}
                    },
                    (TokenType::Minus, Literal(lit)) => match lit {
                        LiteralExpr::Number(n) => *self = Literal(Number(-n)),
                        LiteralExpr::Boolean(b) => *self = Literal(Number(-(*b as u64 as f64))),
                        _ => {}
                    },
                    (TokenType::Plus, Literal(lit)) => match lit {
                        LiteralExpr::Number(n) => *self = Literal(Number(*n)),
                        LiteralExpr::Boolean(b) => *self = Literal(Number(*b as u64 as f64)),
                        _ => {}
                    },
                    _ => {}
                }
            }
            Assignment(AssignmentExpr { left, right, .. }) => {
                left.fold(can_remove);
                right.fold(can_remove);
            }
            Sequence((left, right)) => {
                left.fold(can_remove);
                right.fold(can_remove);
            }
            PropertyAccess(PropertyAccessExpr { property, target, .. }) => {
                property.fold(can_remove);
                target.fold(can_remove);
            }
            Postfix((_, expr)) => {
                expr.fold(can_remove);
            }
            Function(FunctionDeclaration { statements, .. }) => {
                statements.fold(can_remove);
            }
            Array(ArrayLiteral(lit)) => {
                lit.fold(can_remove);
            }
            Object(ObjectLiteral(lit)) => {
                let it = lit.iter_mut().map(|(_, expr)| expr);

                for expr in it {
                    expr.fold(can_remove);
                }
            }
            Conditional(ConditionalExpr { condition, el, then }) => {
                condition.fold(can_remove);
                el.fold(can_remove);
                then.fold(can_remove);

                match condition.is_truthy() {
                    Some(true) => {
                        *self = (**then).clone();
                    }
                    Some(false) => {
                        *self = (**el).clone();
                    }
                    _ => {}
                };
            }
            Call(FunctionCall { target, arguments, .. }) => {
                target.fold(can_remove);
                arguments.fold(can_remove);
            }
            Literal(..) => {}
            Empty => {}
            Compiled(..) => {}
        }
    }

    fn has_side_effect(&self) -> bool {
        match self {
            Self::Binary(expr) => expr.has_side_effect(),
            Self::Literal(expr) => expr.has_side_effect(),
            Self::Grouping(GroupingExpr(exprs)) => exprs.has_side_effect(),
            _ => true, // assume it does to prevent dead code elimination
        }
    }
}

impl<T: Eval> Eval for Option<T> {
    fn fold(&mut self, can_remove: bool) {
        if let Some(expr) = self {
            expr.fold(can_remove);
        }
    }

    fn has_side_effect(&self) -> bool {
        self.as_ref().map_or(false, |expr| expr.has_side_effect())
    }
}

impl<'a> Eval for Statement<'a> {
    fn fold(&mut self, can_remove: bool) {
        match self {
            Self::Expression(expr) => expr.fold(can_remove),
            Self::Variable(VariableDeclaration { value, .. }) => value.fold(can_remove),
            Self::Return(ReturnStatement(expr)) => expr.fold(can_remove),
            Self::Block(BlockStatement(expr)) => expr.fold(can_remove),
            Self::Function(FunctionDeclaration { statements, .. }) => statements.fold(can_remove),
            Self::Loop(r#loop) => {
                let condition = match r#loop {
                    Loop::For(ForLoop { condition, .. }) => condition.as_mut(),
                    Loop::While(WhileLoop { condition, .. }) => Some(condition),
                    _ => None,
                };

                if let Some(condition) = condition {
                    condition.fold(can_remove);

                    // if the condition is known to always be false,
                    // we can remove the loop entirely
                    if let Some(false) = condition.is_truthy() {
                        *self = Statement::Empty;
                    }
                }
            }
            Self::If(IfStatement {
                condition,
                then,
                branches,
                el,
            }) => {
                condition.fold(can_remove);
                then.fold(can_remove);

                let mut branches = branches.borrow_mut();
                for branch in branches.iter_mut() {
                    branch.condition.fold(can_remove);
                    branch.then.fold(can_remove);
                }
                drop(branches);

                if let Some(el) = el {
                    el.fold(can_remove);
                }

                match condition.is_truthy() {
                    Some(true) => {
                        // if the condition is always true, replace the if statement with the `then` branch statements
                        *self = (**then).clone();
                    }
                    Some(false) => {
                        // if the condition is always false, replace it with the else branch
                        // or if there is no else branch, remove it
                        *self = match el {
                            Some(el) => (**el).clone(),
                            None => Statement::Empty,
                        };
                    }
                    _ => {}
                };
            }
            Self::Try(TryCatch {
                try_,
                catch: Catch { body, .. },
                finally,
            }) => {
                try_.fold(can_remove);
                body.fold(can_remove);
                if let Some(finally) = finally {
                    finally.fold(can_remove);
                }
            }
            Self::Throw(expr) => expr.fold(can_remove),
            Self::Import(..) => {}
            Statement::Export(export) => match export {
                ExportKind::Default(expr) => expr.fold(can_remove),
                _ => {}
            },
            Statement::Class(..) => {}
            Statement::Continue => {}
            Statement::Break => {}
            Statement::Debugger => {}
            Statement::Empty => {}
            Statement::Switch(SwitchStatement { cases, default, expr }) => {
                expr.fold(can_remove);

                if let Some(default) = default {
                    default.fold(can_remove);
                }

                for case in cases {
                    case.body.fold(can_remove);
                    case.value.fold(can_remove);
                }
            }
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
