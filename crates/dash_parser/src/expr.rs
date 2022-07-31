use dash_middle::lexer::token::TokenType;
use dash_middle::lexer::token::ASSIGNMENT_TYPES;
use dash_middle::parser::error::ErrorKind;
use dash_middle::parser::expr::ArrayLiteral;
use dash_middle::parser::expr::Expr;
use dash_middle::parser::expr::ObjectLiteral;
use dash_middle::parser::expr::ObjectMemberKind;
use dash_middle::parser::expr::UnaryExpr;
use dash_middle::parser::statement::BlockStatement;
use dash_middle::parser::statement::FunctionDeclaration;
use dash_middle::parser::statement::FunctionKind;
use dash_middle::parser::statement::Parameter;
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::Statement;

use crate::stmt::StatementParser;
use crate::Parser;

pub trait ExpressionParser<'a> {
    fn parse_expression(&mut self) -> Option<Expr<'a>>;
    fn parse_function(&mut self) -> Option<FunctionDeclaration<'a>>;
    fn parse_sequence(&mut self) -> Option<Expr<'a>>;
    fn parse_yield(&mut self) -> Option<Expr<'a>>;
    fn parse_assignment(&mut self) -> Option<Expr<'a>>;
    fn parse_ternary(&mut self) -> Option<Expr<'a>>;
    fn parse_nullish_coalescing(&mut self) -> Option<Expr<'a>>;
    fn parse_logical_or(&mut self) -> Option<Expr<'a>>;
    fn parse_logical_and(&mut self) -> Option<Expr<'a>>;
    fn parse_bitwise_or(&mut self) -> Option<Expr<'a>>;
    fn parse_bitwise_and(&mut self) -> Option<Expr<'a>>;
    fn parse_bitwise_xor(&mut self) -> Option<Expr<'a>>;
    fn parse_equality(&mut self) -> Option<Expr<'a>>;
    fn parse_comparison(&mut self) -> Option<Expr<'a>>;
    fn parse_bitwise_shift(&mut self) -> Option<Expr<'a>>;
    fn parse_term(&mut self) -> Option<Expr<'a>>;
    fn parse_factor(&mut self) -> Option<Expr<'a>>;
    fn parse_pow(&mut self) -> Option<Expr<'a>>;
    fn parse_unary(&mut self) -> Option<Expr<'a>>;
    fn parse_postfix(&mut self) -> Option<Expr<'a>>;
    fn parse_field_access(&mut self) -> Option<Expr<'a>>;
    fn parse_primary_expr(&mut self) -> Option<Expr<'a>>;
    /// Parses the end of an arrow functio, i.e. the expression, and transforms the preceding list of expressions
    /// into the arrow function equivalent.
    ///
    /// Arrow functions are ambiguous and share the same beginning as grouping operator, *and* identifiers,
    /// i.e. `a` can mean `a => 1`, or just `a`, and `(a, b)` can mean `(a, b) => 1` or `(a, b)`
    /// so this can only be called when we have consumed =>
    ///
    /// Calling this will turn all parameters, which were parsed as if they were part of the grouping operator
    /// into their arrow function parameter equivalent
    fn parse_arrow_function_end(&mut self, prec: Vec<Expr<'a>>) -> Option<FunctionDeclaration<'a>>;
}

