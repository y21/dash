use crate::parser::{
    expr::LiteralExpr,
    statement::{ClassProperty, ForOfLoop},
};

use super::{
    consteval::Eval,
    expr::{Expr, UnaryExpr},
    lexer::{self, Lexer},
    statement::{
        BlockStatement, Catch, Class, ClassMember, ClassMemberKind, ExportKind, ForLoop,
        FunctionDeclaration, FunctionKind, IfStatement, ImportKind, Loop, ReturnStatement,
        SpecifierKind, Statement, TryCatch, VariableBinding, VariableDeclaration,
        VariableDeclarationKind, WhileLoop,
    },
    token::{Error, ErrorKind, Token, TokenType, ASSIGNMENT_TYPES, VARIABLE_TYPES},
};

/// A JavaScript source code parser
pub struct Parser<'a> {
    tokens: Box<[Token<'a>]>,
    errors: Vec<Error<'a>>,
    error_sync: bool,
    idx: usize,
    input: &'a [u8],
}

impl<'a> Parser<'a> {
    /// Convenience function for creating a parser from a source code string
    /// This function is equivalent to first sending the input string through a [Lexer]
    /// and then creating a parser using the tokens
    pub fn from_str(input: &'a str) -> Result<Self, Vec<lexer::Error>> {
        let tokens = Lexer::new(input).scan_all()?;
        Ok(Self::new(input, tokens))
    }

