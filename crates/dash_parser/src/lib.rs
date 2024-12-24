use dash_log::{Level, debug, span};
use dash_middle::interner::{StringInterner, Symbol};
use dash_middle::lexer::token::{Token, TokenType};
use dash_middle::parser::error::{Error, TokenTypeSuggestion};
use dash_middle::parser::expr::{Expr, ExprKind};
use dash_middle::parser::statement::{Binding, LocalId, ScopeId, Statement};
use dash_middle::sourcemap::{SourceMap, Span};
use dash_middle::util::{Counter, LevelStack};

mod expr;
mod stmt;
mod types;

pub type ParseResult = Result<(Vec<Statement>, Counter<ScopeId>, Counter<LocalId>), Vec<Error>>;

/// A JavaScript source code parser
pub struct Parser<'a, 'interner> {
    tokens: Box<[Token]>,
    errors: Vec<Error>,
    error_sync: bool,
    idx: usize,
    interner: &'interner mut StringInterner,
    source: SourceMap<'a>,
    new_level_stack: LevelStack,
    scope_count: Counter<ScopeId>,
    local_count: Counter<LocalId>,
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

        let mut scope_count = Counter::new();
        scope_count.inc(); // the implicit top level function

        Self {
            tokens: tokens.into_boxed_slice(),
            errors: Vec::new(),
            error_sync: false,
            idx: 0,
            source: SourceMap::new(input),
            new_level_stack: level_stack,
            interner,
            scope_count,
            local_count: Counter::new(),
        }
    }

    fn create_binding(&mut self, ident: Symbol) -> Binding {
        Binding {
            ident,
            id: self.local_count.inc(),
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
    pub fn parse_all(mut self) -> ParseResult {
        let mut stmts = Vec::new();

        while !self.is_eof() {
            if let Some(stmt) = self.parse() {
                stmts.push(stmt);
            }
        }

        if !self.errors.is_empty() {
            Err(self.errors)
        } else {
            Ok((stmts, self.scope_count, self.local_count))
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
                self.error(Error::ParseIntError(self.previous().cloned()?, e));
                None
            }
        }
    }

    fn is_eof(&self) -> bool {
        self.idx >= self.tokens.len()
    }

    fn matches(&self, mut m: impl Matcher) -> bool {
        self.current().is_some_and(|&token| m.matches(token).is_some())
    }

    fn expect_previous(&mut self, ty: &'static [TokenType], emit_error: bool) -> bool {
        let current = match self.previous() {
            Some(k) => k,
            None => {
                if emit_error {
                    self.error(Error::UnexpectedEof);
                }
                return false;
            }
        };

        let ok = ty.iter().any(|ty| ty.eq(&current.ty));

        if !ok && emit_error {
            let current = *current;
            self.error(Error::unexpected_token(current.span, ty));
        }

        ok
    }

    fn error(&mut self, err: Error) {
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
        self.eat(
            (
                |tok: Token| {
                    if let TokenType::TemplateLiteral(literal) = tok.ty {
                        Some(literal)
                    } else {
                        None
                    }
                },
                &[TokenType::DUMMY_TEMPLATE_LITERAL] as &[_],
            ),
            emit_error,
        )
    }

    pub fn expect_identifier(&mut self, emit_error: bool) -> Option<Symbol> {
        self.eat(
            (|tok: Token| tok.ty.as_identifier(), &[TokenType::DUMMY_IDENTIFIER]
                as &[_]),
            emit_error,
        )
    }

    pub fn expect_string(&mut self, emit_error: bool) -> Option<Symbol> {
        self.eat(
            (
                |tok: Token| {
                    if let TokenType::String(sym) = tok.ty {
                        Some(sym)
                    } else {
                        None
                    }
                },
                &[TokenType::DUMMY_STRING] as &[_],
            ),
            emit_error,
        )
    }

    pub fn expect_identifier_or_reserved_kw(&mut self, emit_error: bool) -> Option<Symbol> {
        self.eat(
            (
                |tok: Token| tok.ty.as_identifier_or_reserved_kw(),
                &[TokenType::DUMMY_IDENTIFIER] as &[_],
            ),
            emit_error,
        )
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

    pub fn eat<M: Matcher>(&mut self, mut matcher: M, emit_error: bool) -> Option<M::Output> {
        let current = match self.current() {
            Some(k) => *k,
            None => {
                if emit_error {
                    self.error(Error::UnexpectedEof);
                }
                return None;
            }
        };

        let res = matcher.matches(current);

        if res.is_some() {
            self.advance();
        } else if emit_error {
            self.error(Error::UnexpectedToken(current.span, matcher.suggestion()));
        }

        res
    }
}

pub trait Matcher {
    type Output;

    fn matches(&mut self, t: Token) -> Option<Self::Output>;
    fn suggestion(&self) -> TokenTypeSuggestion;
}

impl Matcher for TokenType {
    type Output = ();

    fn matches(&mut self, t: Token) -> Option<Self::Output> {
        (t.ty == *self).then_some(())
    }
    fn suggestion(&self) -> TokenTypeSuggestion {
        TokenTypeSuggestion::Exact(*self)
    }
}

impl<F, R> Matcher for (F, &'static [TokenType])
where
    F: FnMut(Token) -> Option<R>,
{
    type Output = R;

    fn matches(&mut self, t: Token) -> Option<Self::Output> {
        (self.0)(t)
    }
    fn suggestion(&self) -> TokenTypeSuggestion {
        TokenTypeSuggestion::AnyOf(self.1)
    }
}

pub fn any(s: &'static [TokenType]) -> impl Matcher<Output = TokenType> {
    struct AnyMatcher(&'static [TokenType]);
    impl Matcher for AnyMatcher {
        type Output = TokenType;

        fn matches(&mut self, t: Token) -> Option<Self::Output> {
            self.0.iter().find(|&ty| *ty == t.ty).copied()
        }
        fn suggestion(&self) -> TokenTypeSuggestion {
            TokenTypeSuggestion::AnyOf(self.0)
        }
    }
    AnyMatcher(s)
}
