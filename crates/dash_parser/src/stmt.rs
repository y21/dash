use dash_middle::lexer::token::{TokenType, VARIABLE_TYPES};
use dash_middle::parser::error::Error;
use dash_middle::parser::expr::{Expr, ExprKind};
use dash_middle::parser::statement::{
    BlockStatement, Catch, Class, ClassMember, ClassMemberKind, ClassProperty, DoWhileLoop, ExportKind, ForInLoop,
    ForLoop, ForOfLoop, FunctionDeclaration, FunctionKind, IfStatement, ImportKind, Loop, Parameter, ReturnStatement,
    SpecifierKind, Statement, StatementKind, SwitchCase, SwitchStatement, TryCatch, VariableBinding,
    VariableDeclaration, VariableDeclarationKind, VariableDeclarationName, VariableDeclarations, WhileLoop,
};
use dash_middle::parser::types::TypeSegment;

use crate::Parser;

type ParameterList = Option<Vec<(Parameter, Option<Expr>, Option<TypeSegment>)>>;

impl<'a, 'interner> Parser<'a, 'interner> {
    pub fn parse_statement(&mut self) -> Option<Statement> {
        self.error_sync = false;
        let lo_span = self.current()?.span;
        let kind = match self.next()?.ty {
            TokenType::Let | TokenType::Const | TokenType::Var => self.parse_variable().map(StatementKind::Variable),
            TokenType::If => self.parse_if(true).map(StatementKind::If),
            TokenType::Function => self.parse_function(false).map(|(k, _)| StatementKind::Function(k)),
            TokenType::Async => {
                // async must be followed by function (todo: or async () => {})
                if !self.expect_token_type_and_skip(&[TokenType::Function], true) {
                    return None;
                }
                self.parse_function(true).map(|(k, _)| StatementKind::Function(k))
            }
            TokenType::LeftBrace => self.parse_block().map(StatementKind::Block),
            TokenType::While => self.parse_while_loop().map(StatementKind::Loop),
            TokenType::Do => self.parse_do_while_loop().map(StatementKind::Loop),
            TokenType::Try => self.parse_try().map(StatementKind::Try),
            TokenType::Throw => self.parse_throw().map(StatementKind::Throw),
            TokenType::Return => self.parse_return().map(StatementKind::Return),
            TokenType::For => self.parse_for_loop().map(StatementKind::Loop),
            TokenType::Import => self.parse_import().map(StatementKind::Import),
            TokenType::Export => self.parse_export().map(StatementKind::Export),
            TokenType::Class => self.parse_class().map(StatementKind::Class),
            TokenType::Switch => self.parse_switch().map(StatementKind::Switch),
            TokenType::Continue => Some(StatementKind::Continue),
            TokenType::Break => Some(StatementKind::Break),
            TokenType::Debugger => Some(StatementKind::Debugger),
            TokenType::Semicolon => Some(StatementKind::Empty),
            _ => {
                // We've skipped the current character because of the statement cases that skip the current token
                // So we go back, as the skipped token belongs to this expression
                self.advance_back();
                Some(StatementKind::Expression(self.parse_expression()?))
            }
        }?;

        let hi_span = self.previous().unwrap().span;

        self.expect_token_type_and_skip(&[TokenType::Semicolon], false);

        Some(Statement {
            kind,
            span: lo_span.to(hi_span),
        })
    }

