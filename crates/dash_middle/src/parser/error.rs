use std::borrow::Cow;
use std::fmt;
use std::fmt::Write;
use std::num::ParseIntError;

use memchr::memchr;
use memchr::memmem::rfind;
use owo_colors::OwoColorize;

use crate::compiler::constant::LimitExceededError as ConstantLimitExceededError;
use crate::compiler::scope::LimitExceededError as LocalLimitExceededError;
use crate::interner::StringInterner;
use crate::lexer::token::Token;
use crate::lexer::token::TokenType;
use crate::sourcemap::Span;

/// An error that occurred during the "middle" stage of execution,
/// i.e. lexing, parsing or compiling (perhaps counterintuitive with the module this is in...)
#[derive(Debug)]
pub enum Error {
    /// An unknown character/byte
    UnknownCharacter(Span, u8),
    /// An unknown token was found
    UnknownToken(Token),
    /// An token was found that we didn't expect, we expect a certain other token type
    UnexpectedToken(Token, TokenType),
    /// Same as UnexpectedToken, but we expected any of the given token types
    UnexpectedTokenMultiple(Token, &'static [TokenType]),
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
    ConstantPoolLimitExceeded,
    LocalLimitExceeded,
    IfBranchLimitExceeded,
    SwitchCaseLimitExceeded,
    ArrayLitLimitExceeded,
    ObjectLitLimitExceeded,
    ExportNameListLimitExceeded,
    DestructureLimitExceeded,
    ConstAssignment,
    Unimplemented(String),
    ParameterLimitExceeded,
    YieldOutsideGenerator,
    AwaitOutsideAsync,
    UnknownBinding,
    IllegalBreak,
    MissingInitializerInDestructuring,
}

impl From<ConstantLimitExceededError> for Error {
    fn from(_: ConstantLimitExceededError) -> Self {
        Self::ConstantPoolLimitExceeded
    }
}

impl From<LocalLimitExceededError> for Error {
    fn from(_: LocalLimitExceededError) -> Self {
        Self::LocalLimitExceeded
    }
}

struct FormattableError<'a, 'buf> {
    error: &'a Error,
    interner: &'a StringInterner,
    source: &'buf str,
    color: bool,
}

enum DiagnosticKind {
    Error,
    Warning,
}

enum NoteKind {
    Error,
    Warning,
    Help,
}

struct Note {
    kind: NoteKind,
    span: Option<Span>,
    message: Cow<'static, str>,
}

struct DiagnosticBuilder<'f, 'a, 'buf> {
    fcx: &'f FormattableError<'a, 'buf>,
    kind: DiagnosticKind,
    message: Option<Cow<'static, str>>,
    span_notes: Vec<Note>,
}

impl<'f, 'a, 'buf> DiagnosticBuilder<'f, 'a, 'buf> {
    fn error(fcx: &'f FormattableError<'a, 'buf>) -> Self {
        Self {
            fcx,
            message: None,
            span_notes: Vec::new(),
            kind: DiagnosticKind::Error,
        }
    }
    fn message(&mut self, message: impl Into<Cow<'static, str>>) {
        self.message = Some(message.into());
    }
    fn span_error(&mut self, span: Span, message: impl Into<Cow<'static, str>>) {
        self.span_notes.push(Note {
            kind: NoteKind::Error,
            message: message.into(),
            span: Some(span),
        });
    }
    fn help(&mut self, message: impl Into<Cow<'static, str>>) {
        self.span_notes.push(Note {
            kind: NoteKind::Help,
            message: message.into(),
            span: None,
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
                    let LineData {
                        start_index: _,
                        end_index: _,
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
                            write_style!(f, blue bold, "help: ")?;
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
    start_index: usize,
    end_index: usize,
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
        start_index,
        end_index,
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
            Error::UnexpectedToken(Token { span, .. }, ty) => {
                diag.message("unexpected token");
                diag.span_error(span, "");
                diag.help(format!("expected: `{}`", ty.fmt_for_expected_tys()))
            }
            Error::UnexpectedTokenMultiple(Token { span, .. }, tys) => {
                diag.message("unexpected token");
                diag.span_error(span, "");
                diag.help(format!(
                    "expected one of: {}",
                    tys.iter().fold(String::new(), |mut acc, ty| {
                        if !acc.is_empty() {
                            acc.push_str(", ");
                        }
                        let _ = write!(acc, "`{}`", ty.fmt_for_expected_tys());
                        acc
                    })
                ))
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
            Error::ConstantPoolLimitExceeded => todo!(),
            Error::LocalLimitExceeded => todo!(),
            Error::IfBranchLimitExceeded => todo!(),
            Error::SwitchCaseLimitExceeded => todo!(),
            Error::ArrayLitLimitExceeded => todo!(),
            Error::ObjectLitLimitExceeded => todo!(),
            Error::ExportNameListLimitExceeded => todo!(),
            Error::DestructureLimitExceeded => todo!(),
            Error::ConstAssignment => todo!(),
            Error::Unimplemented(_) => todo!(),
            Error::ParameterLimitExceeded => todo!(),
            Error::YieldOutsideGenerator => todo!(),
            Error::AwaitOutsideAsync => todo!(),
            Error::UnknownBinding => todo!(),
            Error::IllegalBreak => todo!(),
            Error::MissingInitializerInDestructuring => todo!(),
        }
        fmt::Display::fmt(&diag, f)
    }
}

pub trait IntoFormattableErrors {
    fn formattable<'a, 'buf>(
        &'a self,
        interner: &'a StringInterner,
        source: &'buf str,
        colors: bool,
    ) -> FormattableErrors<'a, 'buf>;
}

impl IntoFormattableErrors for [Error] {
    fn formattable<'a, 'buf>(
        &'a self,
        interner: &'a StringInterner,
        source: &'buf str,
        colors: bool,
    ) -> FormattableErrors<'a, 'buf> {
        FormattableErrors {
            errors: self,
            interner,
            source,
            colors,
        }
    }
}

pub struct FormattableErrors<'a, 'buf> {
    errors: &'a [Error],
    interner: &'a StringInterner,
    source: &'buf str,
    colors: bool,
}

impl<'a, 'buf> fmt::Display for FormattableErrors<'a, 'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in self.errors {
            FormattableError {
                color: self.colors,
                interner: self.interner,
                source: self.source,
                error,
            }
            .fmt(f)?;
            f.write_str("\n\n")?;
        }
        Ok(())
    }
}
