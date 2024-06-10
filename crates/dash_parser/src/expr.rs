use dash_middle::interner::{sym, Symbol};
use dash_middle::lexer::token::{Token, TokenType, ASSIGNMENT_TYPES};
use dash_middle::parser::error::Error;
use dash_middle::parser::expr::{
    ArrayMemberKind, AssignmentExpr, AssignmentTarget, CallArgumentKind, Expr, ExprKind, LiteralExpr, ObjectMemberKind,
};
use dash_middle::parser::statement::{
    Asyncness, BlockStatement, FunctionDeclaration, FunctionKind, Parameter, ReturnStatement, Statement, StatementKind,
};
use dash_middle::sourcemap::Span;
use dash_regex::Flags;

use crate::Parser;

impl<'a, 'interner> Parser<'a, 'interner> {
    pub fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_sequence()
    }

    /// Parses an expression without the comma operator.
    pub fn parse_expression_no_comma(&mut self) -> Option<Expr> {
        // Impl detail of the expression parser: comma has a lower precedence than yield, so start by parsing yield exprs
        self.parse_yield()
    }

    fn parse_sequence(&mut self) -> Option<Expr> {
        let mut expr = self.parse_yield()?;

        while self.expect_token_type_and_skip(&[TokenType::Comma], false) {
            let right = self.parse_sequence()?;
            expr = Expr::grouping(vec![expr, right]);
        }

        Some(expr)
    }

    fn parse_yield(&mut self) -> Option<Expr> {
        if self.expect_token_type_and_skip(&[TokenType::Yield], false) {
            let lo_span = self.previous()?.span;
            let right = self.parse_yield()?;
            return Some(Expr {
                span: lo_span.to(right.span),
                kind: ExprKind::unary(TokenType::Yield, right),
            });
        }

        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<Expr> {
        let mut expr = self.parse_ternary()?;

        if self.expect_token_type_and_skip(ASSIGNMENT_TYPES, false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_yield()?;
            expr = Expr::assignment(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_ternary(&mut self) -> Option<Expr> {
        let mut expr = self.parse_nullish_coalescing()?;

        while self.expect_token_type_and_skip(&[TokenType::Conditional], false) {
            let then_branch = self.parse_yield()?;
            if !self.expect_token_type_and_skip(&[TokenType::Colon], true) {
                return None;
            }
            let else_branch = self.parse_yield()?;
            expr = Expr::conditional(expr, then_branch, else_branch);
        }

        Some(expr)
    }

    fn parse_nullish_coalescing(&mut self) -> Option<Expr> {
        let mut expr = self.parse_logical_or()?;

        while self.expect_token_type_and_skip(&[TokenType::NullishCoalescing], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_logical_or()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_logical_or(&mut self) -> Option<Expr> {
        let mut expr = self.parse_logical_and()?;

        while self.expect_token_type_and_skip(&[TokenType::LogicalOr], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_logical_and()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_logical_and(&mut self) -> Option<Expr> {
        let mut expr = self.parse_bitwise_or()?;

        while self.expect_token_type_and_skip(&[TokenType::LogicalAnd], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_bitwise_or()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_bitwise_or(&mut self) -> Option<Expr> {
        let mut expr = self.parse_bitwise_xor()?;

        while self.expect_token_type_and_skip(&[TokenType::BitwiseOr], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_bitwise_xor()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_bitwise_xor(&mut self) -> Option<Expr> {
        let mut expr = self.parse_bitwise_and()?;

        while self.expect_token_type_and_skip(&[TokenType::BitwiseXor], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_bitwise_and()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_bitwise_and(&mut self) -> Option<Expr> {
        let mut expr = self.parse_equality()?;

        while self.expect_token_type_and_skip(&[TokenType::BitwiseAnd], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_equality()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_equality(&mut self) -> Option<Expr> {
        let mut expr = self.parse_comparison()?;

        while self.expect_token_type_and_skip(
            &[
                TokenType::Inequality,
                TokenType::Equality,
                TokenType::StrictEquality,
                TokenType::StrictInequality,
            ],
            false,
        ) {
            let operator = self.previous()?.ty;
            let rval = self.parse_comparison()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_comparison(&mut self) -> Option<Expr> {
        let mut expr = self.parse_bitwise_shift()?;

        while self.expect_token_type_and_skip(
            &[
                TokenType::Greater,
                TokenType::Less,
                TokenType::GreaterEqual,
                TokenType::LessEqual,
                TokenType::In,
                TokenType::Instanceof,
            ],
            false,
        ) {
            let operator = self.previous()?.ty;
            let rval = self.parse_bitwise_shift()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_bitwise_shift(&mut self) -> Option<Expr> {
        let mut expr = self.parse_term()?;

        while self.expect_token_type_and_skip(
            &[
                TokenType::LeftShift,
                TokenType::RightShift,
                TokenType::UnsignedRightShift,
            ],
            false,
        ) {
            let operator = self.previous()?.ty;
            let rval = self.parse_term()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_term(&mut self) -> Option<Expr> {
        let mut expr = self.parse_factor()?;

        while self.expect_token_type_and_skip(&[TokenType::Plus, TokenType::Minus], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_factor()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_factor(&mut self) -> Option<Expr> {
        let mut expr = self.parse_pow()?;

        while self.expect_token_type_and_skip(&[TokenType::Star, TokenType::Slash, TokenType::Remainder], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_pow()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_pow(&mut self) -> Option<Expr> {
        let mut expr = self.parse_unary()?;

        while self.expect_token_type_and_skip(&[TokenType::Exponentiation], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_unary()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        if self.expect_token_type_and_skip(
            &[
                TokenType::LogicalNot,
                TokenType::Minus,
                TokenType::Await,
                TokenType::Delete,
                TokenType::Void,
                TokenType::Typeof,
                TokenType::Decrement,
                TokenType::Increment,
                TokenType::Plus,
                TokenType::BitwiseNot,
            ],
            false,
        ) {
            let Token { span, ty } = *self.previous()?;
            let rval = self.parse_unary()?;
            let span = span.to(rval.span);

            if [TokenType::Increment, TokenType::Decrement].contains(&ty) {
                Some(Expr {
                    span,
                    kind: ExprKind::prefix(ty, rval),
                })
            } else {
                Some(Expr {
                    span,
                    kind: ExprKind::unary(ty, rval),
                })
            }
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Option<Expr> {
        let expr = self.parse_field_access()?;
        if self.expect_token_type_and_skip(&[TokenType::Increment, TokenType::Decrement], false) {
            let Token { span, ty } = *self.previous()?;
            return Some(Expr {
                span: expr.span.to(span),
                kind: ExprKind::postfix(ty, expr),
            });
        }
        Some(expr)
    }

    fn parse_field_access(&mut self) -> Option<Expr> {
        if self.expect_token_type_and_skip(&[TokenType::New], false) {
            self.new_level_stack
                .inc_level()
                .expect("Failed to increment `new` stack level");

            let rval = self.parse_field_access()?;

            return Some(rval);
        }

        let mut expr = self.parse_primary_expr()?;

        while self.expect_token_type_and_skip(
            &[TokenType::LeftParen, TokenType::Dot, TokenType::LeftSquareBrace],
            false,
        ) {
            let previous = self.previous()?.ty;

            match previous {
                TokenType::LeftParen => {
                    let mut arguments = Vec::new();

                    // Disassociate any new expressions in arguments such that `new x(() => x());`
                    // is parsed as having the `new` operator only apply to the `x` identifier.
                    self.new_level_stack.add_level();
                    // TODO: refactor to `parse_expr_list`
                    while !self.expect_token_type_and_skip(&[TokenType::RightParen], false) {
                        self.expect_token_type_and_skip(&[TokenType::Comma], false);

                        if let Some(spread) = self.parse_spread_operator(false) {
                            arguments.push(CallArgumentKind::Spread(spread));
                        } else {
                            arguments.push(CallArgumentKind::Normal(self.parse_yield()?));
                        }
                    }
                    self.new_level_stack.pop_level().expect("Missing `new` level stack");

                    // End of function call.
                    let level = self.new_level_stack.cur_level().expect("Missing `new` level stack");
                    let is_constructor_call = level > 0;
                    if is_constructor_call {
                        self.new_level_stack.dec_level().expect("Missing `new` level stack");
                    }

                    expr = Expr {
                        span: expr.span.to(self.previous()?.span),
                        kind: ExprKind::function_call(expr, arguments, is_constructor_call),
                    };
                }
                TokenType::Dot => {
                    let ident = self.expect_identifier_or_reserved_kw(true)?;
                    let property = Expr {
                        span: self.previous()?.span,
                        kind: ExprKind::identifier(ident),
                    };
                    expr = Expr {
                        span: expr.span.to(property.span),
                        kind: ExprKind::property_access(false, expr, property),
                    };
                }
                TokenType::LeftSquareBrace => {
                    let property = self.parse_expression()?;
                    self.expect_token_type_and_skip(&[TokenType::RightSquareBrace], true);
                    expr = Expr {
                        span: expr.span.to(property.span),
                        kind: ExprKind::property_access(true, expr, property),
                    };
                }
                _ => unreachable!(),
            }
        }

        Some(expr)
    }

    /// Tries to parse a spread operator (...<expr>). The argument specifies if it's required.
    fn parse_spread_operator(&mut self, must_parse: bool) -> Option<Expr> {
        if self.expect_token_type_and_skip(&[TokenType::Dot], must_parse) {
            for _ in 0..2 {
                let token = self.next()?;
                if !matches!(token.ty, TokenType::Dot) {
                    let token = token.clone();
                    self.create_error(Error::IncompleteSpread(token));
                    return None;
                }
            }
            self.parse_yield()
        } else {
            None
        }
    }

    fn parse_primary_expr(&mut self) -> Option<Expr> {
        let current = self.current()?.clone();

        self.advance();

        let expr = match current.ty {
            // removed to resolve #58
            TokenType::TemplateLiteral(sym) => {
                let mut left = Expr {
                    span: current.span,
                    kind: ExprKind::string_literal(sym),
                };
                while !self.is_eof() {
                    if self.expect_token_type_and_skip(&[TokenType::Dollar], false) {
                        self.expect_token_type_and_skip(&[TokenType::LeftBrace], true);
                        let right = self.parse_expression()?;
                        self.expect_token_type_and_skip(&[TokenType::RightBrace], true);
                        left = Expr::binary(left, right, TokenType::Plus);
                    } else if let Some(sym) = self.expect_template_literal(false) {
                        let right = Expr {
                            span: self.previous()?.span,
                            kind: ExprKind::string_literal(sym),
                        };
                        left = Expr::binary(left, right, TokenType::Plus);
                    } else {
                        break;
                    }
                }
                left
            }
            TokenType::FalseLit => Expr {
                span: current.span,
                kind: ExprKind::bool_literal(false),
            },
            TokenType::TrueLit => Expr {
                span: current.span,
                kind: ExprKind::bool_literal(true),
            },
            TokenType::NullLit => Expr {
                span: current.span,
                kind: ExprKind::null_literal(),
            },
            TokenType::UndefinedLit => Expr {
                span: current.span,
                kind: ExprKind::undefined_literal(),
            },
            TokenType::String(sym) => Expr {
                span: current.span,
                kind: ExprKind::string_literal(sym),
            },
            TokenType::LeftSquareBrace => {
                let mut items = Vec::new();
                while !self.expect_token_type_and_skip(&[TokenType::RightSquareBrace], false) {
                    if self.expect_token_type_and_skip(&[TokenType::Comma], false) {
                        items.push(ArrayMemberKind::Empty);
                        // don't consume following comma as a separator
                        continue;
                    } else if let Some(spread) = self.parse_spread_operator(false) {
                        items.push(ArrayMemberKind::Spread(spread));
                    } else {
                        items.push(ArrayMemberKind::Item(self.parse_yield()?));
                    }
                    self.expect_token_type_and_skip(&[TokenType::Comma], false);
                }
                let rbrace_span = self.previous()?.span;
                Expr {
                    span: current.span.to(rbrace_span),
                    kind: ExprKind::array_literal(items),
                }
            }
            TokenType::LeftBrace => {
                let mut items = Vec::new();
                while !self.expect_token_type_and_skip(&[TokenType::RightBrace], false) {
                    self.expect_token_type_and_skip(&[TokenType::Comma], false);

                    // Allow trailing comma in object literal {f:1,}
                    if self.expect_token_type_and_skip(&[TokenType::RightBrace], false) {
                        break;
                    }

                    let token = self.next()?.clone();
                    let key = match token.ty {
                        TokenType::Get => {
                            if let Some(sym) = self.current().and_then(|t| t.ty.as_identifier()) {
                                self.advance();
                                ObjectMemberKind::Getter(sym)
                            } else {
                                ObjectMemberKind::Static(sym::get)
                            }
                        }
                        TokenType::Set => {
                            if let Some(sym) = self.current().and_then(|t| t.ty.as_identifier()) {
                                self.advance();
                                ObjectMemberKind::Setter(sym)
                            } else {
                                ObjectMemberKind::Static(sym::set)
                            }
                        }
                        TokenType::LeftSquareBrace => {
                            let t = self.parse_expression()?;
                            let o = ObjectMemberKind::Dynamic(t);
                            self.expect_token_type_and_skip(&[TokenType::RightSquareBrace], true);
                            o
                        }
                        TokenType::Dot => {
                            // `.` indicates spread operator `...expr`
                            for _ in 0..2 {
                                let token = self.next()?;
                                if !matches!(token.ty, TokenType::Dot) {
                                    let token = token.clone();
                                    self.create_error(Error::IncompleteSpread(token));
                                    return None;
                                }
                            }
                            ObjectMemberKind::Spread
                        }
                        other => {
                            if let Some(ident) = other.as_property_name() {
                                ObjectMemberKind::Static(ident)
                            } else {
                                self.create_error(Error::UnexpectedToken(token, TokenType::DUMMY_IDENTIFIER));
                                return None;
                            }
                        }
                    };

                    match key {
                        ObjectMemberKind::Spread => {
                            items.push((key, self.parse_yield()?));
                        }
                        ObjectMemberKind::Dynamic(..) | ObjectMemberKind::Static(..) => {
                            if self.expect_token_type_and_skip(&[TokenType::Colon], false) {
                                // Normal property.
                                let value = self.parse_yield()?;
                                items.push((key, value));
                            } else if self.expect_token_type_and_skip(&[TokenType::LeftParen], false) {
                                // Method.
                                let parameters = self.parse_parameter_list()?;
                                self.expect_token_type_and_skip(&[TokenType::LeftBrace], true);
                                let body = self.parse_block()?;
                                let id = self.function_counter.inc();
                                items.push((
                                    key,
                                    Expr {
                                        span: current.span.to(self.previous()?.span),
                                        kind: ExprKind::function(FunctionDeclaration::new(
                                            None,
                                            id,
                                            parameters,
                                            body.0,
                                            FunctionKind::Function(Asyncness::No),
                                            None,
                                            None,
                                        )),
                                    },
                                ));
                            } else {
                                match key {
                                    ObjectMemberKind::Static(name) => items.push((
                                        key,
                                        Expr {
                                            span: token.span,
                                            kind: ExprKind::identifier(name),
                                        },
                                    )),
                                    ObjectMemberKind::Dynamic(..) => {
                                        self.create_error(Error::UnexpectedToken(token, TokenType::Colon));
                                        return None;
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        }
                        ObjectMemberKind::Getter(..) | ObjectMemberKind::Setter(..) => {
                            self.expect_token_type_and_skip(&[TokenType::LeftParen], true);
                            let params = self.parse_parameter_list()?;

                            // Make sure parameter count is correct
                            match key {
                                ObjectMemberKind::Setter(..) => {
                                    if params.len() != 1 {
                                        self.create_error(Error::InvalidAccessorParams {
                                            token,
                                            expect: 1,
                                            got: params.len(),
                                        });
                                        return None;
                                    }
                                }
                                ObjectMemberKind::Getter(..) => {
                                    if !params.is_empty() {
                                        self.create_error(Error::InvalidAccessorParams {
                                            token,
                                            expect: 0,
                                            got: params.len(),
                                        });
                                        return None;
                                    }
                                }
                                _ => unreachable!(),
                            }

                            self.expect_token_type_and_skip(&[TokenType::LeftBrace], true);
                            let BlockStatement(stmts) = self.parse_block()?;

                            // Desugar to function
                            let func_id = self.function_counter.inc();
                            let fun = FunctionDeclaration::new(
                                None,
                                func_id,
                                params,
                                stmts,
                                FunctionKind::Function(Asyncness::No),
                                None,
                                None,
                            );
                            items.push((
                                key,
                                Expr {
                                    span: current.span.to(self.previous()?.span),
                                    kind: ExprKind::function(fun),
                                },
                            ));
                        }
                        ObjectMemberKind::DynamicGetter(_) | ObjectMemberKind::DynamicSetter(_) => {
                            unreachable!("never created")
                        }
                    }
                }
                let rbrace_span = self.previous()?.span;
                Expr {
                    span: current.span.to(rbrace_span),
                    kind: ExprKind::object_literal(items),
                }
            }
            // TODO: this unwrap is not safe
            TokenType::NumberDec(sym) => Expr {
                span: current.span,
                kind: ExprKind::number_literal(self.interner.resolve(sym).parse::<f64>().unwrap()),
            },
            TokenType::NumberHex(sym) => self.parse_prefixed_number_literal(current.span, sym, 16)?,
            TokenType::NumberBin(sym) => self.parse_prefixed_number_literal(current.span, sym, 2)?,
            TokenType::NumberOct(sym) => self.parse_prefixed_number_literal(current.span, sym, 8)?,
            TokenType::LeftParen => {
                // Parsing groups and closures
                if self.expect_token_type_and_skip(&[TokenType::RightParen], false) {
                    // () MUST be followed by an arrow. Empty groups are not valid syntax
                    if !self.expect_token_type_and_skip(&[TokenType::FatArrow], true) {
                        return None;
                    }

                    return self.parse_arrow_function_end(current.span, Vec::new(), None);
                }

                self.new_level_stack.add_level();
                let mut exprs = Vec::new();
                let mut rest_binding = None;

                while !self.expect_token_type_and_skip(&[TokenType::RightParen], false) {
                    self.expect_token_type_and_skip(&[TokenType::Comma], false);

                    if self.expect_token_type_and_skip(&[TokenType::Dot], false) {
                        for _ in 0..2 {
                            if !self.expect_token_type_and_skip(&[TokenType::Dot], true) {
                                return None;
                            }
                        }

                        let span = self.current()?.span;
                        rest_binding = Some((self.expect_identifier(true)?, span));
                        // Rest binding must be the last binding
                        if !self.expect_token_type_and_skip(&[TokenType::RightParen], true) {
                            return None;
                        }
                        break;
                    }

                    // TODO: we can rewrite this to use the parse_sequence rule, but that will require
                    // rewriting the arrow AST transformation to recursively fold sequences
                    exprs.push(self.parse_yield()?);
                }
                self.new_level_stack.pop_level();

                // This is an arrow function if the next token is an arrow (`=>`)
                if self.expect_token_type_and_skip(&[TokenType::FatArrow], false) {
                    return self.parse_arrow_function_end(current.span, exprs, rest_binding.map(|v| v.0));
                }

                // If it's not an arrow function, then it is a group
                if let Some((sym, span)) = rest_binding {
                    self.create_error(Error::UnknownToken(Token {
                        span,
                        ty: TokenType::Identifier(sym),
                    }));
                    return None;
                }

                Expr::grouping(exprs)
            }
            TokenType::Async => {
                if self.expect_token_type_and_skip(&[TokenType::Function], false) {
                    self.parse_function(true).map(|(f, span)| Expr {
                        span,
                        kind: ExprKind::function(f),
                    })?
                } else if self.expect_token_type_and_skip(&[TokenType::LeftParen], true) {
                    let params = self.parse_parameter_list()?;
                    self.expect_token_type_and_skip(&[TokenType::FatArrow], true);
                    let statement = if self.expect_token_type_and_skip(&[TokenType::LeftBrace], false) {
                        self.advance_back();
                        self.parse_statement()?
                    } else {
                        let expr = self.parse_expression_no_comma()?;
                        Statement {
                            span: expr.span,
                            kind: StatementKind::Return(ReturnStatement(expr)),
                        }
                    };

                    Expr {
                        span: current.span.to(statement.span),
                        kind: ExprKind::function(FunctionDeclaration::new(
                            None,
                            self.function_counter.inc(),
                            params,
                            vec![statement],
                            // FIXME: this isn't correct -- we're currently desugaring async closures
                            // as if they're simply async functions
                            FunctionKind::Function(Asyncness::Yes),
                            None,
                            None,
                        )),
                    }
                } else {
                    return None;
                }
            }
            TokenType::Function => self.parse_function(false).map(|(f, span)| Expr {
                span,
                kind: ExprKind::function(f),
            })?,
            TokenType::Class => {
                let class = self.parse_class()?;
                let rbrace = self.previous().unwrap().span;
                Expr {
                    span: current.span.to(rbrace),
                    kind: ExprKind::Class(class),
                }
            }
            TokenType::RegexLiteral { literal, flags } => {
                // Trim / prefix and suffix
                let full = self.interner.resolve(literal);
                let full = &full[1..full.len() - 1];
                let (nodes, flags) = match dash_regex::Parser::new(full.as_bytes()).parse_all().and_then(|node| {
                    self.interner
                        .resolve(flags)
                        .parse::<Flags>()
                        .map_err(Into::into)
                        .map(|flags| (node, flags))
                }) {
                    Ok((nodes, flags)) => (nodes, flags),
                    Err(err) => {
                        let tok = self.previous().unwrap().clone();
                        self.create_error(Error::RegexSyntaxError(tok, err));
                        return None;
                    }
                };
                Expr {
                    span: current.span,
                    kind: ExprKind::regex_literal(nodes, flags, literal),
                }
            }
            other if other.is_identifier() => {
                let expr = Expr {
                    span: current.span,
                    kind: ExprKind::identifier(other.as_identifier().unwrap()),
                };

                // If this identifier is followed by an arrow, this is an arrow function
                if self.expect_token_type_and_skip(&[TokenType::FatArrow], false) {
                    return self.parse_arrow_function_end(current.span, vec![expr], None);
                }

                expr
            }
            _ => {
                let cur = self.previous().cloned()?;
                self.create_error(Error::UnknownToken(cur));
                return None;
            }
        };

        Some(expr)
    }

    pub fn parse_function(&mut self, is_async: bool) -> Option<(FunctionDeclaration, Span)> {
        let is_generator = self.expect_token_type_and_skip(&[TokenType::Star], false);

        let ty = if is_generator {
            if is_async {
                let star_span = self.previous().unwrap().span;
                self.create_error(Error::Unimplemented(star_span, "async generator".into()));
                return None;
            }

            FunctionKind::Generator
        } else {
            FunctionKind::Function(is_async.into())
        };

        let name = {
            let ty = self.current()?.ty;
            if ty.is_identifier() {
                match self.next() {
                    Some(tok) => tok.ty.as_identifier(),
                    None => None,
                }
            } else {
                None
            }
        };

        if !self.expect_token_type_and_skip(&[TokenType::LeftParen], true) {
            return None;
        }

        let arguments = self.parse_parameter_list()?;

        // Parse type param
        let ty_seg = if self.expect_token_type_and_skip(&[TokenType::Colon], false) {
            Some(self.parse_type_segment()?)
        } else {
            None
        };

        if !self.expect_token_type_and_skip(&[TokenType::LeftBrace], true) {
            return None;
        }

        self.new_level_stack.add_level();

        let BlockStatement(statements) = self.parse_block()?;

        self.new_level_stack.pop_level().unwrap();

        let func_id = self.function_counter.inc();
        Some((
            FunctionDeclaration::new(name, func_id, arguments, statements, ty, ty_seg, None),
            self.previous()?.span,
        ))
    }

    /// Parses the end of an arrow functio, i.e. the expression, and transforms the preceding list of expressions
    /// into the arrow function equivalent.
    ///
    /// Arrow functions are ambiguous and share the same beginning as grouping operator, *and* identifiers,
    /// i.e. `a` can mean `a => 1`, or just `a`, and `(a, b)` can mean `(a, b) => 1` or `(a, b)`
    /// so this can only be called when we have consumed =>
    ///
    /// Calling this will turn all parameters, which were parsed as if they were part of the grouping operator
    /// into their arrow function parameter equivalent
    fn parse_arrow_function_end(
        &mut self,
        pre_span: Span,
        prec: Vec<Expr>,
        rest_binding: Option<Symbol>,
    ) -> Option<Expr> {
        let mut list = Vec::with_capacity(prec.len());

        // If it is arrow function, we need to convert everything to their arrow func equivalents
        for expr in prec {
            // TODO: this currently breaks with types in arrow functions
            // e.g. (a: number) => {}
            // we need to properly convert types here too

            let (ident, value) = match expr.kind {
                ExprKind::Literal(LiteralExpr::Identifier(ident)) => (ident, None),
                ExprKind::Assignment(AssignmentExpr {
                    left: AssignmentTarget::Expr(left),
                    right,
                    ..
                }) => (left.kind.as_identifier()?, Some(*right)),
                _ => {
                    self.create_error(Error::Unimplemented(
                        expr.span,
                        "only assignment and identifier expressions are supported as in closure parameter recovery"
                            .into(),
                    ));
                    return None;
                }
            };

            list.push((Parameter::Identifier(ident), value, None));
        }

        if let Some(rest_binding) = rest_binding {
            list.push((Parameter::Spread(rest_binding), None, None));
        }

        let is_statement = self.expect_token_type_and_skip(&[TokenType::LeftBrace], false);

        let body = if is_statement {
            // Go one back ( to the `{` ), so that the next statement is parsed as a block containing all statements
            self.advance_back();

            self.parse_statement()?
        } else {
            let lo_span = self.current()?.span;
            let expr = self.parse_yield()?;
            let hi_span = self.previous()?.span;
            Statement {
                kind: StatementKind::Return(ReturnStatement(expr)),
                span: lo_span.to(hi_span),
            }
        };

        let func_id = self.function_counter.inc();
        Some(Expr {
            span: pre_span.to(body.span),
            kind: ExprKind::function(FunctionDeclaration::new(
                None,
                func_id,
                list,
                vec![body],
                FunctionKind::Arrow,
                None,
                None,
            )),
        })
    }
}
