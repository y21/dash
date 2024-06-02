use dash_log::{debug, span, Level};
use dash_middle::interner::{StringInterner, Symbol};
use dash_middle::lexer::token::{Token, TokenType};
use dash_middle::parser::error::Error;
use dash_middle::parser::expr::{Expr, ExprKind};
use dash_middle::parser::statement::{FuncId, Statement};
use dash_middle::sourcemap::{SourceMap, Span};
use dash_middle::util::{Counter, LevelStack};

mod expr;
mod stmt;
mod types;

/// A JavaScript source code parser
pub struct Parser<'a, 'interner> {
    tokens: Box<[Token]>,
    errors: Vec<Error>,
    error_sync: bool,
    idx: usize,
    interner: &'interner mut StringInterner,
    source: SourceMap<'a>,
    new_level_stack: LevelStack,
    function_counter: Counter<FuncId>,
}

impl<'a, 'interner> Parser<'a, 'interner> {
    /// Creates a [`Parser`] from a string.
    #[cfg(feature = "from_string")]
    pub fn new_from_str(interner: &'interner mut StringInterner, input: &'a str) -> Result<Self, Vec<Error>> {
        dash_lexer::Lexer::new(interner, input)
            .scan_all()
            .map(|tok| Self::new(interner, input, tok))
    }

    /// Creates a new parser from tokens generated by a [Lexer]
    pub fn new(interner: &'interner mut StringInterner, input: &'a str, tokens: Vec<Token>) -> Self {
        let mut level_stack = LevelStack::new();
        level_stack.add_level();

        Self {
            tokens: tokens.into_boxed_slice(),
            errors: Vec::new(),
            error_sync: false,
            idx: 0,
            source: SourceMap::new(input),
            new_level_stack: level_stack,
            interner,
            // FuncId::ROOT (0) is reserved for the root function, so the counter for new functions has to start at 1
            function_counter: Counter::with(FuncId::FIRST_NON_ROOT),
        }
    }

    /// Attempts to parse a single statement
    /// If an error occurs, `None` is returned and an error is added to
    /// an internal errors vec
    /// Usually `parse_all` is used to attempt to parse the entire program
    /// and get any existing errors
    pub fn parse(&mut self) -> Option<Statement> {
        let parse = span!(Level::TRACE, "parser");
        parse.in_scope(|| self.parse_statement())
    }

    /// Iteratively parses every token and returns an AST, or a vector of errors
    ///
    /// The AST will be folded by passing true as the `fold` parameter.
    pub fn parse_all(mut self) -> Result<(Vec<Statement>, Counter<FuncId>), Vec<Error>> {
        let mut stmts = Vec::new();

        while !self.is_eof() {
            if let Some(stmt) = self.parse() {
                stmts.push(stmt);
            }
        }

        if !self.errors.is_empty() {
            Err(self.errors)
        } else {
            Ok((stmts, self.function_counter))
        }
    }

    /// Parses a prefixed number literal (0x, 0o, 0b) and returns the number
    pub fn parse_prefixed_number_literal(&mut self, span: Span, full: Symbol, radix: u32) -> Option<Expr> {
        let src = &self.interner.resolve(full)[2..];
        match u64::from_str_radix(src, radix).map(|x| x as f64) {
            Ok(f) => Some(Expr {
                span,
                kind: ExprKind::number_literal(f),
            }),
            Err(e) => {
                self.create_error(Error::ParseIntError(self.previous().cloned()?, e));
                None
            }
        }
    }

    fn is_eof(&self) -> bool {
        self.idx >= self.tokens.len()
    }

    fn expect(&self, expected_ty: &'static [TokenType]) -> bool {
        match self.current() {
            Some(Token { ty, .. }) => expected_ty.contains(ty),
            _ => false,
        }
    }

