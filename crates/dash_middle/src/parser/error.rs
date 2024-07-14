use std::borrow::Cow;
use std::fmt;
use std::fmt::Write;
use std::num::ParseIntError;

use memchr::memchr;
use memchr::memmem::rfind;
use owo_colors::OwoColorize;

use crate::lexer::token::{Token, TokenType};
use crate::sourcemap::Span;

#[derive(Debug, Copy, Clone)]
pub enum TokenTypeSuggestion {
    AnyOf(&'static [TokenType]),
    Exact(TokenType),
    Unknown,
}
impl From<TokenType> for TokenTypeSuggestion {
    fn from(value: TokenType) -> Self {
        Self::Exact(value)
    }
}
impl From<&'static [TokenType]> for TokenTypeSuggestion {
    fn from(value: &'static [TokenType]) -> Self {
        Self::AnyOf(value)
    }
}

/// An error that occurred during the "middle" stage of execution,
/// i.e. lexing, parsing or compiling (perhaps counterintuitive with the module this is in...)
#[derive(Debug)]
pub enum Error {
    /// An unknown character/byte
    UnknownCharacter(Span, u8),
    /// An unknown token was found
    UnknownToken(Token),
    InvalidEscapeSequence(Span),
    UnexpectedToken(Token, TokenTypeSuggestion),
    /// Unexpected end of file
    UnexpectedEof,
    /// Integer parsing failed
    ParseIntError(Token, ParseIntError),
    /// More than one default clause in a switch statement
    MultipleDefaultInSwitch(Span),
    InvalidAccessorParams {
        got: usize,
        expect: usize,
        token: Token,
    },
    MultipleRestInDestructuring(Token),
    RegexSyntaxError(Token, dash_regex::Error),
    IncompleteSpread(Token),
    /* Compiler */
    ConstantPoolLimitExceeded(Span),
    LocalLimitExceeded(Span),
    IfBranchLimitExceeded(Span),
    SwitchCaseLimitExceeded(Span),
    ArrayLitLimitExceeded(Span),
    ObjectLitLimitExceeded(Span),
    ExportNameListLimitExceeded(Span),
    DestructureLimitExceeded(Span),
    ConstAssignment(Span),
    Unimplemented(Span, String),
    ParameterLimitExceeded(Span),
    YieldOutsideGenerator {
        yield_expr: Span,
    },
    AwaitOutsideAsync {
        await_expr: Span,
    },
    IllegalBreak(Span),
    MissingInitializerInDestructuring(Span),
    ArgumentsInRoot(Span),
    Unexpected(Span, &'static str),
}

impl Error {
    pub fn unexpected_token(token: Token, v: impl Into<TokenTypeSuggestion>) -> Self {
        Self::UnexpectedToken(token, v.into())
    }
}

pub struct FormattableError<'a, 'buf> {
    error: &'a Error,
    source: &'buf str,
    color: bool,
}

pub enum DiagnosticKind {
    Error,
    Warning,
}

pub enum NoteKind {
    Error,
    Warning,
    Help,
}

pub struct Note {
    kind: NoteKind,
    span: Option<Span>,
    message: Cow<'static, str>,
}

pub struct DiagnosticBuilder<'f, 'a, 'buf> {
    fcx: &'f FormattableError<'a, 'buf>,
    kind: DiagnosticKind,
    message: Option<Cow<'static, str>>,
    span_notes: Vec<Note>,
}

impl<'f, 'a, 'buf> DiagnosticBuilder<'f, 'a, 'buf> {
    pub fn error(fcx: &'f FormattableError<'a, 'buf>) -> Self {
        Self {
            fcx,
            message: None,
            span_notes: Vec::new(),
            kind: DiagnosticKind::Error,
        }
    }
    pub fn message(&mut self, message: impl Into<Cow<'static, str>>) {
        self.message = Some(message.into());
    }
    pub fn span_error(&mut self, span: Span, message: impl Into<Cow<'static, str>>) {
        self.span_notes.push(Note {
            kind: NoteKind::Error,
            message: message.into(),
            span: Some(span),
        });
    }
    pub fn help(&mut self, message: impl Into<Cow<'static, str>>) {
        self.span_notes.push(Note {
            kind: NoteKind::Help,
            message: message.into(),
            span: None,
        });
    }
    pub fn span_help(&mut self, span: Span, message: impl Into<Cow<'static, str>>) {
        self.span_notes.push(Note {
            kind: NoteKind::Help,
            message: message.into(),
            span: Some(span),
        });
    }
}

