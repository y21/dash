use crate::parser::expr::LiteralExpr;

use super::{
    expr::{Expr, UnaryExpr},
    statement::{
        BlockStatement, Catch, ForLoop, FunctionDeclaration, IfStatement, ImportKind,
        ReturnStatement, SpecifierKind, Statement, TryCatch, VariableDeclaration,
        VariableDeclarationKind, WhileLoop,
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

        if !self.errors.is_empty() {
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
            TokenType::If => self.if_statement(true).map(Statement::If),
            TokenType::Function => self.function().map(Statement::Function),
            TokenType::LeftBrace => self.block().map(Statement::Block),
            TokenType::While => self.while_loop().map(Statement::While),
            TokenType::Try => self.try_block().map(Statement::Try),
            TokenType::Throw => self.throw().map(Statement::Throw),
            TokenType::Return => self.return_statement().map(Statement::Return),
            TokenType::For => self.for_loop().map(Statement::For),
            TokenType::Import => self.import().map(Statement::Import),
            TokenType::Continue => Some(Statement::Continue),
            TokenType::Break => Some(Statement::Break),
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

    pub fn import(&mut self) -> Option<ImportKind<'a>> {
        // `import` followed by ( is considered a dynamic import
        let is_dynamic = self.expect_and_skip(&[TokenType::LeftParen], false);
        if is_dynamic {
            let specifier = self.expression()?;
            self.expect_and_skip(&[TokenType::RightParen], true);
            return Some(ImportKind::Dynamic(specifier));
        }

        // `import` followed by a `*` imports all exported values
        let is_import_all = self.expect_and_skip(&[TokenType::Star], false);
        if is_import_all {
            self.expect_and_skip(&[TokenType::Identifier], true);
            // TODO: enforce identifier be == b"as"
            let ident = self.next()?.full;
            self.expect_and_skip(&[TokenType::Identifier], true);
            // TODO: enforce identifier be == b"from"
            let specifier = self.next()?.full;
            return Some(ImportKind::AllAs(SpecifierKind::Ident(ident), specifier));
        }

        // `import` followed by an identifier is considered a default import
        if let Some(default_import_ident) = self.next().map(|tok| tok.full) {
            self.expect_and_skip(&[TokenType::Identifier], true);
            // TODO: enforce identifier be == b"from"
            let specifier = self.next()?.full;
            return Some(ImportKind::DefaultAs(
                SpecifierKind::Ident(default_import_ident),
                specifier,
            ));
        }

        None
    }

    pub fn throw(&mut self) -> Option<Expr<'a>> {
        self.expression()
    }

    pub fn try_block(&mut self) -> Option<TryCatch<'a>> {
        let try_ = self.statement()?;

        self.expect_and_skip(&[TokenType::Catch], true);

        let capture_ident = if self.expect_and_skip(&[TokenType::LeftParen], false) {
            let ident = self.next()?.full;
            self.expect_and_skip(&[TokenType::RightParen], true);
            Some(ident)
        } else {
            None
        };

        let catch = self.statement()?;

        // TODO: finally

        Some(TryCatch::new(try_, Catch::new(catch, capture_ident), None))
    }

    pub fn return_statement(&mut self) -> Option<ReturnStatement<'a>> {
        let expr = self.expression()?;
        Some(ReturnStatement(expr))
    }

    pub fn for_loop(&mut self) -> Option<ForLoop<'a>> {
        self.expect_and_skip(&[TokenType::LeftParen], true);

        let init = if self.expect_and_skip(&[TokenType::Semicolon], false) {
            None
        } else {
            self.statement()
        };

        let cond = if self.expect_and_skip(&[TokenType::Semicolon], false) {
            None
        } else {
            let expr = self.expression();
            self.expect_and_skip(&[TokenType::Semicolon], false);
            expr
        };

        let finalizer = if self.expect_and_skip(&[TokenType::RightParen], false) {
            None
        } else {
            let expr = self.expression();
            self.expect_and_skip(&[TokenType::RightParen], false);
            expr
        };

        let body = self.statement()?;

        Some(ForLoop::new(init, cond, finalizer, body))
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

    pub fn if_statement(&mut self, parse_else: bool) -> Option<IfStatement<'a>> {
        if !self.expect_and_skip(&[TokenType::LeftParen], true) {
            return None;
        }

        let condition = self.expression()?;

        if !self.expect_and_skip(&[TokenType::RightParen], true) {
            return None;
        }

        let then = self.statement()?;

        let mut branches = Vec::new();
        let mut el: Option<Box<Statement>> = None;

        if parse_else {
            while self.expect_and_skip(&[TokenType::Else], false) {
                let is_if = self.expect_and_skip(&[TokenType::If], false);

                if is_if {
                    let if_statement = self.if_statement(false)?;
                    branches.push(if_statement);
                } else {
                    el = Some(Box::new(self.statement()?));
                    break;
                }
            }
        }

        Some(IfStatement::new(condition, then, branches, el))
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
            |s| Self::shift(s),
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

    pub fn shift(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(
            |s| Self::term(s),
            &[
                TokenType::LeftShift,
                TokenType::RightShift,
                TokenType::UnsignedRightShift,
            ],
        )
    }

    pub fn term(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::factor(s), &[TokenType::Plus, TokenType::Minus])
    }

    pub fn factor(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(
            |s| Self::pow(s),
            &[TokenType::Star, TokenType::Slash, TokenType::Remainder],
        )
    }

    pub fn pow(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::unary(s), &[TokenType::Exponentiation])
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

            if [TokenType::Increment, TokenType::Decrement].contains(&operator) {
                let operator = if operator == TokenType::Increment {
                    TokenType::PrefixIncrement
                } else {
                    TokenType::PrefixDecrement
                };
                Some(Expr::assignment(rval, Expr::number_literal(1f64), operator))
            } else {
                Some(Expr::Unary(UnaryExpr::new(operator, rval)))
            }
        } else {
            self.postfix()
        }
    }

    pub fn postfix(&mut self) -> Option<Expr<'a>> {
        let expr = self.field_access()?;
        if self.expect_and_skip(&[TokenType::Increment, TokenType::Decrement], false) {
            let operator = self.previous()?.ty;
            return Some(Expr::Postfix((operator, Box::new(expr))));
        }
        Some(expr)
    }

    pub fn field_access(&mut self) -> Option<Expr<'a>> {
        if self.expect_and_skip(&[TokenType::New], false) {
            let mut rval = self.field_access()?;
            if let Expr::Call(fc) = &mut rval {
                fc.constructor_call = true;
            } else {
                todo!()
            };

            return Some(rval);
        }

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

                    expr = Expr::function_call(expr, arguments, false);
                }
                TokenType::Dot => {
                    let property = Expr::Literal(LiteralExpr::Identifier(self.next()?.full));
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
            TokenType::Semicolon => Expr::undefined_literal(),
            TokenType::FalseLit => Expr::bool_literal(false),
            TokenType::TrueLit => Expr::bool_literal(true),
            TokenType::NullLit => Expr::null_literal(),
            TokenType::UndefinedLit => Expr::undefined_literal(),
            TokenType::Identifier => Expr::identifier(full),
            TokenType::String => Expr::string_literal(full),
            TokenType::LeftSquareBrace => {
                let mut items = Vec::new();
                while !self.expect_and_skip(&[TokenType::RightSquareBrace], false) {
                    self.expect_and_skip(&[TokenType::Comma], false);
                    items.push(self.expression()?);
                }
                Expr::Array(items)
            }
            TokenType::LeftBrace => {
                let mut items = Vec::new();
                while !self.expect_and_skip(&[TokenType::RightBrace], false) {
                    self.expect_and_skip(&[TokenType::Comma], false);
                    let key = self.next()?.full;

                    // TODO: support property shorthand, e.g. { test } where test is a var in scope
                    self.expect_and_skip(&[TokenType::Colon], true);
                    let value = self.expression()?;
                    items.push((key, value));
                }
                Expr::Object(items)
            }
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
            TokenType::Function => Expr::Function(self.function()?),
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
