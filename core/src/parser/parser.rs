use super::{
    expr::{Expr, UnaryExpr},
    statement::{
        BlockStatement, FunctionDeclaration, IfStatement, Statement, VariableDeclaration,
        VariableDeclarationKind,
    },
    token::{Token, TokenType, ASSIGNMENT_TYPES},
};

pub struct Parser<'a> {
    tokens: Box<[Token<'a>]>,
    idx: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens: tokens.into_boxed_slice(),
            idx: 0,
        }
    }

    pub fn parse(&mut self) -> Option<Statement<'a>> {
        self.statement()
    }

    pub fn parse_all(mut self) -> Vec<Statement<'a>> {
        let mut stmts = Vec::new();
        while !self.is_eof() {
            if let Some(stmt) = self.parse() {
                stmts.push(stmt);
            }
        }
        stmts
    }

    // Statement rules
    pub fn statement(&mut self) -> Option<Statement<'a>> {
        if self.expect_and_skip(&[TokenType::Let, TokenType::Const, TokenType::Var]) {
            // Parse variable declaration
            let stmt = self.variable().map(Statement::Variable);
            self.expect_and_skip(&[TokenType::Semicolon]);
            return stmt;
        } else if self.expect_and_skip(&[TokenType::If]) {
            let stmt = self.if_statement().map(Statement::If);
            self.expect_and_skip(&[TokenType::Semicolon]);
            return stmt;
        } else if self.expect_and_skip(&[TokenType::Function]) {
            let stmt = self.function().map(Statement::Function);
            self.expect_and_skip(&[TokenType::Semicolon]);
            return stmt;
        } else if self.expect_and_skip(&[TokenType::LeftBrace]) {
            let stmt = self.block().map(Statement::Block);
            self.expect_and_skip(&[TokenType::Semicolon]);
            return stmt;
        }

        Some(Statement::Expression(self.expression()?))
    }

    pub fn function(&mut self) -> Option<FunctionDeclaration<'a>> {
        let name = self.next()?.full;

        // TODO: error if this isnt true
        self.expect_and_skip(&[TokenType::LeftParen]);

        let arguments = self.argument_list().unwrap();

        // TODO: same as above
        self.expect_and_skip(&[TokenType::RightParen]);

        self.expect_and_skip(&[TokenType::LeftBrace]);

        let statements = self.block().unwrap().0;

        Some(FunctionDeclaration::new(name, arguments, statements))
    }

    pub fn argument_list(&mut self) -> Option<Vec<&'a [u8]>> {
        let mut arguments = Vec::new();
        while !self.expect_and_skip(&[TokenType::RightParen]) {
            let tok = self.previous()?;

            if tok.ty == TokenType::Identifier {
                arguments.push(tok.full);
            } else {
                // ??? handle this case
                todo!()
            }
        }
        Some(arguments)
    }

    pub fn block(&mut self) -> Option<BlockStatement<'a>> {
        let mut stmts = Vec::new();
        while !self.expect_and_skip(&[TokenType::RightBrace]) {
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
        let has_value = self.expect_and_skip(&[TokenType::Assignment]);

        if !has_value {
            return Some(VariableDeclaration::new(name, kind, None));
        }

        let value = self.expression()?;

        return Some(VariableDeclaration::new(name, kind, Some(value)));
    }

    pub fn if_statement(&mut self) -> Option<IfStatement<'a>> {
        if !self.expect_and_skip(&[TokenType::LeftParen]) {
            return None;
        }

        let condition = self.expression()?;

        if !self.expect_and_skip(&[TokenType::RightParen]) {
            return None;
        }

        let then = self.statement()?;

        // TODO: else

        Some(IfStatement::new(condition, then, None))
    }

    // Expression rules
    pub fn expression(&mut self) -> Option<Expr<'a>> {
        self._yield()
    }

    pub fn _yield(&mut self) -> Option<Expr<'a>> {
        if self.expect_and_skip(&[TokenType::Yield]) {
            return Some(Expr::Unary(UnaryExpr::new(
                TokenType::Yield,
                self._yield()?,
            )));
        }

        self.assignment()
    }

    pub fn assignment(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::equality(s), ASSIGNMENT_TYPES)
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
            ],
        )
    }

    pub fn term(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::factor(s), &[TokenType::Plus, TokenType::Minus])
    }

    pub fn factor(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::unary(s), &[TokenType::Star, TokenType::Slash])
    }

    pub fn unary(&mut self) -> Option<Expr<'a>> {
        if self.expect_and_skip(&[TokenType::LogicalNot, TokenType::Minus]) {
            let operator = self.previous()?.ty;
            let rval = self.unary()?;
            Some(Expr::Unary(UnaryExpr::new(operator, rval)))
        } else {
            self.primary()
        }
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
            TokenType::LeftParen => Expr::grouping(self.expression()?), // TODO: make sure there's a ) after the expression
            _ => {
                // TODO: this should return an error expr(?)
                unimplemented!()
            }
        };

        Some(expr)
    }

    // Helper functions

    pub fn read_infix_expression<F>(&mut self, lower: F, tokens: &[TokenType]) -> Option<Expr<'a>>
    where
        F: Fn(&mut Self) -> Option<Expr<'a>>,
    {
        let mut expr = lower(self)?;

        while self.expect_and_skip(tokens) {
            let operator = self.previous()?.ty;
            let rval = lower(self)?;
            expr = Expr::binary(expr, rval, operator);
        }

        Some(expr)
    }

    pub fn is_eof(&self) -> bool {
        self.idx >= self.tokens.len()
    }

    pub fn expect_and_skip(&mut self, ty: &[TokenType]) -> bool {
        let current = match self.current() {
            Some(k) => k,
            None => return false,
        };

        let ok = ty.iter().any(|ty| ty.eq(&current.ty));

        if ok {
            self.advance();
        }

        ok
    }

    pub fn advance(&mut self) {
        self.idx += 1;
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