impl<'f, 'a, 'buf> fmt::Display for DiagnosticBuilder<'f, 'a, 'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! write_style {
            ($sink:expr, $($style:ident) *, $s:expr) => {
                if self.fcx.color {
                    ::std::write!($sink, "{}", $s.$($style()).*)
                } else {
                    ::std::write!($sink, "{}", $s)
                }
            };
        }

        match self.kind {
            DiagnosticKind::Error => {
                write_style!(f, red bold, "error: ")?;
            }
            DiagnosticKind::Warning => {
                write_style!(f, yellow bold, "warning: ")?;
            }
        }

        f.write_str(self.message.as_ref().expect("no message set for diagnostic"))?;
        f.write_str("\n\n")?;

        for (index, Note { kind, span, message }) in self.span_notes.iter().enumerate() {
            if index > 0 {
                f.write_str("\n\n")?;
            }

            match *span {
                Some(span) => {
                    assert!(span.is_user_span(), "compiler-generated span in diagnostic");
                    let LineData {
                        relative_span_lo,
                        relative_span_hi,
                        line,
                    } = line_data(self.fcx.source, span);

                    write_style!(f, blue bold, " | ")?;
                    f.write_str(line)?;
                    f.write_char('\n')?;

                    f.write_str(&" ".repeat(3 + relative_span_lo))?;

                    let arrows = "^".repeat(relative_span_hi - relative_span_lo);
                    match kind {
                        NoteKind::Error => {
                            write_style!(f, red bold, arrows)?;
                            f.write_char(' ')?;
                            write_style!(f, red bold, message)?;
                        }
                        NoteKind::Warning => {
                            write_style!(f, yellow bold, arrows)?;
                            f.write_char(' ')?;
                            write_style!(f, yellow bold, message)?;
                        }
                        NoteKind::Help => {
                            write_style!(f, cyan bold, arrows)?;
                            f.write_char(' ')?;
                            write_style!(f, cyan bold, message)?;
                        }
                    }
                }
                None => {
                    match kind {
                        NoteKind::Error => {
                            write_style!(f, red bold, "error: ")?;
                        }
                        NoteKind::Warning => {
                            write_style!(f, yellow bold, "warning: ")?;
                        }
                        NoteKind::Help => {
                            write_style!(f, cyan bold, "help: ")?;
                        }
                    }
                    f.write_str(message)?;
                }
            }
        }

        Ok(())
    }
}

struct LineData<'a> {
    relative_span_lo: usize,
    relative_span_hi: usize,
    line: &'a str,
}

fn line_data(source: &str, span: Span) -> LineData<'_> {
    let start_index = rfind(source[..span.lo as usize].as_bytes(), b"\n")
        .map(|x| x + 1)
        .unwrap_or(0);

    let end_index = memchr(b'\n', source[span.hi as usize..].as_bytes())
        .map(|x| x + span.hi as usize)
        .unwrap_or(source.len());

    let relative_span_lo = span.lo as usize - start_index;
    let relative_span_hi = span.hi as usize - start_index;

    let line = &source[start_index..end_index];
    LineData {
        relative_span_lo,
        relative_span_hi,
        line,
    }
}

