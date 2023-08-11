use dash_middle::lexer::token::TokenType;
use dash_middle::lexer::token::ASSIGNMENT_TYPES;
use dash_middle::parser::error::ErrorKind;
use dash_middle::parser::expr::ArrayMemberKind;
use dash_middle::parser::expr::CallArgumentKind;
use dash_middle::parser::expr::Expr;
use dash_middle::parser::expr::ObjectMemberKind;
use dash_middle::parser::statement::BlockStatement;
use dash_middle::parser::statement::FunctionDeclaration;
use dash_middle::parser::statement::FunctionKind;
use dash_middle::parser::statement::Parameter;
use dash_middle::parser::statement::ReturnStatement;
use dash_middle::parser::statement::Statement;

use crate::Parser;

impl<'a, 'interner> Parser<'a, 'interner> {
    pub fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_sequence()
    }

    fn parse_sequence(&mut self) -> Option<Expr> {
        // TODO: sequence is currently ambiguous and we can't parse it
        // i.e. x(1, 2) is ambiguous because it could mean x((1, 2)) or x(1, 2)
        self.parse_yield()
    }

    fn parse_yield(&mut self) -> Option<Expr> {
        if self.expect_token_type_and_skip(&[TokenType::Yield], false) {
            let right = self.parse_yield()?;
            return Some(Expr::unary(TokenType::Yield, right));
        }

        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<Expr> {
        let mut expr = self.parse_ternary()?;

        if self.expect_token_type_and_skip(ASSIGNMENT_TYPES, false) {
            let operator = self.previous()?.ty;
            let rval = self.parse_assignment()?;
            expr = Expr::assignment(expr, rval, operator);
        }

        Some(expr)
    }

    fn parse_ternary(&mut self) -> Option<Expr> {
        let mut expr = self.parse_nullish_coalescing()?;

        while self.expect_token_type_and_skip(&[TokenType::Conditional], false) {
            let then_branch = self.parse_ternary()?;
            if !self.expect_token_type_and_skip(&[TokenType::Colon], true) {
                return None;
            }
            let else_branch = self.parse_ternary()?;
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
            let operator = self.previous()?.ty;
            let rval = self.parse_unary()?;

            if [TokenType::Increment, TokenType::Decrement].contains(&operator) {
                Some(Expr::prefix(operator, rval))
            } else {
                Some(Expr::unary(operator, rval))
            }
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Option<Expr> {
        let expr = self.parse_field_access()?;
        if self.expect_token_type_and_skip(&[TokenType::Increment, TokenType::Decrement], false) {
            let operator = self.previous()?.ty;
            return Some(Expr::postfix(operator, expr));
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

                    // TODO: refactor to `parse_expr_list`
                    while !self.expect_token_type_and_skip(&[TokenType::RightParen], false) {
                        self.expect_token_type_and_skip(&[TokenType::Comma], false);

                        if let Some(spread) = self.parse_spread_operator(false) {
                            arguments.push(CallArgumentKind::Spread(spread));
                        } else {
                            arguments.push(CallArgumentKind::Normal(self.parse_expression()?));
                        }
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
                    let property = Expr::identifier(self.expect_identifier_or_reserved_kw(true)?);
                    expr = Expr::property_access(false, expr, property);
                }
                TokenType::LeftSquareBrace => {
                    let property = self.parse_expression()?;
                    self.expect_token_type_and_skip(&[TokenType::RightSquareBrace], false);
                    expr = Expr::property_access(true, expr, property);
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
                    self.create_error(ErrorKind::IncompleteSpread(token));
                    return None;
                }
            }
            self.parse_expression()
        } else {
            None
        }
    }

    fn parse_primary_expr(&mut self) -> Option<Expr> {
        let current = self.current()?.clone();

        self.advance();

        let expr = match current.ty {
            // removed to resolve #58
            // TokenType::Semicolon => Expr::undefined_literal(),
            TokenType::TemplateLiteral(sym) => {
                let mut left = Expr::string_literal(sym);
                while !self.is_eof() {
                    if self.expect_token_type_and_skip(&[TokenType::Dollar], false) {
                        self.expect_token_type_and_skip(&[TokenType::LeftBrace], true);
                        let right = self.parse_expression()?;
                        self.expect_token_type_and_skip(&[TokenType::RightBrace], true);
                        left = Expr::binary(left, right, TokenType::Plus);
                    } else if let Some(sym) = self.expect_template_literal(false) {
                        let right = Expr::string_literal(sym);
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
            TokenType::String(sym) => Expr::string_literal(sym),
            TokenType::EmptySquareBrace => Expr::array_literal(Vec::new()),
            TokenType::LeftSquareBrace => {
                let mut items = Vec::new();
                while !self.expect_token_type_and_skip(&[TokenType::RightSquareBrace], false) {
                    self.expect_token_type_and_skip(&[TokenType::Comma], false);
                    if let Some(spread) = self.parse_spread_operator(false) {
                        items.push(ArrayMemberKind::Spread(spread));
                    } else {
                        items.push(ArrayMemberKind::Item(self.parse_expression()?));
                    }
                }
                Expr::array_literal(items)
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
                        // TODO: this breaks object literals with a normal property named "get"
                        TokenType::Get => ObjectMemberKind::Getter(self.expect_identifier_or_reserved_kw(true)?),
                        TokenType::Set => ObjectMemberKind::Setter(self.expect_identifier_or_reserved_kw(true)?),
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
                                    self.create_error(ErrorKind::IncompleteSpread(token));
                                    return None;
                                }
                            }
                            ObjectMemberKind::Spread
                        }
                        other => {
                            if let Some(ident) = other.as_identifier_or_reserved_kw() {
                                ObjectMemberKind::Static(ident)
                            } else {
                                self.create_error(ErrorKind::UnexpectedToken(token, TokenType::DUMMY_IDENTIFIER));
                                return None;
                            }
                        }
                    };

                    match key {
                        ObjectMemberKind::Spread => {
                            items.push((key, self.parse_expression()?));
                        }
                        ObjectMemberKind::Dynamic(..) | ObjectMemberKind::Static(..) => {
                            if self.expect_token_type_and_skip(&[TokenType::Colon], false) {
                                // Normal property.
                                let value = self.parse_expression()?;
                                items.push((key, value));
                            } else if self.expect_token_type_and_skip(&[TokenType::LeftParen], false) {
                                // Method.
                                let parameters = self.parse_parameter_list()?;
                                self.expect_token_type_and_skip(&[TokenType::LeftBrace], true);
                                let body = self.parse_block()?;
                                let id = self.function_counter.advance();
                                items.push((
                                    key,
                                    Expr::function(FunctionDeclaration::new(
                                        None,
                                        id,
                                        parameters,
                                        body.0,
                                        FunctionKind::Function,
                                        false,
                                        None,
                                    )),
                                ));
                            } else {
                                match key {
                                    ObjectMemberKind::Static(name) => items.push((key, Expr::identifier(name))),
                                    ObjectMemberKind::Dynamic(..) => {
                                        self.create_error(ErrorKind::UnexpectedToken(token, TokenType::Colon));
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

                            self.expect_token_type_and_skip(&[TokenType::LeftBrace], true);
                            let BlockStatement(stmts) = self.parse_block()?;

                            // Desugar to function
                            let func_id = self.function_counter.advance();
                            let fun = FunctionDeclaration::new(
                                None,
                                func_id,
                                params,
                                stmts,
                                FunctionKind::Function,
                                false,
                                None,
                            );
                            items.push((key, Expr::function(fun)));
                        }
                    }
                }
                Expr::object_literal(items)
            }
            // TODO: this unwrap is not safe
            TokenType::NumberDec(sym) => Expr::number_literal(self.interner.resolve(sym).parse::<f64>().unwrap()),
            TokenType::NumberHex(sym) => self.parse_prefixed_number_literal(sym, 16).map(Expr::number_literal)?,
            TokenType::NumberBin(sym) => self.parse_prefixed_number_literal(sym, 2).map(Expr::number_literal)?,
            TokenType::NumberOct(sym) => self.parse_prefixed_number_literal(sym, 8).map(Expr::number_literal)?,
            TokenType::LeftParen => {
                if self.expect_token_type_and_skip(&[TokenType::RightParen], false) {
                    // () MUST be followed by an arrow. Empty groups are not valid syntax
                    if !self.expect_token_type_and_skip(&[TokenType::FatArrow], true) {
                        return None;
                    }

                    return self.parse_arrow_function_end(Vec::new()).map(Expr::function);
                }

                self.new_level_stack.add_level();
                let mut exprs = vec![self.parse_expression()?];

                while !self.expect_token_type_and_skip(&[TokenType::RightParen], false) {
                    self.expect_token_type_and_skip(&[TokenType::Comma], false);
                    exprs.push(self.parse_expression()?);
                }
                self.new_level_stack.pop_level();

                // This is an arrow function if the next token is an arrow (`=>`)
                if self.expect_token_type_and_skip(&[TokenType::FatArrow], false) {
                    return self.parse_arrow_function_end(exprs).map(Expr::function);
                }

                // If it's not an arrow function, then it is a group
                Expr::grouping(exprs)
            }
            TokenType::Async => {
                // TODO: if it isn't followed by function, check if followed by ( for arrow functions
                // or if not, parse it as an identifier
                if !self.expect_token_type_and_skip(&[TokenType::Function], true) {
                    return None;
                }
                Expr::function(self.parse_function(true)?)
            }
            TokenType::Function => Expr::function(self.parse_function(false)?),
            TokenType::RegexLiteral(sym) => {
                // Trim / prefix and suffix
                let full = self.interner.resolve(sym);
                let full = &full[1..full.len() - 1];
                let nodes = match dash_regex::Parser::new(full.as_bytes()).parse_all() {
                    Ok(nodes) => nodes,
                    Err(err) => {
                        let tok = self.current().unwrap().clone();
                        self.create_error(ErrorKind::RegexSyntaxError(tok, err));
                        return None;
                    }
                };
                Expr::regex_literal(nodes, sym)
            }
            other if other.is_identifier() => {
                let expr = Expr::identifier(other.as_identifier().unwrap());

                // If this identifier is followed by an arrow, this is an arrow function
                if self.expect_token_type_and_skip(&[TokenType::FatArrow], false) {
                    return self.parse_arrow_function_end(vec![expr]).map(Expr::function);
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

    pub fn parse_function(&mut self, is_async: bool) -> Option<FunctionDeclaration> {
        let is_generator = self.expect_token_type_and_skip(&[TokenType::Star], false);

        let ty = if is_generator {
            FunctionKind::Generator
        } else {
            FunctionKind::Function
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

        let func_id = self.function_counter.advance();
        Some(FunctionDeclaration::new(
            name, func_id, arguments, statements, ty, is_async, ty_seg,
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
    fn parse_arrow_function_end(&mut self, prec: Vec<Expr>) -> Option<FunctionDeclaration> {
        let mut list = Vec::with_capacity(prec.len());

        // If it is arrow function, we need to convert everything to their arrow func equivalents
        for expr in prec {
            // TODO: this currently breaks with types in arrow functions
            // e.g. (a: number) => {}
            // we need to properly convert types here too

            // TODO2: handle parameter default values
            list.push((Parameter::Identifier(expr.as_identifier()?), None, None));
        }

        let is_statement = self.expect_token_type_and_skip(&[TokenType::LeftBrace], false);

        let body = if is_statement {
            // Go one back ( to the `{` ), so that the next statement is parsed as a block containing all statements
            self.advance_back();

            self.parse_statement()?
        } else {
            Statement::Return(ReturnStatement(self.parse_expression()?))
        };

        let func_id = self.function_counter.advance();
        Some(FunctionDeclaration::new(
            None,
            func_id,
            list,
            vec![body],
            FunctionKind::Arrow,
            false,
            None,
        ))
    }
}