    /// Creates a new parser from tokens generated by a [Lexer]
    pub fn new(input: &'a str, tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens: tokens.into_boxed_slice(),
            errors: Vec::new(),
            error_sync: false,
            idx: 0,
            input: input.as_bytes(),
        }
    }

    /// Attempts to parse a single statement
    /// If an error occurs, `None` is returned and an error is added to
    /// an internal errors vec
    /// Usually `parse_all` is used to attempt to parse the entire program
    /// and get any existing errors
    pub fn parse(&mut self) -> Option<Statement<'a>> {
        self.statement()
    }

    /// Iteratively parses every token and returns an AST, or a vector of errors
    ///
    /// The AST will be folded by passing true as the `fold` parameter.
    pub fn parse_all(mut self, fold: bool) -> Result<Vec<Statement<'a>>, Vec<Error<'a>>> {
        let mut stmts = Vec::new();

        while !self.is_eof() {
            if let Some(stmt) = self.parse() {
                stmts.push(stmt);
            }
        }

        if !self.errors.is_empty() {
            Err(self.errors)
        } else {
            let len = stmts.len();

            if fold && len > 1 {
                stmts[..len - 1].fold();
            }

            Ok(stmts)
        }
    }

    // Statement rules
    fn statement(&mut self) -> Option<Statement<'a>> {
        self.error_sync = false;
        let stmt = match self.next()?.ty {
            TokenType::Let | TokenType::Const | TokenType::Var => {
                self.variable().map(Statement::Variable)
            }
            TokenType::If => self.if_statement(true).map(Statement::If),
            TokenType::Function => self.function().map(Statement::Function),
            TokenType::LeftBrace => self.block().map(Statement::Block),
            TokenType::While => self.while_loop().map(Statement::Loop),
            TokenType::Try => self.try_block().map(Statement::Try),
            TokenType::Throw => self.throw().map(Statement::Throw),
            TokenType::Return => self.return_statement().map(Statement::Return),
            TokenType::For => self.for_loop().map(Statement::Loop),
            TokenType::Import => self.import().map(Statement::Import),
            TokenType::Export => self.export().map(Statement::Export),
            TokenType::Class => self.class().map(Statement::Class),
            TokenType::Continue => Some(Statement::Continue),
            TokenType::Break => Some(Statement::Break),
            TokenType::Debugger => Some(Statement::Debugger),
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

    fn class(&mut self) -> Option<Class<'a>> {
        let name = if self.expect_and_skip(&[TokenType::Identifier], false) {
            self.previous().map(|x| x.full)
        } else {
            None
        };

        let extends = if self.expect_and_skip(&[TokenType::Extends], false) {
            Some(self.expression()?)
        } else {
            None
        };

        self.expect_and_skip(&[TokenType::LeftBrace], true);

        let mut members = Vec::new();

        // Start parsing class members
        while !self.expect_and_skip(&[TokenType::RightBrace], false) {
            let is_static = self.expect_and_skip(&[TokenType::Static], false);
            let is_private = self.expect_and_skip(&[TokenType::Hash], false);

            let name = self.next()?.full;

            let is_method = self.expect_and_skip(&[TokenType::LeftParen], false);

            if is_method {
                let arguments = self.argument_list()?;
                let body = self.statement()?;

                let func = FunctionDeclaration::new(
                    Some(name),
                    arguments,
                    vec![body],
                    FunctionKind::Function,
                );

                members.push(ClassMember {
                    private: is_private,
                    static_: is_static,
                    kind: ClassMemberKind::Method(func),
                });
            } else {
                let kind = self.next()?.ty;

                let value = match kind {
                    TokenType::Assignment => Some(self.expression()?),
                    TokenType::Semicolon => None,
                    _ => {
                        // We don't know what this token is, so we assume the user left out the semicolon and meant to declare a property
                        // For this reason we need to go back so we don't throw away the token we just read
                        self.advance_back();
                        None
                    }
                };

                self.expect_and_skip(&[TokenType::Semicolon], false);

                members.push(ClassMember {
                    private: is_private,
                    static_: is_static,
                    kind: ClassMemberKind::Property(ClassProperty { name, value }),
                });
            };
        }

        Some(Class {
            name,
            extends,
            members,
        })
    }

    fn export(&mut self) -> Option<ExportKind<'a>> {
        let is_named = self.expect_and_skip(&[TokenType::LeftBrace], false);

        if is_named {
            let mut names = Vec::new();
            while !self.expect_and_skip(&[TokenType::RightBrace], false) {
                let name = self.next()?.full;
                names.push(name);
                self.expect_and_skip(&[TokenType::Comma], false);
            }
            return Some(ExportKind::Named(names));
        }

        let current = self.current()?;

        if VARIABLE_TYPES.contains(&current.ty) {
            let mut variables = Vec::new();
            while self.expect_and_skip(VARIABLE_TYPES, false) {
                let variable = self.variable()?;
                variables.push(variable);
                self.expect_and_skip(&[TokenType::Comma], false);
            }
            return Some(ExportKind::NamedVar(variables));
        }

        // We emit an error because this is the last possible way to create
        // an export statement
        if self.expect_and_skip(&[TokenType::Default], true) {
            let expr = self.expression()?;
            return Some(ExportKind::Default(expr));
        }

        unreachable!()
    }

    fn import(&mut self) -> Option<ImportKind<'a>> {
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

    fn throw(&mut self) -> Option<Expr<'a>> {
        self.expression()
    }

    fn try_block(&mut self) -> Option<TryCatch<'a>> {
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

    fn return_statement(&mut self) -> Option<ReturnStatement<'a>> {
        let expr = self.expression()?;
        Some(ReturnStatement(expr))
    }

    fn for_loop(&mut self) -> Option<Loop<'a>> {
        self.expect_and_skip(&[TokenType::LeftParen], true);

        let init = if self.expect_and_skip(&[TokenType::Semicolon], false) {
            None
        } else {
            let is_binding = self.expect_and_skip(VARIABLE_TYPES, false);

            if is_binding {
                let binding = self.variable_binding()?;
                let is_of = self.expect_and_skip(&[TokenType::Of], false);

                if is_of {
                    let expr = self.expression()?;

                    self.expect_and_skip(&[TokenType::RightParen], true);

                    let body = Box::new(self.statement()?);

                    return Some(Loop::ForOf(ForOfLoop {
                        binding,
                        expr,
                        body,
                    }));
                } else {
                    let value = self.variable_value();

                    self.expect_and_skip(&[TokenType::Semicolon], false);

                    Some(Statement::Variable(VariableDeclaration::new(
                        binding, value,
                    )))
                }
            } else {
                self.statement()
            }
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

        Some(ForLoop::new(init, cond, finalizer, body).into())
    }

    fn while_loop(&mut self) -> Option<Loop<'a>> {
        if !self.expect_and_skip(&[TokenType::LeftParen], true) {
            return None;
        }

        let condition = self.expression()?;

        if !self.expect_and_skip(&[TokenType::RightParen], true) {
            return None;
        }

        let body = self.statement()?;

        Some(WhileLoop::new(condition, body).into())
    }

    fn function(&mut self) -> Option<FunctionDeclaration<'a>> {
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

        let arguments = self.argument_list().unwrap();

        if !self.expect_and_skip(&[TokenType::LeftBrace], true) {
            return None;
        }

        let statements = self.block().unwrap().0;

        Some(FunctionDeclaration::new(name, arguments, statements, ty))
    }

    fn argument_list(&mut self) -> Option<Vec<&'a [u8]>> {
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

    fn block(&mut self) -> Option<BlockStatement<'a>> {
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

    fn variable_binding(&mut self) -> Option<VariableBinding<'a>> {
        let kind: VariableDeclarationKind = self.previous()?.ty.into();

        let name = self.next()?.full;

        Some(VariableBinding { kind, name })
    }

    fn variable_value(&mut self) -> Option<Expr<'a>> {
        // If the next char is `=`, we assume this declaration has a value
        let has_value = self.expect_and_skip(&[TokenType::Assignment], false);

        if !has_value {
            return None;
        }

        self.expression()
    }

    fn variable(&mut self) -> Option<VariableDeclaration<'a>> {
        let binding = self.variable_binding()?;

        let value = self.variable_value();

        Some(VariableDeclaration::new(binding, value))
    }

    fn if_statement(&mut self, parse_else: bool) -> Option<IfStatement<'a>> {
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
    fn expression(&mut self) -> Option<Expr<'a>> {
        self.sequence()
    }

    fn sequence(&mut self) -> Option<Expr<'a>> {
        let expr = self._yield()?;

        // TODO: this is ambiguous and causes problems when we're calling a function with multiple params
        /* while self.expect_and_skip(&[TokenType::Comma], false) {
            let rhs = self._yield()?;
            expr = Expr::Sequence((Box::new(expr), Box::new(rhs)));
        } */

        Some(expr)
    }

    fn _yield(&mut self) -> Option<Expr<'a>> {
        if self.expect_and_skip(&[TokenType::Yield], false) {
            return Some(Expr::Unary(UnaryExpr::new(
                TokenType::Yield,
                self._yield()?,
            )));
        }

        self.assignment()
    }

    fn assignment(&mut self) -> Option<Expr<'a>> {
        let mut expr = self.ternary()?;

        while self.expect_and_skip(ASSIGNMENT_TYPES, false) {
            let operator = self.previous()?.ty;
            let rval = self.ternary()?;
            expr = Expr::assignment(expr, rval, operator);
        }

        Some(expr)
    }

    fn ternary(&mut self) -> Option<Expr<'a>> {
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

    fn nullish_coalescing(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::logical_or(s), &[TokenType::NullishCoalescing])
    }

    fn logical_or(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::logical_and(s), &[TokenType::LogicalOr])
    }

    fn logical_and(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::bitwise_or(s), &[TokenType::LogicalAnd])
    }

    fn bitwise_or(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::bitwise_xor(s), &[TokenType::BitwiseOr])
    }

    fn bitwise_xor(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::bitwise_and(s), &[TokenType::BitwiseXor])
    }

    fn bitwise_and(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::equality(s), &[TokenType::BitwiseAnd])
    }

    fn equality(&mut self) -> Option<Expr<'a>> {
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

    fn comparison(&mut self) -> Option<Expr<'a>> {
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

    fn shift(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(
            |s| Self::term(s),
            &[
                TokenType::LeftShift,
                TokenType::RightShift,
                TokenType::UnsignedRightShift,
            ],
        )
    }

    fn term(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::factor(s), &[TokenType::Plus, TokenType::Minus])
    }

    fn factor(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(
            |s| Self::pow(s),
            &[TokenType::Star, TokenType::Slash, TokenType::Remainder],
        )
    }

    fn pow(&mut self) -> Option<Expr<'a>> {
        self.read_infix_expression(|s| Self::unary(s), &[TokenType::Exponentiation])
    }

    fn unary(&mut self) -> Option<Expr<'a>> {
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

    fn postfix(&mut self) -> Option<Expr<'a>> {
        let expr = self.field_access()?;
        if self.expect_and_skip(&[TokenType::Increment, TokenType::Decrement], false) {
            let operator = self.previous()?.ty;
            return Some(Expr::Postfix((operator, Box::new(expr))));
        }
        Some(expr)
    }

    fn field_access(&mut self) -> Option<Expr<'a>> {
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

    fn primary(&mut self) -> Option<Expr<'a>> {
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
            TokenType::Identifier => {
                let expr = Expr::identifier(full);

                // If this identifier is followed by an arrow, this is an arrow function
                if self.expect_and_skip(&[TokenType::Arrow], false) {
                    return self.arrow_function(vec![expr]).map(Expr::Function);
                }

                expr
            }
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
            TokenType::NumberDec => {
                Expr::number_literal(std::str::from_utf8(full).unwrap().parse::<f64>().unwrap())
            }
            TokenType::NumberHex => self
                .parse_prefixed_number_literal(full, 16)
                .map(Expr::number_literal)?,
            TokenType::NumberBin => self
                .parse_prefixed_number_literal(full, 2)
                .map(Expr::number_literal)?,
            TokenType::NumberOct => self
                .parse_prefixed_number_literal(full, 8)
                .map(Expr::number_literal)?,
            TokenType::LeftParen => {
                if self.expect_and_skip(&[TokenType::RightParen], false) {
                    // () MUST be followed by an arrow. Empty groups are not valid syntax
                    if !self.expect_and_skip(&[TokenType::Arrow], true) {
                        return None;
                    }

                    return self.arrow_function(Vec::new()).map(Expr::Function);
                }

                let mut exprs = vec![self.expression()?];

                while !self.expect_and_skip(&[TokenType::RightParen], false) {
                    self.expect_and_skip(&[TokenType::Comma], false);
                    exprs.push(self.expression()?);
                }

                // This is an arrow function if the next token is an arrow (`=>`)
                if self.expect_and_skip(&[TokenType::Arrow], false) {
                    return self.arrow_function(exprs).map(Expr::Function);
                }

                // If it's not an arrow function, then it is a group
                Expr::grouping(exprs)
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

    /// Parses a prefixed number literal (0x, 0o, 0b) and returns the number
    pub fn parse_prefixed_number_literal(&mut self, full: &[u8], radix: u32) -> Option<f64> {
        let src = std::str::from_utf8(&full[2..]).unwrap();
        match u64::from_str_radix(src, radix).map(|x| x as f64) {
            Ok(f) => Some(f),
            Err(e) => {
                self.create_error(ErrorKind::ParseIntError(self.previous().copied()?, e));
                None
            }
        }
    }

    /// Parses an arrow function
    ///
    /// Arrow functions are ambiguous and share the same beginning as grouping operator,
    /// so this can only be called when we having consumed =>
    /// Calling this will turn all parameters, which were parsed as if they were part of the grouping operator
    /// into their arrow function parameter equivalent
    fn arrow_function(&mut self, parameter_list: Vec<Expr<'a>>) -> Option<FunctionDeclaration<'a>> {
        let mut list = Vec::with_capacity(parameter_list.len());

        // If it is arrow function, we need to convert everything to their arrow func equivalents
        for expr in parameter_list {
            list.push(expr.to_identifier()?);
        }

        let body = match self.statement()? {
            Statement::Expression(expr) => Statement::Return(ReturnStatement(expr)),
            other => other,
        };

        // If the last token is a semicolon, we want to go back to it
        if self.previous().map(|x| x.ty) == Some(TokenType::Semicolon) {
            self.advance_back();
        }

        Some(FunctionDeclaration::new(
            None,
            list,
            vec![body],
            FunctionKind::Arrow,
        ))
    }

    fn read_infix_expression<F>(
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

    fn is_eof(&self) -> bool {
        self.idx >= self.tokens.len()
    }

    fn expect_and_skip(&mut self, ty: &'static [TokenType], emit_error: bool) -> bool {
        let current = match self.current() {
            Some(k) => *k,
            None => {
                if emit_error {
                    self.create_error(ErrorKind::UnexpectedEof);
                }
                return false;
            }
        };

        let ok = ty.iter().any(|ty| ty.eq(&current.ty));

        if ok {
            self.advance();
        } else if emit_error {
            self.create_error(ErrorKind::UnexpectedTokenMultiple(current, ty));
        }

        ok
    }

    fn create_error(&mut self, kind: ErrorKind<'a>) {
        if !self.error_sync {
            self.errors.push(Error {
                kind,
                source: self.input,
            });
            self.error_sync = true;
        }
    }

    fn advance(&mut self) {
        self.idx += 1;
    }

    fn advance_back(&mut self) {
        self.idx -= 1;
    }

    fn current(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.idx)
    }

    fn previous(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.idx - 1)
    }

    fn next(&mut self) -> Option<&Token<'a>> {
        self.advance();
        self.previous()
    }
}
