use dash_middle::compiler::FunctionCallKind;
use dash_middle::interner::sym;
use dash_middle::parser::error::Error;
use dash_middle::parser::expr::{CallArgumentKind, ExprKind, FunctionCall, LiteralExpr, PropertyAccessExpr};
use dash_middle::sourcemap::Span;
use dash_middle::visitor::Visitor;

use crate::builder::InstructionBuilder;

impl InstructionBuilder<'_, '_> {
    fn lower_super_call(&mut self, span: Span, fc: FunctionCall) -> Result<Result<(), FunctionCall>, Error> {
        if let ExprKind::Literal(LiteralExpr::Identifier(sym::super_)) = fc.target.kind {
            // Lower `super()` to `this = Reflect.construct(Superclass, [], new.target)`
            self.lower_function_call_common(span, fc.target.span, false, FunctionCallKind::Super, fc.arguments)?;

            self.build_bind_this();

            // Leave the instance on the stack as required by expressions
            // FIXME: not even necessary, `super()` can't be used as an expression
            self.build_this();

            return Ok(Ok(()));
        }

        Ok(Err(fc))
    }

    /// Attempts to specialize a function call
    ///
    /// For example, if the expression is `Math.max(a, b)`, then we can skip
    /// the overhead of a dynamic property lookup at runtime and emit a specialized `max` instruction.
    /// Of course, the VM still needs a guard to account for bad code messing with builtins, e.g.
    /// ```js
    /// let k = input(); // assume k = "max", black box to the compiler
    /// delete Math[k];
    ///
    /// Math.max(1, 2); // *should* throw a TypeError, but will not without a guard
    /// ```
    fn specialize_function_call(&mut self, target: &ExprKind, arguments: &[CallArgumentKind]) -> Result<bool, Error> {
        if let ExprKind::PropertyAccess(PropertyAccessExpr { target, property, .. }) = target {
            let Some(target) = target.kind.as_identifier() else {
                return Ok(false);
            };

            let Some(property) = property.kind.as_identifier() else {
                return Ok(false);
            };

            let Ok(arg_len) = u8::try_from(arguments.len()) else {
                return Ok(false);
            };

            let arguments_iter = arguments.iter().filter_map(|a| match a {
                CallArgumentKind::Normal(expr) => Some(expr),
                // Can't specialize spread args for now
                CallArgumentKind::Spread(_) => None,
            });
            if arguments_iter.clone().count() != arguments.len() {
                return Ok(false);
            }

            macro_rules! emit_spec {
                ($spec:expr) => {{
                    for arg in arguments_iter {
                        self.accept_expr(arg.clone())?;
                    }
                    $spec(self, arg_len);
                    return Ok(true);
                }};
            }

            match (target, property) {
                (sym::Math, sym::exp) => emit_spec!(InstructionBuilder::build_exp),
                (sym::Math, sym::log2) => emit_spec!(InstructionBuilder::build_log2),
                (sym::Math, sym::expm1) => emit_spec!(InstructionBuilder::build_expm1),
                (sym::Math, sym::cbrt) => emit_spec!(InstructionBuilder::build_cbrt),
                (sym::Math, sym::clz32) => emit_spec!(InstructionBuilder::build_clz32),
                (sym::Math, sym::atanh) => emit_spec!(InstructionBuilder::build_atanh),
                (sym::Math, sym::atan2) => emit_spec!(InstructionBuilder::build_atanh2),
                (sym::Math, sym::round) => emit_spec!(InstructionBuilder::build_round),
                (sym::Math, sym::acosh) => emit_spec!(InstructionBuilder::build_acosh),
                (sym::Math, sym::abs) => emit_spec!(InstructionBuilder::build_abs),
                (sym::Math, sym::sinh) => emit_spec!(InstructionBuilder::build_sinh),
                (sym::Math, sym::sin) => emit_spec!(InstructionBuilder::build_sin),
                (sym::Math, sym::ceil) => emit_spec!(InstructionBuilder::build_ceil),
                (sym::Math, sym::tan) => emit_spec!(InstructionBuilder::build_tan),
                (sym::Math, sym::trunc) => emit_spec!(InstructionBuilder::build_trunc),
                (sym::Math, sym::asinh) => emit_spec!(InstructionBuilder::build_asinh),
                (sym::Math, sym::log10) => emit_spec!(InstructionBuilder::build_log10),
                (sym::Math, sym::asin) => emit_spec!(InstructionBuilder::build_asin),
                (sym::Math, sym::random) => emit_spec!(InstructionBuilder::build_random),
                (sym::Math, sym::log1p) => emit_spec!(InstructionBuilder::build_log1p),
                (sym::Math, sym::sqrt) => emit_spec!(InstructionBuilder::build_sqrt),
                (sym::Math, sym::atan) => emit_spec!(InstructionBuilder::build_atan),
                (sym::Math, sym::log) => emit_spec!(InstructionBuilder::build_log),
                (sym::Math, sym::floor) => emit_spec!(InstructionBuilder::build_floor),
                (sym::Math, sym::cosh) => emit_spec!(InstructionBuilder::build_cosh),
                (sym::Math, sym::acos) => emit_spec!(InstructionBuilder::build_acos),
                (sym::Math, sym::cos) => emit_spec!(InstructionBuilder::build_cos),
                _ => {}
            }
        }
        Ok(false)
    }

    pub fn lower_function_call_expr(&mut self, span: Span, fc: FunctionCall) -> Result<(), Error> {
        let target_span = fc.target.span;
        // TODO: this also needs to be specialized for assignment expressions with property access as target

        if self.specialize_function_call(&fc.target.kind, &fc.arguments)? {
            return Ok(());
        }

        let fc = match self.lower_super_call(span, fc)? {
            Ok(()) => return Ok(()),
            Err(fc) => fc,
        };

        let has_this = if let ExprKind::PropertyAccess(p) = fc.target.kind {
            self.visit_property_access_expr(fc.target.span, p, true)?;
            true
        } else {
            self.accept_expr(*fc.target)?;
            false
        };

        let kind = if fc.constructor_call {
            FunctionCallKind::Constructor
        } else {
            FunctionCallKind::Function
        };

        self.lower_function_call_common(span, target_span, has_this, kind, fc.arguments)
    }

    fn lower_function_call_common(
        &mut self,
        span: Span,
        target_span: Span,
        has_this: bool,
        kind: FunctionCallKind,
        arguments: Vec<CallArgumentKind>,
    ) -> Result<(), Error> {
        let argc = u8::try_from(arguments.len()).map_err(|_| Error::ParameterLimitExceeded(span))?;

        let mut spread_arg_indices = Vec::new();

        for (index, arg) in arguments.into_iter().enumerate() {
            match arg {
                CallArgumentKind::Normal(expr) => {
                    self.accept_expr(expr)?;
                }
                CallArgumentKind::Spread(expr) => {
                    self.accept_expr(expr)?;
                    spread_arg_indices.push(index.try_into().unwrap());
                }
            }
        }

        self.build_call(argc, has_this, kind, spread_arg_indices, target_span);

        Ok(())
    }
}
