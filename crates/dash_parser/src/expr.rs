use std::borrow::Cow;

use dash_middle::lexer::token::TokenType;
use dash_middle::lexer::token::ASSIGNMENT_TYPES;
use dash_middle::parser::error::ErrorKind;
use dash_middle::parser::expr::ArrayLiteral;
use dash_middle::parser::expr::Expr;
use dash_middle::parser::expr::LiteralExpr;
use dash_middle::parser::expr::ObjectLiteral;
use dash_middle::parser::expr::ObjectMemberKind;
use dash_middle::parser::expr::UnaryExpr;
use dash_middle::parser::statement::BlockStatement;
use dash_middle::parser::statement::FunctionDeclaration;
use dash_middle::parser::statement::FunctionKind;
use dash_middle::parser::statement::Parameter;
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::Statement;

use crate::must_borrow_lexeme;
use crate::stmt::StatementParser;
use crate::Parser;

pub trait ExpressionParser<'a> {
    fn parse_expression(&mut self) -> Option<Expr<'a>>;
    fn parse_function(&mut self, is_async: bool) -> Option<FunctionDeclaration<'a>>;
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
                Some(Expr::Prefix((operator, Box::new(rval))))
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
            self.new_level_stack
                .inc_level()
                .expect("Failed to increment `new` stack level");

            let rval = self.parse_field_access()?;

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

                    // End of function call.
                    let level = self.new_level_stack.cur_level().expect("Missing `new` level stack");
                    let is_constructor_call = level > 0;
                    if is_constructor_call {
                        self.new_level_stack.dec_level().expect("Missing `new` level stack");
                    }

                    expr = Expr::function_call(expr, arguments, is_constructor_call);
                }
                TokenType::Dot => {
                    let property = Expr::identifier(self.next()?.full.clone());
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
        let current = self.current()?.clone();

        self.advance();

        let expr = match current.ty {
            // TODO: ; shouldnt be a valid expression
            TokenType::Semicolon => Expr::undefined_literal(),
            TokenType::TemplateLiteral => {
                let mut left = Expr::string_literal(current.full);
                while !self.is_eof() {
                    if self.expect_and_skip(&[TokenType::Dollar], false) {
                        self.expect_and_skip(&[TokenType::LeftBrace], true);
                        let right = self.parse_expression()?;
                        self.expect_and_skip(&[TokenType::RightBrace], true);
                        left = Expr::binary(left, right, TokenType::Plus);
                    } else if self.expect_and_skip(&[TokenType::TemplateLiteral], false) {
                        let right = Expr::string_literal(self.previous()?.full.clone());
                        left = Expr::binary(left, right, TokenType::Plus);
                    } else {
                        break;
                    }
                }
                left
            }
            TokenType::FalseLit => Expr::bool_literal(false),
            TokenType::TrueLit => Expr::bool_literal(true),
            TokenType::NullLit => Expr::null_literal(),
            TokenType::UndefinedLit => Expr::undefined_literal(),
            TokenType::String => Expr::string_literal(current.full),
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
                        TokenType::Get => ObjectMemberKind::Getter(self.next()?.full.clone()),
                        TokenType::Set => ObjectMemberKind::Setter(self.next()?.full.clone()),
                        TokenType::LeftSquareBrace => {
                            let t = self.parse_expression()?;
                            let o = ObjectMemberKind::Dynamic(t);
                            self.expect_and_skip(&[TokenType::RightSquareBrace], true);
                            o
                        }
                        _ => ObjectMemberKind::Static(must_borrow_lexeme!(self, &token)?),
                    };

                    match key {
                        ObjectMemberKind::Dynamic(..) | ObjectMemberKind::Static(..) => {
                            let has_colon = self.expect_and_skip(&[TokenType::Colon], false);

                            if has_colon {
                                let value = self.parse_expression()?;
                                items.push((key, value));
                            } else {
                                match key {
                                    ObjectMemberKind::Static(name) => {
                                        items.push((key, Expr::identifier(Cow::Borrowed(name))))
                                    }
                                    ObjectMemberKind::Dynamic(..) => {
                                        self.create_error(ErrorKind::UnexpectedToken(token, TokenType::Colon));
                                        return None;
                                    }
                                    _ => unreachable!(),
                                }
                            }
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
                            let fun = FunctionDeclaration::new(None, params, stmts, FunctionKind::Function, false);
                            items.push((key, Expr::Function(fun)));
                        }
                    }
                }
                Expr::Object(ObjectLiteral(items))
            }
            // TODO: this unwrap is not safe
            TokenType::NumberDec => Expr::number_literal(current.full.parse::<f64>().unwrap()),
            TokenType::NumberHex => self
                .parse_prefixed_number_literal(&current.full, 16)
                .map(Expr::number_literal)?,
            TokenType::NumberBin => self
                .parse_prefixed_number_literal(&current.full, 2)
                .map(Expr::number_literal)?,
            TokenType::NumberOct => self
                .parse_prefixed_number_literal(&current.full, 8)
                .map(Expr::number_literal)?,
            TokenType::LeftParen => {
                if self.expect_and_skip(&[TokenType::RightParen], false) {
                    // () MUST be followed by an arrow. Empty groups are not valid syntax
                    if !self.expect_and_skip(&[TokenType::FatArrow], true) {
                        return None;
                    }

                    return self.parse_arrow_function_end(Vec::new()).map(Expr::Function);
                }

                self.new_level_stack.add_level();
                let mut exprs = vec![self.parse_expression()?];

                while !self.expect_and_skip(&[TokenType::RightParen], false) {
                    self.expect_and_skip(&[TokenType::Comma], false);
                    exprs.push(self.parse_expression()?);
                }
                self.new_level_stack.pop_level();

                // This is an arrow function if the next token is an arrow (`=>`)
                if self.expect_and_skip(&[TokenType::FatArrow], false) {
                    return self.parse_arrow_function_end(exprs).map(Expr::Function);
                }

                // If it's not an arrow function, then it is a group
                Expr::grouping(exprs)
            }
            TokenType::Async => {
                // TODO: if it isn't followed by function, check if followed by ( for arrow functions
                // or if not, parse it as an identifier
                if !self.expect_and_skip(&[TokenType::Function], true) {
                    return None;
                }
                Expr::Function(self.parse_function(true)?)
            }
            TokenType::Function => Expr::Function(self.parse_function(false)?),
            TokenType::RegexLiteral => {
                // Trim / prefix and suffix
                let full = must_borrow_lexeme!(self, &current)?;
                let full = &full[1..full.len() - 1];
                let nodes = match dash_regex::Parser::new(full.as_bytes()).parse_all() {
                    Ok(nodes) => nodes,
                    Err(err) => {
                        let tok = self.current().unwrap().clone();
                        self.create_error(ErrorKind::RegexSyntaxError(tok, err));
                        return None;
                    }
                };
                Expr::Literal(LiteralExpr::Regex(nodes, full))
            }
            other if other.is_identifier() => {
                let expr = Expr::identifier(current.full);

                // If this identifier is followed by an arrow, this is an arrow function
                if self.expect_and_skip(&[TokenType::FatArrow], false) {
                    return self.parse_arrow_function_end(vec![expr]).map(Expr::Function);
                }

                expr
            }
            _ => {
                let cur = self.previous().cloned()?;
                self.create_error(ErrorKind::UnknownToken(cur));
                return None;
            }
        };

        Some(expr)
    }

    fn parse_function(&mut self, is_async: bool) -> Option<FunctionDeclaration<'a>> {
        let is_generator = self.expect_and_skip(&[TokenType::Star], false);

        let ty = if is_generator {
            FunctionKind::Generator
        } else {
            FunctionKind::Function
        };

        let name = {
            let ty = self.current()?.ty;
            if ty.is_identifier() {
                match self.next() {
                    Some(tok) => Some(must_borrow_lexeme!(self, tok)?),
                    None => None,
                }
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

        self.new_level_stack.add_level();

        let BlockStatement(statements) = self.parse_block()?;

        self.new_level_stack.pop_level().unwrap();

        Some(FunctionDeclaration::new(name, arguments, statements, ty, is_async))
    }

    fn parse_arrow_function_end(&mut self, prec: Vec<Expr<'a>>) -> Option<FunctionDeclaration<'a>> {
        let mut list = Vec::with_capacity(prec.len());

        // If it is arrow function, we need to convert everything to their arrow func equivalents
        for expr in prec {
            // TODO: this currently breaks with types in arrow functions
            // e.g. (a: number) => {}
            // we need to properly convert types here too

            // TODO2: handle parameter default values
            list.push((Parameter::Identifier(expr.as_identifier()?), None, None));
        }

        let is_statement = self.expect_and_skip(&[TokenType::LeftBrace], false);

        let body = if is_statement {
            // Go one back ( to the `{` ), so that the next statement is parsed as a block containing all statements
            self.advance_back();

            self.parse_statement()?
        } else {
            Statement::Return(ReturnStatement(self.parse_expression()?))
        };

        Some(FunctionDeclaration::new(
            None,
            list,
            vec![body],
            FunctionKind::Arrow,
            false,
        ))
    }
}