impl<'a> ExpressionParser<'a> for Parser<'a> {
    fn parse_expression(&mut self) -> Option<Expr<'a>> {
        self.parse_sequence()
    }

    fn parse_sequence(&mut self) -> Option<Expr<'a>> {
        // TODO: sequence is currently ambiguous and we can't parse it
        // i.e. x(1, 2) is ambiguous because it could mean x((1, 2)) or x(1, 2)
        self.parse_yield()
    }

    fn parse_yield(&mut self) -> Option<Expr<'a>> {
        if self.expect_and_skip(&[TokenType::Yield], false) {
            let right = self.parse_yield()?;
            return Some(Expr::Unary(UnaryExpr::new(TokenType::Yield, right)));
        }

        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_ternary()?;

        if self.expect_and_skip(ASSIGNMENT_TYPES, false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_assignment()?;
            expr = Expr::assignment(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_ternary(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_nullish_coalescing()?;

        while self.expect_and_skip(&[TokenType::Conditional], false) {
            let then_branch = self.parse_ternary()?;
            if !self.expect_and_skip(&[TokenType::Colon], true) {
                return None;
            }
            let else_branch = self.parse_ternary()?;
            expr = Expr::conditional(expr, then_branch, else_branch);
        }

        Some(expr)
    }

    fn parse_nullish_coalescing(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_logical_or()?;

        while self.expect_and_skip(&[TokenType::NullishCoalescing], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_logical_or()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_logical_or(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_logical_and()?;

        while self.expect_and_skip(&[TokenType::LogicalOr], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_logical_and()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_logical_and(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_bitwise_or()?;

        while self.expect_and_skip(&[TokenType::LogicalAnd], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_bitwise_or()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_bitwise_or(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_bitwise_xor()?;

        while self.expect_and_skip(&[TokenType::BitwiseOr], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_bitwise_xor()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_bitwise_xor(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_bitwise_and()?;

        while self.expect_and_skip(&[TokenType::BitwiseXor], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_bitwise_and()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_bitwise_and(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_equality()?;

        while self.expect_and_skip(&[TokenType::BitwiseAnd], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_equality()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_equality(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_comparison()?;

        while self.expect_and_skip(
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

    fn parse_comparison(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_bitwise_shift()?;

        while self.expect_and_skip(
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

    fn parse_bitwise_shift(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_term()?;

        while self.expect_and_skip(
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

    fn parse_term(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_factor()?;

        while self.expect_and_skip(&[TokenType::Plus, TokenType::Minus], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_factor()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_factor(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_pow()?;

        while self.expect_and_skip(&[TokenType::Star, TokenType::Slash, TokenType::Remainder], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_pow()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_pow(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.parse_unary()?;

        while self.expect_and_skip(&[TokenType::Exponentiation], false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_unary()?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_unary(&mut self) -> Option<Expr<'a>> {
        if self.expect_and_skip(
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
            let operator = self.previous()?.ty;
            let rval = self.parse_unary()?;

            if [TokenType::Increment, TokenType::Decrement].contains(&operator) {
                let operator = if operator == TokenType::Increment {
                    TokenType::AdditionAssignment
                } else {
                    TokenType::SubtractionAssignment
                };

                // Desugar ++foo and --foo directly to foo += 1 and foo -= 1
                Some(Expr::assignment(rval, Expr::number_literal(1f64), operator))
            } else {
                Some(Expr::Unary(UnaryExpr::new(operator, rval)))
            }
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Option<Expr<'a>> {
        let expr = self.parse_field_access()?;
        if self.expect_and_skip(&[TokenType::Increment, TokenType::Decrement], false) {
            let operator = self.previous()?.ty;
            return Some(Expr::Postfix((operator, Box::new(expr))));
        }
        Some(expr)
    }

    fn parse_field_access(&mut self) -> Option<Expr<'a>> {
        if self.expect_and_skip(&[TokenType::New], false) {
            let mut rval = self.parse_field_access()?;
            if let Expr::Call(fc) = &mut rval {
                fc.constructor_call = true;
            } else {
                self.create_error(ErrorKind::UnexpectedToken(self.previous()?.clone(), TokenType::New));
                return None;
            };

            return Some(rval);
        }

        let mut expr = self.parse_primary_expr()?;

        while self.expect_and_skip(
            &[TokenType::LeftParen, TokenType::Dot, TokenType::LeftSquareBrace],
            false,
        ) {
            let previous = self.previous()?.ty;

            match previous {
                TokenType::LeftParen => {
                    let mut arguments = Vec::new();

                    // TODO: refactor to `parse_expr_list`
                    while !self.expect_and_skip(&[TokenType::RightParen], false) {
                        self.expect_and_skip(&[TokenType::Comma], false);
                        arguments.push(self.parse_expression()?);
                    }

                    expr = Expr::function_call(expr, arguments, false);
                }
                TokenType::Dot => {
                    let property = Expr::identifier(self.next()?.full);
                    expr = Expr::property_access(false, expr, property);
                }
                TokenType::LeftSquareBrace => {
                    let property = self.parse_expression()?;
                    self.expect_and_skip(&[TokenType::RightSquareBrace], false);
                    expr = Expr::property_access(true, expr, property);
                }
                _ => unreachable!(),
            }
        }

        Some(expr)
    }

    fn parse_primary_expr(&mut self) -> Option<Expr<'a>> {
        let (ty, full) = {
            let cur = self.current()?;
            (cur.ty, cur.full)
        };

        self.advance();

        let expr = match ty {
            // TODO: ; shouldnt be a valid expression
            TokenType::Semicolon => Expr::undefined_literal(),
            TokenType::FalseLit => Expr::bool_literal(false),
            TokenType::TrueLit => Expr::bool_literal(true),
            TokenType::NullLit => Expr::null_literal(),
            TokenType::UndefinedLit => Expr::undefined_literal(),
            TokenType::Identifier => {
                let expr = Expr::identifier(full);

                // If this identifier is followed by an arrow, this is an arrow function
                if self.expect_and_skip(&[TokenType::FatArrow], false) {
                    return self.parse_arrow_function_end(vec![expr]).map(Expr::Function);
                }

                expr
            }
            TokenType::String => Expr::string_literal(full),
            TokenType::EmptySquareBrace => Expr::Array(ArrayLiteral(Vec::new())),
            TokenType::LeftSquareBrace => {
                let mut items = Vec::new();
                while !self.expect_and_skip(&[TokenType::RightSquareBrace], false) {
                    self.expect_and_skip(&[TokenType::Comma], false);
                    items.push(self.parse_expression()?);
                }
                Expr::Array(ArrayLiteral(items))
            }
            TokenType::LeftBrace => {
                let mut items = Vec::new();
                while !self.expect_and_skip(&[TokenType::RightBrace], false) {
                    self.expect_and_skip(&[TokenType::Comma], false);
                    let token = self.next()?.clone();
                    let key = match token.ty {
                        // TODO: this breaks object literals with a normal property named "get"
                        TokenType::Get => ObjectMemberKind::Getter(self.next()?.full),
                        TokenType::Set => ObjectMemberKind::Setter(self.next()?.full),
                        TokenType::LeftSquareBrace => {
                            let t = self.parse_expression()?;
                            let o = ObjectMemberKind::Dynamic(t);
                            self.expect_and_skip(&[TokenType::RightSquareBrace], true);
                            o
                        }
                        _ => ObjectMemberKind::Static(token.full),
                    };

                    match key {
                        ObjectMemberKind::Dynamic(..) | ObjectMemberKind::Static(..) => {
                            // TODO: support property shorthand, e.g. { test } where test is a var in scope
                            self.expect_and_skip(&[TokenType::Colon], true);
                            let value = self.parse_expression()?;
                            items.push((key, value));
                        }
                        ObjectMemberKind::Getter(..) | ObjectMemberKind::Setter(..) => {
                            self.expect_and_skip(&[TokenType::LeftParen], true);
                            let params = self.parse_parameter_list()?;

                            // Make sure parameter count is correct
                            match key {
                                ObjectMemberKind::Setter(..) => {
                                    if params.len() != 1 {
                                        self.create_error(ErrorKind::InvalidAccessorParams {
                                            token,
                                            expect: 1,
                                            got: params.len(),
                                        });
                                        return None;
                                    }
                                }
                                ObjectMemberKind::Getter(..) => {
                                    if !params.is_empty() {
                                        self.create_error(ErrorKind::InvalidAccessorParams {
                                            token,
                                            expect: 0,
                                            got: params.len(),
                                        });
                                        return None;
                                    }
                                }
                                _ => unreachable!(),
                            }

                            self.expect_and_skip(&[TokenType::LeftBrace], true);
                            let BlockStatement(stmts) = self.parse_block()?;

                            // Desugar to function
                            let fun = FunctionDeclaration::new(None, params, stmts, FunctionKind::Function);
                            items.push((key, Expr::Function(fun)));
                        }
                    }
                }
                Expr::Object(ObjectLiteral(items))
            }
            // TODO: this unwrap is not safe
            TokenType::NumberDec => Expr::number_literal(full.parse::<f64>().unwrap()),
            TokenType::NumberHex => self.parse_prefixed_number_literal(full, 16).map(Expr::number_literal)?,
            TokenType::NumberBin => self.parse_prefixed_number_literal(full, 2).map(Expr::number_literal)?,
            TokenType::NumberOct => self.parse_prefixed_number_literal(full, 8).map(Expr::number_literal)?,
            TokenType::LeftParen => {
                if self.expect_and_skip(&[TokenType::RightParen], false) {
                    // () MUST be followed by an arrow. Empty groups are not valid syntax
                    if !self.expect_and_skip(&[TokenType::FatArrow], true) {
                        return None;
                    }

                    return self.parse_arrow_function_end(Vec::new()).map(Expr::Function);
                }

                let mut exprs = vec![self.parse_expression()?];

                while !self.expect_and_skip(&[TokenType::RightParen], false) {
                    self.expect_and_skip(&[TokenType::Comma], false);
                    exprs.push(self.parse_expression()?);
                }

                // This is an arrow function if the next token is an arrow (`=>`)
                if self.expect_and_skip(&[TokenType::FatArrow], false) {
                    return self.parse_arrow_function_end(exprs).map(Expr::Function);
                }

                // If it's not an arrow function, then it is a group
                Expr::grouping(exprs)
            }
            TokenType::Function => Expr::Function(self.parse_function()?),
            _ => {
                let cur = self.previous().cloned()?;
                self.create_error(ErrorKind::UnknownToken(cur));
                return None;
            }
        };

        Some(expr)
    }

    fn parse_function(&mut self) -> Option<FunctionDeclaration<'a>> {
        let is_generator = self.expect_and_skip(&[TokenType::Star], false);

        let ty = if is_generator {
            FunctionKind::Generator
        } else {
            FunctionKind::Function
        };

        let name = {
            let ty = self.current()?.ty;
            if ty == TokenType::Identifier {
                self.next().map(|x| x.full)
            } else {
                None
            }
        };

        if !self.expect_and_skip(&[TokenType::LeftParen], true) {
            return None;
        }

        let arguments = self.parse_parameter_list()?;

        if !self.expect_and_skip(&[TokenType::LeftBrace], true) {
            return None;
        }

        let BlockStatement(statements) = self.parse_block()?;

        Some(FunctionDeclaration::new(name, arguments, statements, ty))
    }

    fn parse_arrow_function_end(&mut self, prec: Vec<Expr<'a>>) -> Option<FunctionDeclaration<'a>> {
        let mut list = Vec::with_capacity(prec.len());

        // If it is arrow function, we need to convert everything to their arrow func equivalents
        for expr in prec {
            // TODO: this currently breaks with types in arrow functions
            // e.g. (a: number) => {}
            // we need to properly convert types here too
            list.push((Parameter::Identifier(expr.as_identifier()?), None));
        }

        let is_statement = self.expect_and_skip(&[TokenType::LeftBrace], false);

        let body = if is_statement {
            // Go one back ( to the `{` ), so that the next statement is parsed as a block containing all statements
            self.advance_back();

            self.parse_statement()?
        } else {
            Statement::Return(ReturnStatement(self.parse_expression()?))
        };

        Some(FunctionDeclaration::new(None, list, vec![body], FunctionKind::Arrow))
    }
}
