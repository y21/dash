use dash_middle::compiler::infer_type;
use dash_middle::compiler::scope::CompileValueType;
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
use dash_middle::parser::statement::VariableDeclarationName;
use dash_middle::parser::statement::WhileLoop;

use crate::context::OptimizerContext;

/// A trait for evaluating constant expressions.
pub trait Eval<'a> {
    /// Attempts to fold an expression or statement prior to execution
    fn fold(&mut self, cx: &mut OptimizerContext<'a>, can_remove: bool);
    /// Whether this item has side effects
    ///
    /// If this function returns true, it may be entirely removed when folded
    fn has_side_effect(&self, _cx: &mut OptimizerContext<'a>) -> bool {
        true
    }
}

impl<'a> Eval<'a> for LiteralExpr<'a> {
    fn fold(&mut self, _cx: &mut OptimizerContext<'a>, _can_remove: bool) {}
    fn has_side_effect(&self, _cx: &mut OptimizerContext<'a>) -> bool {
        // identifier might invoke a global getter
        matches!(self, Self::Identifier(_))
    }
}

impl<'a> Eval<'a> for BinaryExpr<'a> {
    fn fold(&mut self, cx: &mut OptimizerContext<'a>, can_remove: bool) {
        self.left.fold(cx, can_remove);
        self.right.fold(cx, can_remove);
    }

    fn has_side_effect(&self, cx: &mut OptimizerContext<'a>) -> bool {
        self.left.has_side_effect(cx) || self.right.has_side_effect(cx)
    }
}

impl<'a> Eval<'a> for Expr<'a> {
    fn fold(&mut self, cx: &mut OptimizerContext<'a>, can_remove: bool) {
        use Expr::*;
        use LiteralExpr::*;

        // infer_type might want to write to scope
        infer_type(cx.scope_mut(), self);

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
                expr.fold(cx, can_remove);

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
                    (Literal(String(left)), Literal(String(right))) => match expr.operator {
                        TokenType::Equality | TokenType::StrictEquality => *self = Literal(Boolean(left == right)),
                        _ => {}
                    },
                    _ => {}
                }
            }
            Grouping(GroupingExpr(expr)) => {
                expr.fold(cx, can_remove);
            }
            Unary(UnaryExpr { operator, expr }) => {
                expr.fold(cx, can_remove);

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
                    (TokenType::Typeof, Literal(lit)) => match lit {
                        LiteralExpr::Number(_) => *self = Literal(String("number".into())),
                        LiteralExpr::Boolean(_) => *self = Literal(String("boolean".into())),
                        LiteralExpr::String(_) => *self = Literal(String("string".into())),
                        LiteralExpr::Null => *self = Literal(String("object".into())),
                        LiteralExpr::Undefined => *self = Literal(String("undefined".into())),
                        LiteralExpr::Identifier(ident) => match cx.scope_mut().find_local(&ident) {
                            Some((_, local)) => match local.inferred_type().borrow().as_ref() {
                                Some(CompileValueType::Boolean) => *self = Literal(String("boolean".into())),
                                Some(CompileValueType::Null) => *self = Literal(String("object".into())),
                                Some(CompileValueType::Undefined) => *self = Literal(String("undefined".into())),
                                Some(CompileValueType::Uninit) => *self = Literal(String("undefined".into())),
                                Some(CompileValueType::Number) => *self = Literal(String("number".into())),
                                Some(CompileValueType::String) => *self = Literal(String("string".into())),
                                // don't guess about Either and Maybe
                                _ => {}
                            },
                            None => {}
                        },
                        LiteralExpr::Binding(..) => {}
                        LiteralExpr::Regex(..) => *self = Literal(String("object".into())),
                    },
                    _ => {}
                }
            }
            Assignment(AssignmentExpr { left, right, .. }) => {
                left.fold(cx, can_remove);
                right.fold(cx, can_remove);
            }
            Sequence((left, right)) => {
                left.fold(cx, can_remove);
                right.fold(cx, can_remove);
            }
            PropertyAccess(PropertyAccessExpr { property, target, .. }) => {
                property.fold(cx, can_remove);
                target.fold(cx, can_remove);
            }
            Postfix((_, expr)) => {
                expr.fold(cx, can_remove);
            }
            Function(FunctionDeclaration { statements, .. }) => {
                statements.fold(cx, can_remove);
            }
            Array(ArrayLiteral(lit)) => {
                lit.fold(cx, can_remove);
            }
            Object(ObjectLiteral(lit)) => {
                let it = lit.iter_mut().map(|(_, expr)| expr);

                for expr in it {
                    expr.fold(cx, can_remove);
                }
            }
            Conditional(ConditionalExpr { condition, el, then }) => {
                condition.fold(cx, can_remove);
                el.fold(cx, can_remove);
                then.fold(cx, can_remove);

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
                target.fold(cx, can_remove);
                arguments.fold(cx, can_remove);
            }
            Literal(..) => {}
            Empty => {}
            Compiled(..) => {}
        }
    }

    fn has_side_effect(&self, cx: &mut OptimizerContext<'a>) -> bool {
        match self {
            Self::Binary(expr) => expr.has_side_effect(cx),
            Self::Literal(expr) => expr.has_side_effect(cx),
            Self::Grouping(GroupingExpr(exprs)) => exprs.has_side_effect(cx),
            _ => true, // assume it does to prevent dead code elimination
        }
    }
}