    fn parse_class(&mut self) -> Option<Class> {
        let name = self.expect_identifier(false);

        let extends = if self.expect_token_type_and_skip(&[TokenType::Extends], false) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect_token_type_and_skip(&[TokenType::LeftBrace], true);

        let mut members = Vec::new();

        // Start parsing class members
        while !self.expect_token_type_and_skip(&[TokenType::RightBrace], false) {
            let is_static = self.expect_token_type_and_skip(&[TokenType::Static], false);
            let is_private = self.expect_token_type_and_skip(&[TokenType::Hash], false);

            let name = self.expect_identifier_or_reserved_kw(true)?;

            let is_method = self.expect_token_type_and_skip(&[TokenType::LeftParen], false);

            if is_method {
                let arguments = self.parse_parameter_list()?;

                // Parse type param
                // TODO: this should probably be part of parse_aprameter_list
                let ty_seg = if self.expect_token_type_and_skip(&[TokenType::Colon], false) {
                    Some(self.parse_type_segment()?)
                } else {
                    None
                };

                let body = self.parse_statement()?;

                let func_id = self.function_counter.advance();
                let func = FunctionDeclaration::new(
                    Some(name),
                    func_id,
                    arguments,
                    vec![body],
                    FunctionKind::Function,
                    false,
                    ty_seg,
                );

                members.push(ClassMember {
                    private: is_private,
                    static_: is_static,
                    kind: ClassMemberKind::Method(func),
                });
            } else {
                let kind = self.next()?.ty;

                let value = match kind {
                    TokenType::Assignment => Some(self.parse_expression()?),
                    TokenType::Semicolon => None,
                    _ => {
                        // We don't know what this token is, so we assume the user left out the semicolon and meant to declare a property
                        // For this reason we need to go back so we don't throw away the token we just read
                        self.advance_back();
                        None
                    }
                };

                self.expect_token_type_and_skip(&[TokenType::Semicolon], false);

                members.push(ClassMember {
                    private: is_private,
                    static_: is_static,
                    kind: ClassMemberKind::Property(ClassProperty { name, value }),
                });
            };
        }

        Some(Class { name, extends, members })
    }

    fn parse_export(&mut self) -> Option<ExportKind> {
        let is_named = self.expect_token_type_and_skip(&[TokenType::LeftBrace], false);

        if is_named {
            let mut names = Vec::new();
            while !self.expect_token_type_and_skip(&[TokenType::RightBrace], false) {
                let name = self.expect_identifier(true)?;
                names.push(name);
                self.expect_token_type_and_skip(&[TokenType::Comma], false);
            }
            return Some(ExportKind::Named(names));
        }

        let current = self.current()?;

        if current.ty.is_variable() {
            self.advance();
            let variables = self.parse_variable()?;

            return Some(ExportKind::NamedVar(variables));
        }

        // We emit an error because this is the last possible way to create
        // an export statement
        if self.expect_token_type_and_skip(&[TokenType::Default], true) {
            let expr = self.parse_expression()?;
            return Some(ExportKind::Default(expr));
        }

        None
    }

    fn parse_import(&mut self) -> Option<ImportKind> {
        // `import` followed by ( is considered a dynamic import
        let is_dynamic = self.expect_token_type_and_skip(&[TokenType::LeftParen], false);
        if is_dynamic {
            let specifier = self.parse_expression()?;
            self.expect_token_type_and_skip(&[TokenType::RightParen], true);
            return Some(ImportKind::Dynamic(specifier));
        }

        // `import` followed by a `*` imports all exported values
        let is_import_all = self.expect_token_type_and_skip(&[TokenType::Star], false);
        if is_import_all {
            self.expect_identifier(true);
            // TODO: enforce identifier be == b"as"
            let ident = self.expect_identifier(true)?;
            // TODO: enforce identifier be == b"from"
            self.expect_identifier(true);
            let specifier = self.expect_string(true)?;
            return Some(ImportKind::AllAs(SpecifierKind::Ident(ident), specifier));
        }

        // `import` followed by an identifier is considered a default import
        if let Some(default_import_ident) = self.expect_identifier(false) {
            self.expect_identifier(true); // TODO: enforce == from
            let specifier = self.expect_string(true)?;
            return Some(ImportKind::DefaultAs(
                SpecifierKind::Ident(default_import_ident),
                specifier,
            ));
        }

        None
    }

    fn parse_throw(&mut self) -> Option<Expr> {
        self.parse_expression()
    }

    fn parse_try(&mut self) -> Option<TryCatch> {
        let try_ = self.parse_statement()?;

        self.expect_token_type_and_skip(&[TokenType::Catch], true);

        let capture_ident = if self.expect_token_type_and_skip(&[TokenType::LeftParen], false) {
            let ident = self.expect_identifier(true)?;
            self.expect_token_type_and_skip(&[TokenType::RightParen], true);
            Some(ident)
        } else {
            None
        };

        let catch = self.parse_statement()?;

        // TODO: finally

        Some(TryCatch::new(try_, Catch::new(catch, capture_ident), None))
    }

    fn parse_return(&mut self) -> Option<ReturnStatement> {
        let return_kw = self.previous()?.span;
        if self.expect_token_type_and_skip(&[TokenType::Semicolon], false) {
            Some(ReturnStatement(Expr {
                span: return_kw, /* `return;` intentionally has an implicit `undefined` with the same span as `return;` */
                kind: ExprKind::undefined_literal(),
            }))
        } else {
            let expr = self.parse_expression()?;
            Some(ReturnStatement(expr))
        }
    }