impl<'a, 'buf> fmt::Display for FormattableError<'a, 'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut diag = DiagnosticBuilder::error(self);
        match *self.error {
            Error::MultipleDefaultInSwitch(span) => {
                diag.message("more than one default in a switch statement");
                diag.span_error(span, "second `default` clause defined here");
                diag.help("consider merging the two `default` clauses");
            }
            Error::UnknownCharacter(span, byte) => {
                diag.message(format!("unknown character: {}", byte as char));
                diag.span_error(span, "");
            }
            Error::UnknownToken(Token { span, .. }) => {
                diag.message("unexpected token");
                diag.span_error(span, "");
            }
            Error::UnexpectedToken(Token { span, .. }, sugg) => {
                diag.message("unexpected token");
                diag.span_error(span, "");
                match sugg {
                    TokenTypeSuggestion::AnyOf(tys) => diag.help(format!(
                        "expected one of: {}",
                        tys.iter().fold(String::new(), |mut acc, ty| {
                            if !acc.is_empty() {
                                acc.push_str(", ");
                            }
                            _ = write!(acc, "`{}`", ty.fmt_for_expected_tys());
                            acc
                        })
                    )),
                    TokenTypeSuggestion::Exact(ty) => diag.help(format!("expected: `{}`", ty.fmt_for_expected_tys())),
                    TokenTypeSuggestion::Unknown => {}
                }
            }
            Error::InvalidEscapeSequence(span) => {
                diag.message("invalid escape sequence");
                diag.span_error(span, "");
            }
            Error::UnexpectedEof => {
                diag.message("unexpected end of file");
                diag.help("more tokens are expected for the last item to parse");
            }
            Error::ParseIntError(Token { span, .. }, ref err) => {
                diag.message("number failed to parse");
                diag.span_error(span, err.to_string());
            }
            Error::InvalidAccessorParams {
                expect,
                got,
                token: Token { span, .. },
            } => {
                diag.message("incorrect number of parameters for accessor");
                diag.span_error(span, format!("expected {expect}, got {got}"));
            }
            Error::MultipleRestInDestructuring(Token { span, .. }) => {
                diag.message("multiple rest elements in destructuring");
                diag.span_error(span, "second rest element defined here");
            }
            Error::RegexSyntaxError(Token { span, .. }, ref err) => {
                diag.message("invalid regular expression");
                diag.span_error(span, err.to_string());
            }
            Error::IncompleteSpread(Token { span, .. }) => {
                diag.message("incomplete spread operator");
                diag.span_error(span, "expected `...`, followed by an expression");
            }
            Error::ConstantPoolLimitExceeded(span) => {
                diag.message("processing this node exceeded the constant pool size limit");
                diag.span_error(span, "");
                diag.help("consider splitting this function into smaller functions as a workaround");
                // TODO: a note that mentions that this is a technical limitation
            }
            Error::LocalLimitExceeded(span) => {
                diag.message("processing this local variable declaration exceeded the variable limit");
                diag.span_error(span, "");
            }
            Error::IfBranchLimitExceeded(span) => {
                diag.message("processing this conditional branch exceeded the branch limit");
                diag.span_error(span, "");
            }
            Error::SwitchCaseLimitExceeded(span) => {
                diag.message("processing this switch statement exceeded the case limit");
                diag.span_error(span, "");
            }
            Error::ArrayLitLimitExceeded(span) => {
                diag.message("processing this array literal exceeded the element count limit");
                diag.span_error(span, "");
            }
            Error::ObjectLitLimitExceeded(span) => {
                diag.message("processing this object literal exceeded the property count limit");
                diag.span_error(span, "");
            }
            Error::ExportNameListLimitExceeded(span) => {
                diag.message("processing this export statement exceeded the binding count limit");
                diag.span_error(span, "");
            }
            Error::DestructureLimitExceeded(span) => {
                diag.message("processing this destructuring pattern exceeded the binding count limit");
                diag.span_error(span, "");
            }
            Error::ConstAssignment(span) => {
                diag.message("attempted to reassign a value to a constant");
                diag.span_error(span, "");
                diag.help("consider changing `const` to `let` to allow reassigning");
            }
            Error::Unimplemented(span, ref msg) => {
                diag.message(format!("unimplemented: {msg}"));
                diag.span_error(span, "error occurred while processing this node");
            }
            Error::ParameterLimitExceeded(span) => {
                diag.message("processing this function exceeded the parameter count limit");
                diag.span_error(span, "");
            }
            Error::YieldOutsideGenerator { yield_expr } => {
                diag.message("`yield` expression outside of a generator function");
                diag.span_error(yield_expr, "expression yielded here");
                diag.help("consider making this a generator function"); // TODO: use span of fn kw?
            }
            Error::AwaitOutsideAsync { await_expr } => {
                diag.message("`await` expression outside of an async function");
                diag.span_error(await_expr, "expression awaited here");
                diag.help("consider marking this function as `async`");
            }
            Error::IllegalBreak(span) => {
                diag.message("`break` or `continue` statement outside of iteration statement encountered");
                diag.span_error(span, "");
            }
            Error::MissingInitializerInDestructuring(span) => {
                diag.message("missing initializer in destructuring pattern");
                diag.span_error(span, "consider adding an initializer to this variable declaration");
            }
            Error::ArgumentsInRoot(span) => {
                diag.message("referencing `arguments` in the root function");
                diag.span_error(span, "");
                diag.help("this function is in the root context and is never called");
            }
            Error::Unexpected(span, descr) => {
                diag.message(format!("unexpected {descr}"));
                diag.span_error(span, "");
            }
        }
        fmt::Display::fmt(&diag, f)
    }
}

pub trait IntoFormattableErrors {
    fn formattable<'a, 'buf>(&'a self, source: &'buf str, colors: bool) -> FormattableErrors<'a, 'buf>;
}

impl IntoFormattableErrors for [Error] {
    fn formattable<'a, 'buf>(&'a self, source: &'buf str, colors: bool) -> FormattableErrors<'a, 'buf> {
        FormattableErrors {
            errors: self,
            source,
            colors,
        }
    }
}

pub struct FormattableErrors<'a, 'buf> {
    errors: &'a [Error],
    source: &'buf str,
    colors: bool,
}

impl<'a, 'buf> fmt::Display for FormattableErrors<'a, 'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in self.errors {
            FormattableError {
                color: self.colors,
                source: self.source,
                error,
            }
            .fmt(f)?;
            f.write_str("\n\n")?;
        }
        Ok(())
    }
}
