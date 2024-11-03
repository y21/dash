use dash_middle::interner::{sym, Symbol};
use dash_middle::lexer::token::{Token, TokenType, VARIABLE_TYPES};
use dash_middle::parser::error::Error;
use dash_middle::parser::expr::{Expr, ExprKind};
use dash_middle::parser::statement::{
    Asyncness, BlockStatement, Catch, Class, ClassMember, ClassMemberKey, ClassMemberValue, DoWhileLoop, ExportKind,
    ForInLoop, ForLoop, ForOfLoop, FunctionDeclaration, FunctionKind, IfStatement, ImportKind, Loop, Parameter,
    ReturnStatement, ScopeId, SpecifierKind, Statement, StatementKind, SwitchCase, SwitchStatement, TryCatch,
    VariableBinding, VariableDeclaration, VariableDeclarationKind, VariableDeclarationName, VariableDeclarations,
    WhileLoop,
};
use dash_middle::parser::types::TypeSegment;
use dash_middle::sourcemap::Span;

use crate::{any, Parser};

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
                self.eat(TokenType::Function, true)?;
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
            TokenType::Continue => Some(StatementKind::Continue(self.parse_break_continue_label())),
            TokenType::Break => Some(StatementKind::Break(self.parse_break_continue_label())),
            TokenType::Debugger => Some(StatementKind::Debugger),
            TokenType::Semicolon => Some(StatementKind::Empty),
            other => 'other: {
                if let TokenType::Identifier(label) = other {
                    if self.eat(TokenType::Colon, false).is_some() {
                        // `foo: <statement that can be broken out of>`
                        let stmt = self.parse_statement()?;
                        break 'other Some(StatementKind::Labelled(label, Box::new(stmt)));
                    }
                }

                // We've skipped the current character because of the statement cases that skip the current token
                // So we go back, as the skipped token belongs to this expression
                self.advance_back();
                Some(StatementKind::Expression(self.parse_expression()?))
            }
        }?;

        let hi_span = self.previous().unwrap().span;

        _ = self.eat(TokenType::Semicolon, false);

        Some(Statement {
            kind,
            span: lo_span.to(hi_span),
        })
    }

    pub fn parse_break_continue_label(&mut self) -> Option<Symbol> {
        if self.eat(TokenType::Semicolon, false).is_some() || self.at_lineterm() {
            None
        } else {
            self.expect_identifier(true)
        }
    }

    pub fn parse_class(&mut self) -> Option<Class> {
        let name = self.expect_identifier(false).map(|ident| self.create_binding(ident));

        let extends = if self.eat(TokenType::Extends, false).is_some() {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.eat(TokenType::LeftBrace, true)?;

        let mut members = Vec::new();

        // Start parsing class members
        while self.eat(TokenType::RightBrace, false).is_none() {
            enum Kind {
                Getter(Span),
                Setter(Span),
                Normal,
            }

            let is_static = self.eat(TokenType::Static, false).is_some();
            let is_private = self.eat(TokenType::Hash, false).is_some();
            let asyncness = match self.eat(TokenType::Async, false).is_some() {
                true => Asyncness::Yes,
                false => Asyncness::No,
            };
            let is_generator = self.eat(TokenType::Star, false).is_some();

            let mut property_kind = if self.eat(TokenType::Get, false).is_some() {
                Kind::Getter(self.previous().unwrap().span)
            } else if self.eat(TokenType::Set, false).is_some() {
                Kind::Setter(self.previous().unwrap().span)
            } else {
                Kind::Normal
            };

            let key = if self.eat(TokenType::LeftSquareBrace, false).is_some() {
                let expr = self.parse_expression()?;
                self.eat(TokenType::RightSquareBrace, true)?;
                ClassMemberKey::Computed(expr)
            } else {
                // HACK: if we have `get` + `(`, it is not a getter
                // change `Kind::Getter` to `Kind::Normal` and treat it as a "get" named method (same for set)
                if self.current().is_some_and(|tok| tok.ty == TokenType::LeftParen)
                    && matches!(property_kind, Kind::Getter(_) | Kind::Setter(_))
                {
                    let key = match property_kind {
                        Kind::Getter(_) => sym::get,
                        Kind::Setter(_) => sym::set,
                        Kind::Normal => unreachable!(),
                    };
                    property_kind = Kind::Normal;
                    ClassMemberKey::Named(key)
                } else {
                    ClassMemberKey::Named(self.expect_identifier_or_reserved_kw(true)?)
                }
            };

            let is_method = self.eat(TokenType::LeftParen, false).is_some();

            if is_method {
                let arguments = self.parse_parameter_list()?;

                // Parse type param
                // TODO: this should probably be part of parse_aprameter_list
                let ty_seg = if self.eat(TokenType::Colon, false).is_some() {
                    Some(self.parse_type_segment()?)
                } else {
                    None
                };

                let body = self.parse_statement()?;

                let func_id = self.scope_count.inc();
                let func = FunctionDeclaration::new(
                    match key {
                        ClassMemberKey::Named(name) => Some(self.create_binding(name)),
                        // TODO: not correct, `class V { ['a']() {} }` should have its name set to 'a'
                        ClassMemberKey::Computed(_) => None,
                    },
                    func_id,
                    arguments,
                    vec![body],
                    match is_generator {
                        true => FunctionKind::Generator,
                        false => FunctionKind::Function(asyncness),
                    },
                    ty_seg,
                    None,
                );

                members.push(ClassMember {
                    private: is_private,
                    static_: is_static,
                    key,
                    value: match property_kind {
                        Kind::Getter(_) => ClassMemberValue::Getter(func),
                        Kind::Setter(_) => ClassMemberValue::Setter(func),
                        Kind::Normal => ClassMemberValue::Method(func),
                    },
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

                _ = self.eat(TokenType::Semicolon, false);
                // Error on `get v = 3`
                if let Kind::Getter(span) | Kind::Setter(span) = property_kind {
                    self.error(Error::Unexpected(span, "getter or setter as field"));
                    return None;
                }

                members.push(ClassMember {
                    private: is_private,
                    static_: is_static,
                    key,
                    value: ClassMemberValue::Field(value),
                });
            };
        }

        Some(Class { name, extends, members })
    }

    fn parse_export(&mut self) -> Option<ExportKind> {
        let is_named = self.eat(TokenType::LeftBrace, false).is_some();

        if is_named {
            let mut names = Vec::new();
            while self.eat(TokenType::RightBrace, false).is_none() {
                let name = self.expect_identifier(true)?;
                names.push(name);
                _ = self.eat(TokenType::Comma, false);
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
        if self.eat(TokenType::Default, true).is_some() {
            let expr = self.parse_expression()?;
            return Some(ExportKind::Default(expr));
        }

        None
    }

    fn parse_import(&mut self) -> Option<ImportKind> {
        // `import` followed by ( is considered a dynamic import
        let is_dynamic = self.eat(TokenType::LeftParen, false).is_some();
        if is_dynamic {
            let specifier = self.parse_expression()?;
            self.eat(TokenType::RightParen, true)?;
            return Some(ImportKind::Dynamic(specifier));
        }

        // `import` followed by a `*` imports all exported values
        let is_import_all = self.eat(TokenType::Star, false).is_some();
        if is_import_all {
            self.expect_identifier(true);
            // TODO: enforce identifier be == b"as"
            let ident = self.expect_identifier(true)?;
            // TODO: enforce identifier be == b"from"
            self.expect_identifier(true);
            let specifier = self.expect_string(true)?;
            return Some(ImportKind::AllAs(
                SpecifierKind::Ident(self.create_binding(ident)),
                specifier,
            ));
        }

        // `import` followed by an identifier is considered a default import
        if let Some(default_import_ident) = self.expect_identifier(false) {
            self.expect_identifier(true); // TODO: enforce == from
            let specifier = self.expect_string(true)?;
            return Some(ImportKind::DefaultAs(
                SpecifierKind::Ident(self.create_binding(default_import_ident)),
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

        let catch = if self.eat(TokenType::Catch, false).is_some() {
            let capture_ident = if self.eat(TokenType::LeftParen, false).is_some() {
                let ident = self.expect_identifier(true)?;
                self.eat(TokenType::RightParen, true)?;
                Some(self.create_binding(ident))
            } else {
                None
            };

            let (span, block) = self.parse_full_block()?;

            Some(Catch::new(span, block, capture_ident))
        } else {
            None
        };

        let finally = if self.eat(TokenType::Finally, false).is_some() {
            Some(self.parse_statement()?)
        } else {
            None
        };

        Some(TryCatch::new(try_, catch, finally))
    }

    fn parse_return(&mut self) -> Option<ReturnStatement> {
        let return_kw = self.previous()?.span;
        if self.eat(TokenType::Semicolon, false).is_some() || self.at_lineterm() {
            Some(ReturnStatement(Expr {
                span: return_kw, /* `return;` intentionally has an implicit `undefined` with the same span as `return;` */
                kind: ExprKind::undefined_literal(),
            }))
        } else {
            let expr = self.parse_expression()?;
            Some(ReturnStatement(expr))
        }
    }

    /// Parses the rest of a for in/for of loop after having confirmed that it is one (after the `in`/`of`).
    fn parse_in_of_loop_after_binding(
        &mut self,
        binding: VariableBinding,
        in_or_of: TokenType,
        scope: ScopeId,
    ) -> Option<Loop> {
        let expr = self.parse_expression()?;

        self.eat(TokenType::RightParen, true)?;

        let body = Box::new(self.parse_statement()?);

        Some(match in_or_of {
            TokenType::In => Loop::ForIn(ForInLoop {
                binding,
                expr,
                body,
                scope,
            }),
            TokenType::Of => Loop::ForOf(ForOfLoop {
                binding,
                expr,
                body,
                scope,
            }),
            _ => unreachable!(),
        })
    }

    fn parse_for_loop(&mut self) -> Option<Loop> {
        self.eat(TokenType::LeftParen, true)?;
        let scope = self.scope_count.inc();

        let init = if self.eat(TokenType::Semicolon, false).is_some() {
            None
        } else {
            if let Some(name) = self.eat(
                (
                    |tok: Token| tok.ty.as_identifier(),
                    [TokenType::DUMMY_IDENTIFIER].as_slice(),
                ),
                false,
            ) {
                if let Some(in_or_of) = self.eat(any(&[TokenType::Of, TokenType::In]), false) {
                    // for (ident in ..)

                    let name = self.create_binding(name);
                    return self.parse_in_of_loop_after_binding(
                        VariableBinding {
                            name: VariableDeclarationName::Identifier(name),
                            kind: VariableDeclarationKind::Var,
                            ty: None,
                        },
                        in_or_of,
                        scope,
                    );
                } else {
                    // Back to the identifier to re-parse it as a regular for statement
                    self.advance_back();
                }
            }

            let is_binding = self.eat(any(VARIABLE_TYPES), false).is_some();

            if is_binding {
                let binding_span_lo = self.previous()?.span;
                let binding = self.parse_variable_binding()?;
                let binding_kind = binding.kind;

                if let Some(in_or_of) = self.eat(any(&[TokenType::Of, TokenType::In]), false) {
                    // for (const binding in ..)

                    return self.parse_in_of_loop_after_binding(binding, in_or_of, scope);
                } else {
                    let value = self.parse_variable_definition();

                    let mut decls = vec![VariableDeclaration::new(binding, value)];
                    while self.eat(TokenType::Comma, false).is_some() {
                        let binding = self.parse_variable_binding_with_kind(binding_kind)?;
                        let def = self.parse_variable_definition();
                        decls.push(VariableDeclaration::new(binding, def));
                    }

                    self.eat(TokenType::Semicolon, true)?;
                    let binding_span_hi = self.previous()?.span;

                    Some(Statement {
                        kind: StatementKind::Variable(VariableDeclarations(decls)),
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

        let cond = if self.eat(TokenType::Semicolon, false).is_some() {
            None
        } else {
            let expr = self.parse_expression();
            self.eat(TokenType::Semicolon, true)?;
            expr
        };

        let finalizer = if self.eat(TokenType::RightParen, false).is_some() {
            None
        } else {
            let expr = self.parse_expression();
            self.eat(TokenType::RightParen, true)?;
            expr
        };

        let body = self.parse_statement()?;

        Some(ForLoop::new(init, cond, finalizer, body, scope).into())
    }

    fn parse_while_loop(&mut self) -> Option<Loop> {
        self.eat(TokenType::LeftParen, true)?;

        let condition = self.parse_expression()?;

        self.eat(TokenType::RightParen, true)?;

        let body = self.parse_statement()?;

        Some(WhileLoop::new(condition, body).into())
    }

    fn parse_do_while_loop(&mut self) -> Option<Loop> {
        let body = self.parse_statement()?;

        self.eat(TokenType::While, true)?;

        let condition = self.parse_expression()?;

        Some(DoWhileLoop::new(condition, body).into())
    }

    pub fn parse_full_block(&mut self) -> Option<(Span, BlockStatement)> {
        self.eat(TokenType::LeftBrace, true)?;
        let lbrace = self.previous().unwrap().span;
        let block = self.parse_block()?;
        let rbrace = self.previous().unwrap().span;

        Some((lbrace.to(rbrace), block))
    }

    /// Parses a block. Assumes that the left brace `{` has already been consumed.
    pub fn parse_block(&mut self) -> Option<BlockStatement> {
        let scope_id = self.scope_count.inc();
        let mut stmts = Vec::new();
        while self.eat(TokenType::RightBrace, false).is_none() {
            if self.is_eof() {
                return None;
            }

            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            }
        }
        Some(BlockStatement(stmts, scope_id))
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

        while self.eat(TokenType::Comma, false).is_some() {
            let binding = self.parse_variable_binding_with_kind(initial_kind)?;
            let value = self.parse_variable_definition();
            decls.push(VariableDeclaration::new(binding, value));
        }

        Some(VariableDeclarations(decls))
    }

    fn parse_if(&mut self, parse_else: bool) -> Option<IfStatement> {
        self.eat(TokenType::LeftParen, true)?;

        let condition = self.parse_expression()?;

        self.eat(TokenType::RightParen, true)?;

        let then = self.parse_statement()?;

        let mut branches = Vec::new();
        let mut el: Option<Box<Statement>> = None;

        if parse_else {
            while self.eat(TokenType::Else, false).is_some() {
                let is_if = self.eat(TokenType::If, false).is_some();

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

        while self.eat(TokenType::RightParen, false).is_none() {
            let tok = self.next().cloned()?;

            let parameter = match tok.ty {
                TokenType::Dot => {
                    // Begin of spread operator
                    for _ in 0..2 {
                        self.eat(TokenType::Dot, true)?;
                    }

                    let ident = self.expect_identifier(true)?;

                    Parameter::Spread(self.create_binding(ident))
                }
                TokenType::Comma => continue,
                // TODO: refactor to if let guards once stable
                other if other.is_identifier() => {
                    Parameter::Identifier(self.create_binding(other.as_identifier().unwrap()))
                }
                _ => {
                    self.error(Error::unexpected_token(tok, TokenType::Comma));
                    return None;
                }
            };

            // Parse type param
            let ty = if self.eat(TokenType::Colon, false).is_some() {
                Some(self.parse_type_segment()?)
            } else {
                None
            };

            // Parse default value
            let default = if self.eat(TokenType::Assignment, false).is_some() {
                Some(self.parse_expression()?)
            } else {
                None
            };

            let is_spread = matches!(parameter, Parameter::Spread(..));

            parameters.push((parameter, default, ty));

            if is_spread {
                // Must be followed by )
                self.eat(TokenType::RightParen, true)?;

                break;
            }
        }

        Some(parameters)
    }

    /// Parses the `x` in `let x = 1`, `[x, y]` in `let [x, y] = [1, 2]`, etc.
    fn parse_variable_binding_with_kind(&mut self, kind: VariableDeclarationKind) -> Option<VariableBinding> {
        let name = if self.eat(TokenType::LeftBrace, false).is_some() {
            // Object destructuring
            let mut fields = Vec::new();
            let mut rest = None;

            while self.eat(TokenType::RightBrace, false).is_none() {
                if !fields.is_empty() {
                    self.eat(TokenType::Comma, true)?;
                }

                let cur = *self.current()?;
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
                            self.eat(TokenType::Dot, true)?;
                        }

                        let name = *self.current()?;
                        if let Some(sym) = name.ty.as_identifier() {
                            if rest.is_some() {
                                // Only allow one rest operator
                                self.error(Error::MultipleRestInDestructuring(name));
                                return None;
                            }

                            rest = Some(self.create_binding(sym));
                            self.advance();
                        } else {
                            self.error(Error::unexpected_token(name, TokenType::DUMMY_IDENTIFIER));
                            return None;
                        }
                    }
                    other if other.is_identifier() => {
                        let name = other.as_identifier().unwrap();
                        self.advance();

                        let alias = if self.eat(TokenType::Colon, false).is_some() {
                            let alias = *self.current()?;
                            if let Some(alias) = alias.ty.as_identifier() {
                                self.advance();
                                Some(alias)
                            } else {
                                self.error(Error::unexpected_token(alias, TokenType::DUMMY_IDENTIFIER));
                                return None;
                            }
                        } else {
                            None
                        };
                        fields.push((self.local_count.inc(), name, alias));
                    }
                    _ => {
                        self.error(Error::unexpected_token(cur, TokenType::DUMMY_IDENTIFIER));
                        return None;
                    }
                }
            }

            VariableDeclarationName::ObjectDestructuring { fields, rest }
        } else if self.eat(TokenType::LeftSquareBrace, false).is_some() {
            // Array destructuring
            let mut fields = Vec::new();
            let mut rest = None;

            while self.eat(TokenType::RightSquareBrace, false).is_none() {
                if !fields.is_empty() {
                    self.eat(TokenType::Comma, true)?;
                }

                let cur = *self.current()?;
                match cur.ty {
                    TokenType::RightSquareBrace => {
                        // Trailing comma.
                        self.advance();
                        break;
                    }
                    TokenType::Comma => {
                        fields.push(None);
                    }
                    TokenType::Dot => {
                        // Skip the dot
                        self.advance();
                        // Begin of rest operator, must be followed by two more dots
                        for _ in 0..2 {
                            self.eat(TokenType::Dot, true)?;
                        }

                        let name = *self.current()?;
                        if let Some(sym) = name.ty.as_identifier() {
                            if rest.is_some() {
                                // Only allow one rest operator
                                self.error(Error::MultipleRestInDestructuring(name));
                                return None;
                            }

                            rest = Some(self.create_binding(sym));
                            self.advance();
                        } else {
                            self.error(Error::unexpected_token(name, TokenType::DUMMY_IDENTIFIER));
                            return None;
                        }
                    }
                    other if other.is_identifier() => {
                        let name = other.as_identifier().unwrap();
                        self.advance();
                        fields.push(Some(self.create_binding(name)));
                    }
                    _ => {
                        self.error(Error::unexpected_token(cur, TokenType::DUMMY_IDENTIFIER));
                        return None;
                    }
                }
            }

            VariableDeclarationName::ArrayDestructuring { fields, rest }
        } else {
            // Identifier
            let name = self.expect_identifier(true)?;
            VariableDeclarationName::Identifier(self.create_binding(name))
        };

        let ty = if self.eat(TokenType::Colon, false).is_some() {
            Some(self.parse_type_segment()?)
        } else {
            None
        };

        Some(VariableBinding { kind, name, ty })
    }

    /// Parses a variable binding, i.e. `x`, assuming that the binding kind has been consumed
    fn parse_variable_binding(&mut self) -> Option<VariableBinding> {
        let kind: VariableDeclarationKind = self.previous()?.ty.into();
        self.parse_variable_binding_with_kind(kind)
    }

    /// Parses the definition segment of a variable declaration statement, i.e. `= 5`
    fn parse_variable_definition(&mut self) -> Option<Expr> {
        // If the next char is `=`, we assume this declaration has a value
        let has_value = self.eat(TokenType::Assignment, false).is_some();

        if !has_value {
            return None;
        }

        self.parse_expression_no_comma()
    }

    fn parse_switch(&mut self) -> Option<SwitchStatement> {
        self.eat(TokenType::LeftParen, true)?;
        let value = self.parse_expression()?;
        self.eat(TokenType::RightParen, true)?;

        self.eat(TokenType::LeftBrace, true)?;

        let mut cases = Vec::new();
        let mut default = None;

        // Parse cases
        while self.eat(TokenType::RightBrace, false).is_none() {
            let cur = *self.current()?;
            self.next()?;

            match cur.ty {
                TokenType::Case => {
                    let value = self.parse_expression()?;
                    self.eat(TokenType::Colon, true)?;

                    let mut body = Vec::new();
                    while !self.matches(any(&[TokenType::Case, TokenType::Default, TokenType::RightBrace])) {
                        body.push(self.parse_statement()?);
                    }

                    cases.push(SwitchCase { body, value });
                }
                TokenType::Default => {
                    self.eat(TokenType::Colon, true)?;

                    let mut body = Vec::new();
                    while !self.matches(any(&[TokenType::Case, TokenType::Default, TokenType::RightBrace])) {
                        body.push(self.parse_statement()?);
                    }

                    if default.replace(body).is_some() {
                        self.error(Error::MultipleDefaultInSwitch(cur.span));
                        return None;
                    }
                }
                _ => {
                    self.error(Error::unexpected_token(
                        cur,
                        &[TokenType::Case, TokenType::Default] as &[_],
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
