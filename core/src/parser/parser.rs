use super::{
    expr::{Expr, UnaryExpr},
    statement::{
        BlockStatement, FunctionDeclaration, IfStatement, ReturnStatement, Statement,
        VariableDeclaration, VariableDeclarationKind, WhileLoop,
    },
    token::{Error, ErrorKind, Token, TokenType, ASSIGNMENT_TYPES},
};

pub struct Parser<'a> {
    tokens: Box<[Token<'a>]>,
    errors: Vec<Error<'a>>,
    error_sync: bool,
    idx: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens: tokens.into_boxed_slice(),
            errors: Vec::new(),
            error_sync: false,
            idx: 0,
        }
    }

    pub fn parse(&mut self) -> Option<Statement<'a>> {
        self.statement()
    }

    pub fn parse_all(mut self) -> Result<Vec<Statement<'a>>, Vec<Error<'a>>> {
        let mut stmts = Vec::new();

        while !self.is_eof() {
            if let Some(stmt) = self.parse() {
                stmts.push(stmt);
            }
        }

        if self.errors.len() > 0 {
            Err(self.errors)
        } else {
            Ok(stmts)
        }
    }

    // Statement rules
    pub fn statement(&mut self) -> Option<Statement<'a>> {
        self.error_sync = false;
        let stmt = match self.next()?.ty {
            TokenType::Let | TokenType::Const | TokenType::Var => {
                self.variable().map(Statement::Variable)
            }
            TokenType::If => self.if_statement().map(Statement::If),
            TokenType::Function => self.function().map(Statement::Function),
            TokenType::LeftBrace => self.block().map(Statement::Block),
            TokenType::While => self.while_loop().map(Statement::While),
            TokenType::Return => self.return_statement().map(Statement::Return),
            _ => {
                // We've skipped the current character because of the statement cases that skip the current token
                // So we go back, as the skipped token belongs to this expression
                self.advance_back();
                Some(Statement::Expression(self.expression()?))
            }
        };

        self.expect_and_skip(&[TokenType::Semicolon], false);

        stmt
    }

    pub fn return_statement(&mut self) -> Option<ReturnStatement<'a>> {
        let expr = self.expression()?;
        Some(ReturnStatement(expr))
    }

    pub fn while_loop(&mut self) -> Option<WhileLoop<'a>> {
        if !self.expect_and_skip(&[TokenType::LeftParen], true) {
            return None;
        }

        let condition = self.expression()?;

        if !self.expect_and_skip(&[TokenType::RightParen], true) {
            return None;
        }

        let body = self.statement()?;

        Some(WhileLoop::new(condition, body))
    }

    pub fn function(&mut self) -> Option<FunctionDeclaration<'a>> {
        let name = self.next()?.full;

        if !self.expect_and_skip(&[TokenType::LeftParen], true) {
            return None;
        }

        let arguments = self.argument_list().unwrap();

        if !self.expect_and_skip(&[TokenType::LeftBrace], true) {
            return None;
        }

        let statements = self.block().unwrap().0;

        Some(FunctionDeclaration::new(name, arguments, statements))
    }

    pub fn argument_list(&mut self) -> Option<Vec<&'a [u8]>> {
        let mut arguments = Vec::new();

        while !self.expect_and_skip(&[TokenType::RightParen], false) {
            let tok = self.next()?;

            match tok.ty {
                TokenType::Identifier => arguments.push(tok.full),
                TokenType::Comma => continue,
                _ => todo!(), // TODO: handle
            };
        }

        Some(arguments)
    }

    pub fn block(&mut self) -> Option<BlockStatement<'a>> {
        let mut stmts = Vec::new();
        while !self.expect_and_skip(&[TokenType::RightBrace], false) {
            if self.is_eof() {
                return None;
            }

            if let Some(stmt) = self.statement() {
                stmts.push(stmt);
            }
        }
        Some(BlockStatement(stmts))
    }

    pub fn variable(&mut self) -> Option<VariableDeclaration<'a>> {
        let kind: VariableDeclarationKind = self.previous()?.ty.into();

        let name = self.next()?.full;

        // If the next char is `=`, we assume this declaration has a value
        let has_value = self.expect_and_skip(&[TokenType::Assignment], false);

        if !has_value {
            return Some(VariableDeclaration::new(name, kind, None));
        }

        let value = self.expression()?;

        return Some(VariableDeclaration::new(name, kind, Some(value)));
    }

    pub fn if_statement(&mut self) -> Option<IfStatement<'a>> {
        if !self.expect_and_skip(&[TokenType::LeftParen], true) {
            return None;
        }

        let condition = self.expression()?;

        if !self.expect_and_skip(&[TokenType::RightParen], true) {
            return None;
        }

        let then = self.statement()?;

        // TODO: else

        Some(IfStatement::new(condition, then, None))
    }

    // Expression rules
    pub fn expression(&mut self) -> Option<Expr<'a>> {
        self.sequence()
    }

    pub fn sequence(&mut self) -> Option<Expr<'a>> {
        let expr = self._yield()?;

        // TODO: this is ambiguous and causes problems when we're calling a function with multiple params
        /* while self.expect_and_skip(&[TokenType::Comma], false) {
            let rhs = self._yield()?;
            expr = Expr::Sequence((Box::new(expr), Box::new(rhs)));
        } */

        Some(expr)
    }

    pub fn _yield(&mut self) -> Option<Expr<'a>> {
        if self.expect_and_skip(&[TokenType::Yield], false) {
            return Some(Expr::Unary(UnaryExpr::new(
                TokenType::Yield,
                self._yield()?,
            )));
        }

        self.assignment()
    }

    pub fn assignment(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::ternary(s), ASSIGNMENT_TYPES)
    }

    pub fn ternary(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.nullish_coalescing()?;

        while self.expect_and_skip(&[TokenType::Conditional], false) {
            let then_branch = self.ternary()?;
            if !self.expect_and_skip(&[TokenType::Colon], true) {
                return None;
            }
            let else_branch = self.ternary()?;
            expr = Expr::conditional(expr, then_branch, else_branch);
        }

        Some(expr)
    }

    pub fn nullish_coalescing(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::logical_or(s), &[TokenType::NullishCoalescing])
    }

    pub fn logical_or(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::logical_and(s), &[TokenType::LogicalOr])
    }

    pub fn logical_and(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::bitwise_or(s), &[TokenType::LogicalAnd])
    }

    pub fn bitwise_or(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::bitwise_xor(s), &[TokenType::BitwiseOr])
    }

    pub fn bitwise_xor(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::bitwise_and(s), &[TokenType::BitwiseXor])
    }

    pub fn bitwise_and(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::equality(s), &[TokenType::BitwiseAnd])
    }

    pub fn equality(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(
            |s| Self::comparison(s),
            &[
                TokenType::Inequality,
                TokenType::Equality,
                TokenType::StrictEquality,
                TokenType::StrictInequality,
            ],
        )
    }

    pub fn comparison(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(
            |s| Self::term(s),
            &[
                TokenType::Greater,
                TokenType::Less,
                TokenType::GreaterEqual,
                TokenType::LessEqual,
                TokenType::In,
                TokenType::Instanceof,
            ],
        )
    }

    pub fn term(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::factor(s), &[TokenType::Plus, TokenType::Minus])
    }

    pub fn factor(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(
            |s| Self::unary(s),
            &[TokenType::Star, TokenType::Slash, TokenType::Remainder],
        )
    }

    pub fn unary(&mut self) -> Option<Expr<'a>> {
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
            let rval = self.unary()?;
            Some(Expr::Unary(UnaryExpr::new(operator, rval)))
        } else {
            self.postfix()
        }
    }

    pub fn postfix(&mut self) -> Option<Expr<'a>> {
        let expr = self.field_access()?;
        if self.expect_and_skip(&[TokenType::Increment, TokenType::Decrement], false) {
            let operator = self.previous()?.ty;
            // TODO: this is not true
            // `x++` is not the same as `x += 1`; it needs to return the old number
            return Some(Expr::assignment(expr, Expr::number_literal(1f64), operator));
        }
        Some(expr)
    }

    pub fn field_access(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.primary()?;

        while self.expect_and_skip(
            &[
                TokenType::LeftParen,
                TokenType::Dot,
                TokenType::LeftSquareBrace,
            ],
            false,
        ) {
            let previous = self.previous()?.ty;
            match previous {
                TokenType::LeftParen => {
                    let mut arguments = Vec::new();

                    while !self.expect_and_skip(&[TokenType::RightParen], false) {
                        self.expect_and_skip(&[TokenType::Comma], false);
                        arguments.push(self.expression()?);
                    }

                    expr = Expr::function_call(expr, arguments);
                }
                TokenType::Dot => {
                    let property = self.primary()?;
                    expr = Expr::property_access(false, expr, property);
                }
                TokenType::LeftSquareBrace => {
                    let property = self.expression()?;
                    self.expect_and_skip(&[TokenType::RightSquareBrace], false);
                    expr = Expr::property_access(true, expr, property);
                }
                _ => unreachable!(),
            }
        }

        Some(expr)
    }

    pub fn primary(&mut self) -> Option<Expr<'a>> {
        let (ty, full) = {
            let cur = self.current()?;
            (cur.ty, cur.full)
        };

        self.advance();

        let expr = match ty {
            TokenType::FalseLit => Expr::bool_literal(false),
            TokenType::TrueLit => Expr::bool_literal(true),
            TokenType::NullLit => Expr::null_literal(),
            TokenType::UndefinedLit => Expr::undefined_literal(),
            TokenType::Identifier => Expr::identifier(full),
            TokenType::String => Expr::string_literal(full),
            TokenType::Number => {
                Expr::number_literal(std::str::from_utf8(full).unwrap().parse::<f64>().unwrap())
            }
            TokenType::LeftParen => {
                let expr = self.expression()?;
                if !self.expect_and_skip(&[TokenType::RightParen], true) {
                    return None;
                }
                Expr::grouping(expr)
            }
            _ => {
                let cur = self.previous().cloned()?;
                self.create_error(ErrorKind::UnknownToken(cur));
                return None;
            }
        };

        Some(expr)
    }

    // Helper functions

    pub fn read_infix_expression<F>(
        &mut self,
        lower: F,
        tokens: &'static [TokenType],
    ) -> Option<Expr<'a>>
    where
        F: Fn(&mut Self) -> Option<Expr<'a>>,
    {
        let mut expr = lower(self)?;

        while self.expect_and_skip(tokens, false) {
            let operator = self.previous()?.ty;
            let rval = lower(self)?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    pub fn is_eof(&self) -> bool {
        self.idx >= self.tokens.len()
    }

    pub fn expect_and_skip(&mut self, ty: &'static [TokenType], emit_error: bool) -> bool {
        let current = match self.current() {
            Some(k) => *k,
            None => return false,
        };

        let ok = ty.iter().any(|ty| ty.eq(&current.ty));

        if ok {
            self.advance();
        } else if emit_error {
            self.create_error(ErrorKind::UnexpectedTokenMultiple(current, ty));
        }

        ok
    }

    pub fn create_error(&mut self, kind: ErrorKind<'a>) {
        if !self.error_sync {
            self.errors.push(Error { kind });
            self.error_sync = true;
        }
    }

    pub fn advance(&mut self) {
        self.idx += 1;
    }

    pub fn advance_back(&mut self) {
        self.idx -= 1;
    }

    pub fn current(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.idx)
    }

    pub fn previous(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.idx - 1)
    }

    pub fn next(&mut self) -> Option<&Token<'a>> {
        self.advance();
        self.previous()
    }
}