    fn parse_for_loop(&mut self) -> Option<Loop> {
        self.expect_token_type_and_skip(&[TokenType::LeftParen], true);

        let init = if self.expect_token_type_and_skip(&[TokenType::Semicolon], false) {
            None
        } else {
            let is_binding = self.expect_token_type_and_skip(VARIABLE_TYPES, false);

            if is_binding {
                let binding_span_lo = self.previous()?.span;
                let binding = self.parse_variable_binding()?;
                let is_of_or_in = self.expect_token_type_and_skip(&[TokenType::Of, TokenType::In], false);

                if is_of_or_in {
                    let ty = self.previous()?.ty;
                    let expr = self.parse_expression()?;

                    self.expect_token_type_and_skip(&[TokenType::RightParen], true);

                    let body = Box::new(self.parse_statement()?);

                    return Some(match ty {
                        TokenType::In => Loop::ForIn(ForInLoop { binding, expr, body }),
                        TokenType::Of => Loop::ForOf(ForOfLoop { binding, expr, body }),
                        _ => unreachable!(),
                    });
                } else {
                    let value = self.parse_variable_definition();

                    self.expect_token_type_and_skip(&[TokenType::Semicolon], true);

                    let binding_span_hi = self.previous()?.span;

                    Some(Statement {
                        kind: StatementKind::Variable(VariableDeclarations(vec![VariableDeclaration::new(
                            binding, value,
                        )])),
                        span: binding_span_lo.to(binding_span_hi),
                    })
                }
            } else {
                let stmt = self.parse_statement();
                // The call to statement must have skipped a semicolon
                self.expect_previous(&[TokenType::Semicolon], true);
                stmt
            }
        };

        let cond = if self.expect_token_type_and_skip(&[TokenType::Semicolon], false) {
            None
        } else {
            let expr = self.parse_expression();
            self.expect_token_type_and_skip(&[TokenType::Semicolon], true);
            expr
        };

        let finalizer = if self.expect_token_type_and_skip(&[TokenType::RightParen], false) {
            None
        } else {
            let expr = self.parse_expression();
            self.expect_token_type_and_skip(&[TokenType::RightParen], true);
            expr
        };

        let body = self.parse_statement()?;

        Some(ForLoop::new(init, cond, finalizer, body).into())
    }

    fn parse_while_loop(&mut self) -> Option<Loop> {
        if !self.expect_token_type_and_skip(&[TokenType::LeftParen], true) {
            return None;
        }

        let condition = self.parse_expression()?;

        if !self.expect_token_type_and_skip(&[TokenType::RightParen], true) {
            return None;
        }

        let body = self.parse_statement()?;

        Some(WhileLoop::new(condition, body).into())
    }

    fn parse_do_while_loop(&mut self) -> Option<Loop> {
        let body = self.parse_statement()?;

        if !self.expect_token_type_and_skip(&[TokenType::While], true) {
            return None;
        }

        let condition = self.parse_expression()?;

        Some(DoWhileLoop::new(condition, body).into())
    }