    fn expect_previous(&mut self, ty: &'static [TokenType], emit_error: bool) -> bool {
        let current = match self.previous() {
            Some(k) => k,
            None => {
                if emit_error {
                    self.create_error(Error::UnexpectedEof);
                }
                return false;
            }
        };

        let ok = ty.iter().any(|ty| ty.eq(&current.ty));

        if !ok && emit_error {
            let current = current.clone();
            self.create_error(Error::UnexpectedTokenMultiple(current, ty));
        }

        ok
    }

    fn expect_token_type_and_skip(&mut self, ty: &'static [TokenType], emit_error: bool) -> bool {
        self.expect_token_and_skip(|to_check| ty.contains(to_check), ty, emit_error)
    }

    fn expect_token_and_skip(
        &mut self,
        check: impl FnOnce(&TokenType) -> bool,
        expected_ty: &'static [TokenType],
        emit_error: bool,
    ) -> bool {
        let current = match self.current() {
            Some(k) => k,
            None => {
                if emit_error {
                    self.create_error(Error::UnexpectedEof);
                }
                return false;
            }
        };

        let ok = check(&current.ty);

        if ok {
            self.advance();
        } else if emit_error {
            let current = current.clone();
            self.create_error(Error::UnexpectedTokenMultiple(current, expected_ty));
        }

        ok
    }

    fn create_error(&mut self, err: Error) {
        debug!("got error {:?}, recovering", err);
        if !self.error_sync {
            self.errors.push(err);
            self.error_sync = true;
        }
    }

    fn advance(&mut self) {
        self.idx += 1;
    }

    fn advance_back(&mut self) {
        self.idx -= 1;
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.idx)
    }

    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.idx - 1)
    }

    fn next(&mut self) -> Option<&Token> {
        self.advance();
        self.previous()
    }

    pub fn expect_template_literal(&mut self, emit_error: bool) -> Option<Symbol> {
        if self.expect_token_and_skip(
            |ty| matches!(ty, TokenType::TemplateLiteral(_)),
            &[TokenType::DUMMY_TEMPLATE_LITERAL],
            emit_error,
        ) {
            match self.previous().unwrap().ty {
                TokenType::TemplateLiteral(sym) => Some(sym),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    pub fn expect_identifier(&mut self, emit_error: bool) -> Option<Symbol> {
        if self.expect_token_and_skip(|ty| ty.is_identifier(), &[TokenType::DUMMY_IDENTIFIER], emit_error) {
            self.previous().unwrap().ty.as_identifier()
        } else {
            None
        }
    }

    pub fn expect_string(&mut self, emit_error: bool) -> Option<Symbol> {
        if self.expect_token_and_skip(
            |ty| matches!(ty, TokenType::String(_)),
            &[TokenType::DUMMY_STRING],
            emit_error,
        ) {
            match self.previous().unwrap().ty {
                TokenType::String(sym) => Some(sym),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    pub fn expect_identifier_or_reserved_kw(&mut self, emit_error: bool) -> Option<Symbol> {
        // TODO: this isn't quite right, it should always skip, even if it didn't match. also the argument is useless, we always call it with false
        if self.expect_token_and_skip(
            |ty| ty.is_identifier_or_reserved_kw(),
            &[TokenType::DUMMY_IDENTIFIER],
            emit_error,
        ) {
            self.previous().unwrap().ty.as_identifier_or_reserved_kw()
        } else {
            None
        }
    }

    /// Checks if between the previous token and the current token there is a LineTerminator or EOF. Necessary for automatic semicolon insertion.
    ///
    /// ```ignore
    /// (function() {
    ///     return
    ///     var x = 1
    ///     ^~~ true for this token
    /// })
    /// ```
    pub fn at_lineterm(&self) -> bool {
        let prev_span = self.previous().map(|t| t.span);
        let curr_span = self.current().map(|c| c.span);

        let (lo, hi) = match (prev_span, curr_span) {
            (_, None) => return true,
            (None, Some(curr)) => (0, curr.lo),
            (Some(prev), Some(curr)) => (prev.hi, curr.lo),
        };
        let src = self.source.resolve(Span { lo, hi });
        src.contains('\n')
    }
}