impl<'a, T: Eval<'a>> Eval<'a> for Option<T> {
    fn fold(&mut self, cx: &mut OptimizerContext<'a>, can_remove: bool) {
        if let Some(expr) = self {
            expr.fold(cx, can_remove);
        }
    }

    fn has_side_effect(&self, cx: &mut OptimizerContext<'a>) -> bool {
        self.as_ref().map_or(false, |expr| expr.has_side_effect(cx))
    }
}

impl<'a> Eval<'a> for Statement<'a> {
    fn fold(&mut self, cx: &mut OptimizerContext<'a>, can_remove: bool) {
        let enters_scope = self.enters_scope();
        if enters_scope {
            cx.scope_mut().enter();
        }

        match self {
            Self::Expression(expr) => expr.fold(cx, can_remove),
            Self::Variable(VariableDeclaration { value, binding }) => {
                value.fold(cx, can_remove);

                if let VariableDeclarationName::Identifier(name) = binding.name {
                    let ty = match value {
                        Some(expr) => infer_type(cx.scope_mut(), expr),
                        None => Some(CompileValueType::Uninit),
                    };

                    // TODO: can't really do anything with the error here
                    // once we have logging, log the error
                    let _ = cx.scope_mut().add_local(name, binding.kind, false, ty);
                }
            }
            Self::Return(ReturnStatement(expr)) => expr.fold(cx, can_remove),
            Self::Block(BlockStatement(expr)) => expr.fold(cx, can_remove),
            Self::Function(FunctionDeclaration { statements, .. }) => statements.fold(cx, can_remove),
            Self::Loop(r#loop) => {
                let condition = match r#loop {
                    Loop::For(ForLoop { condition, .. }) => condition.as_mut(),
                    Loop::While(WhileLoop { condition, .. }) => Some(condition),
                    _ => None,
                };

                if let Some(condition) = condition {
                    condition.fold(cx, can_remove);

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
                condition.fold(cx, can_remove);
                then.fold(cx, can_remove);

                let mut branches = branches.borrow_mut();
                for branch in branches.iter_mut() {
                    branch.condition.fold(cx, can_remove);
                    branch.then.fold(cx, can_remove);
                }
                drop(branches);

                if let Some(el) = el {
                    el.fold(cx, can_remove);
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
                try_.fold(cx, can_remove);
                body.fold(cx, can_remove);
                if let Some(finally) = finally {
                    finally.fold(cx, can_remove);
                }
            }
            Statement::Throw(expr) => expr.fold(cx, can_remove),
            Statement::Import(..) => {}
            Statement::Export(export) => match export {
                ExportKind::Default(expr) => expr.fold(cx, can_remove),
                _ => {}
            },
            Statement::Class(..) => {}
            Statement::Continue => {}
            Statement::Break => {}
            Statement::Debugger => {}
            Statement::Empty => {}
            Statement::Switch(SwitchStatement { cases, default, expr }) => {
                expr.fold(cx, can_remove);

                if let Some(default) = default {
                    default.fold(cx, can_remove);
                }

                for case in cases {
                    case.body.fold(cx, can_remove);
                    case.value.fold(cx, can_remove);
                }
            }
        };

        if can_remove && !self.has_side_effect(cx) {
            *self = Statement::Empty;
        }

        if enters_scope {
            cx.scope_mut().exit();
        }
    }

    fn has_side_effect(&self, cx: &mut OptimizerContext<'a>) -> bool {
        match self {
            Self::Expression(expr) => expr.has_side_effect(cx),
            _ => true, // assume it does to prevent dead code elimination
        }
    }
}

impl<'a> Eval<'a> for [Statement<'a>] {
    fn fold(&mut self, cx: &mut OptimizerContext<'a>, can_remove: bool) {
        let len = self.len();
        for (id, stmt) in self.iter_mut().enumerate() {
            let is_last = id == len - 1;

            stmt.fold(cx, can_remove && !is_last);
        }
    }

    fn has_side_effect(&self, cx: &mut OptimizerContext<'a>) -> bool {
        self.iter().any(|e| e.has_side_effect(cx))
    }
}

impl<'a> Eval<'a> for [Expr<'a>] {
    fn fold(&mut self, cx: &mut OptimizerContext<'a>, can_remove: bool) {
        for stmt in self.iter_mut() {
            stmt.fold(cx, can_remove);
        }
    }

    fn has_side_effect(&self, cx: &mut OptimizerContext<'a>) -> bool {
        self.iter().any(|e| e.has_side_effect(cx))
    }
}