    /// Parses a block. Assumes that the left brace `{` has already been consumed.
    pub fn parse_block(&mut self) -> Option<BlockStatement> {
        let mut stmts = Vec::new();
        while !self.expect_token_type_and_skip(&[TokenType::RightBrace], false) {
            if self.is_eof() {
                return None;
            }

            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            }
        }
        Some(BlockStatement(stmts))
    }

    fn parse_variable(&mut self) -> Option<VariableDeclarations> {
        let mut decls = Vec::new();

        let initial_kind = {
            let binding = self.parse_variable_binding()?;
            let value = self.parse_variable_definition();
            let kind = binding.kind;
            decls.push(VariableDeclaration::new(binding, value));
            kind
        };

        while self.expect_token_type_and_skip(&[TokenType::Comma], false) {
            let binding = self.parse_variable_binding_with_kind(initial_kind)?;
            let value = self.parse_variable_definition();
            decls.push(VariableDeclaration::new(binding, value));
        }

        Some(VariableDeclarations(decls))
    }

    fn parse_if(&mut self, parse_else: bool) -> Option<IfStatement> {
        if !self.expect_token_type_and_skip(&[TokenType::LeftParen], true) {
            return None;
        }

        let condition = self.parse_expression()?;

        if !self.expect_token_type_and_skip(&[TokenType::RightParen], true) {
            return None;
        }

        let then = self.parse_statement()?;

        let mut branches = Vec::new();
        let mut el: Option<Box<Statement>> = None;

        if parse_else {
            while self.expect_token_type_and_skip(&[TokenType::Else], false) {
                let is_if = self.expect_token_type_and_skip(&[TokenType::If], false);

                if is_if {
                    let if_statement = self.parse_if(false)?;
                    branches.push(if_statement);
                } else {
                    el = Some(Box::new(self.parse_statement()?));
                    break;
                }
            }
        }

        Some(IfStatement::new(condition, then, branches, el))
    }

    /// Parses a list of parameters (identifier, followed by optional type segment) delimited by comma,
    /// assuming that the ( has already been consumed
    pub fn parse_parameter_list(&mut self) -> ParameterList {
        let mut parameters = Vec::new();

        while !self.expect_token_type_and_skip(&[TokenType::RightParen], false) {
            let tok = self.next().cloned()?;

            let parameter = match tok.ty {
                TokenType::Dot => {
                    // Begin of spread operator
                    for _ in 0..2 {
                        self.expect_token_type_and_skip(&[TokenType::Dot], true);
                    }

                    let ident = self.expect_identifier(true)?;

                    Parameter::Spread(ident)
                }
                TokenType::Comma => continue,
                // TODO: refactor to if let guards once stable
                other if other.is_identifier() => Parameter::Identifier(other.as_identifier().unwrap()),
                _ => {
                    self.create_error(Error::UnexpectedToken(tok.clone(), TokenType::Comma));
                    return None;
                }
            };

            // Parse type param
            let ty = if self.expect_token_type_and_skip(&[TokenType::Colon], false) {
                Some(self.parse_type_segment()?)
            } else {
                None
            };

            // Parse default value
            let default = if self.expect_token_type_and_skip(&[TokenType::Assignment], false) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            let is_spread = matches!(parameter, Parameter::Spread(..));

            parameters.push((parameter, default, ty));

            if is_spread {
                // Must be followed by )
                if !self.expect_token_type_and_skip(&[TokenType::RightParen], true) {
                    return None;
                }

                break;
            }
        }

        Some(parameters)
    }

    fn parse_variable_binding_with_kind(&mut self, kind: VariableDeclarationKind) -> Option<VariableBinding> {
        let name = if self.expect_token_type_and_skip(&[TokenType::LeftBrace], false) {
            // Object destructuring
            let mut fields = Vec::new();
            let mut rest = None;

            while !self.expect_token_type_and_skip(&[TokenType::RightBrace], false) {
                if !fields.is_empty() {
                    self.expect_token_type_and_skip(&[TokenType::Comma], true);
                }

                let cur = self.current()?.clone();
                match cur.ty {
                    TokenType::RightBrace => {
                        // Trailing comma.
                        self.advance();
                        break;
                    }
                    TokenType::Dot => {
                        // Skip the dot
                        self.advance();
                        // Begin of rest operator, must be followed by two more dots
                        for _ in 0..2 {
                            self.expect_token_type_and_skip(&[TokenType::Dot], true);
                        }

                        let name = self.current()?.clone();
                        if let Some(sym) = name.ty.as_identifier() {
                            if rest.is_some() {
                                // Only allow one rest operator
                                self.create_error(Error::MultipleRestInDestructuring(name));
                                return None;
                            }

                            rest = Some(sym);
                            self.advance();
                        } else {
                            self.create_error(Error::UnexpectedToken(name, TokenType::DUMMY_IDENTIFIER));
                            return None;
                        }
                    }
                    other if other.is_identifier() => {
                        let name = other.as_identifier().unwrap();
                        self.advance();

                        let alias = if self.expect_token_type_and_skip(&[TokenType::Colon], false) {
                            let alias = self.current()?.clone();
                            if let Some(alias) = alias.ty.as_identifier() {
                                self.advance();
                                Some(alias)
                            } else {
                                self.create_error(Error::UnexpectedToken(alias, TokenType::DUMMY_IDENTIFIER));
                                return None;
                            }
                        } else {
                            None
                        };
                        fields.push((name, alias));
                    }
                    _ => {
                        self.create_error(Error::UnexpectedToken(cur, TokenType::DUMMY_IDENTIFIER));
                        return None;
                    }
                }
            }

            VariableDeclarationName::ObjectDestructuring { fields, rest }
        } else if self.expect_token_type_and_skip(&[TokenType::LeftSquareBrace], false) {
            // Array destructuring
            let mut fields = Vec::new();
            let mut rest = None;

            while !self.expect_token_type_and_skip(&[TokenType::RightSquareBrace], false) {
                if !fields.is_empty() {
                    self.expect_token_type_and_skip(&[TokenType::Comma], true);
                }

                let cur = self.current()?.clone();
                match cur.ty {
                    TokenType::RightSquareBrace => {
                        // Trailing comma.
                        self.advance();
                        break;
                    }
                    TokenType::Dot => {
                        // Skip the dot
                        self.advance();
                        // Begin of rest operator, must be followed by two more dots
                        for _ in 0..2 {
                            self.expect_token_type_and_skip(&[TokenType::Dot], true);
                        }

                        let name = self.current()?.clone();
                        if let Some(sym) = name.ty.as_identifier() {
                            if rest.is_some() {
                                // Only allow one rest operator
                                self.create_error(Error::MultipleRestInDestructuring(name));
                                return None;
                            }

                            rest = Some(sym);
                            self.advance();
                        } else {
                            self.create_error(Error::UnexpectedToken(name, TokenType::DUMMY_IDENTIFIER));
                            return None;
                        }
                    }
                    other if other.is_identifier() => {
                        let name = other.as_identifier().unwrap();
                        self.advance();
                        fields.push(name);
                    }
                    _ => {
                        self.create_error(Error::UnexpectedToken(cur, TokenType::DUMMY_IDENTIFIER));
                        return None;
                    }
                }
            }

            VariableDeclarationName::ArrayDestructuring { fields, rest }
        } else {
            // Identifier
            let name = self.expect_identifier(true)?;
            VariableDeclarationName::Identifier(name)
        };

        let ty = if self.expect_token_type_and_skip(&[TokenType::Colon], false) {
            Some(self.parse_type_segment()?)
        } else {
            None
        };

        Some(VariableBinding { kind, name, ty })
    }

    /// Parses a variable binding, i.e. `let x`
    fn parse_variable_binding(&mut self) -> Option<VariableBinding> {
        let kind: VariableDeclarationKind = self.previous()?.ty.into();
        self.parse_variable_binding_with_kind(kind)
    }

    /// Parses the definition segment of a variable declaration statement, i.e. `= 5`
    fn parse_variable_definition(&mut self) -> Option<Expr> {
        // If the next char is `=`, we assume this declaration has a value
        let has_value = self.expect_token_type_and_skip(&[TokenType::Assignment], false);

        if !has_value {
            return None;
        }

        self.parse_expression()
    }

    fn parse_switch(&mut self) -> Option<SwitchStatement> {
        self.expect_token_type_and_skip(&[TokenType::LeftParen], true);
        let value = self.parse_expression()?;
        self.expect_token_type_and_skip(&[TokenType::RightParen], true);

        self.expect_token_type_and_skip(&[TokenType::LeftBrace], true);

        let mut cases = Vec::new();
        let mut default = None;

        // Parse cases
        while !self.expect_token_type_and_skip(&[TokenType::RightBrace], false) {
            let cur = self.current()?.clone();
            self.next()?;

            match cur.ty {
                TokenType::Case => {
                    let value = self.parse_expression()?;
                    self.expect_token_type_and_skip(&[TokenType::Colon], true);

                    let mut body = Vec::new();
                    while !self.expect(&[TokenType::Case, TokenType::Default, TokenType::RightBrace]) {
                        body.push(self.parse_statement()?);
                    }

                    cases.push(SwitchCase { body, value });
                }
                TokenType::Default => {
                    self.expect_token_type_and_skip(&[TokenType::Colon], true);

                    let mut body = Vec::new();
                    while !self.expect(&[TokenType::Case, TokenType::Default, TokenType::RightBrace]) {
                        body.push(self.parse_statement()?);
                    }

                    if default.replace(body).is_some() {
                        self.create_error(Error::MultipleDefaultInSwitch(cur.span));
                        return None;
                    }
                }
                _ => {
                    self.create_error(Error::UnexpectedTokenMultiple(
                        cur,
                        &[TokenType::Case, TokenType::Default],
                    ));
                    return None;
                }
            }
        }

        Some(SwitchStatement {
            cases,
            default,
            expr: value,
        })
    }
}
