use std::{borrow::Cow, ops::Range};

use super::token::{Location, Token, TokenType};
use crate::util::{self, Either};

fn force_utf8(s: &[u8]) -> String {
    std::str::from_utf8(s).expect("Invalid UTF8").into()
}

fn force_utf8_borrowed(s: &[u8]) -> &str {
    std::str::from_utf8(s).expect("Invalid UTF8")
}

/// A JavaScript source code lexer
#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a [u8],
    idx: usize,
    line: usize,
    start: usize,
    line_idx: usize,
}

/// An error that may occur during lexing
#[derive(Debug)]
pub struct Error<'a> {
    /// The kind of error
    pub kind: ErrorKind,
    /// Where this error is located in the source string
    pub loc: Location,
    /// The input string
    ///
    /// Errors carry the input string with them because this is necessary
    /// when formatting errors. In the future, we might be able to avoid storing
    /// it here.
    pub source: &'a [u8],
}

impl<'a> Error<'a> {
    /// Formats this error
    pub fn to_string(&self) -> Cow<str> {
        match &self.kind {
            ErrorKind::UnknownCharacter(c) => {
                Cow::Owned(
                    self.loc
                        .to_string(self.source, Either::Right(*c as char), "unknown character", true),
                )
            }
            ErrorKind::UnexpectedEof => Cow::Borrowed("Unexpected end of input"),
        }
    }
}

/// The type of error
#[derive(Debug)]
pub enum ErrorKind {
    /// An unknown character/byte
    UnknownCharacter(u8),
    /// Unexpected end of file
    UnexpectedEof,
}

/// Represents a comment
#[derive(Debug)]
pub enum CommentKind {
    /// A multiline comment: /* */
    Multiline,
    /// A singleline comment: //
    Singleline,
}

/// A lexer node (either a token or an error)
pub enum Node<'a> {
    /// A valid token
    Token(Token<'a>),
    /// An error
    Error(Error<'a>),
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer
    pub fn new(source: &'a str) -> Self {
        Self {
            input: source.as_bytes(),
            idx: 0,
            line: 1,
            start: 0,
            line_idx: 0,
        }
    }

    /// This lexer is exhausted and has reached the end of the string
    fn is_eof(&self) -> bool {
        self.idx >= self.input.len()
    }

    /// Returns the next character
    fn next_char(&mut self) -> Option<u8> {
        let cur = self.current()?;
        self.advance();
        Some(cur)
    }

    /// Returns the current byte
    fn current(&self) -> Option<u8> {
        self.input.get(self.idx).copied()
    }

    /// Looks ahead by one and returns the next byte
    fn peek(&self) -> Option<u8> {
        self.input.get(self.idx + 1).copied()
    }

    /// Returns the current byte, without returning an Option
    fn current_real(&self) -> u8 {
        self.input[self.idx]
    }

    /// Creates a token based on the current location
    fn create_contextified_token(&mut self, ty: TokenType) -> Node<'a> {
        Node::Token(Token {
            ty,
            loc: Location {
                line: self.line,
                offset: self.start,
                line_offset: self.line_idx,
            },
            full: force_utf8_borrowed(self.get_lexeme()),
        })
    }

    /// Creates a token based on the current location and a given predicate
    ///
    /// A token may be multiple bytes wide, in which case this function can be used.
    /// This function can be seen as a helper function to create a token based on the next bytes.
    fn create_contextified_conditional_token(
        &mut self,
        default: Option<TokenType>,
        tokens: &[(&[u8], TokenType)],
    ) -> Node<'a> {
        for (expect, token) in tokens {
            let from = self.idx;
            let slice = self.safe_subslice(from, from + expect.len());

            if slice.eq(*expect) {
                let tok = self.create_contextified_token(*token);
                self.idx += expect.len();
                return tok;
            }
        }

        if let Some(tt) = default {
            return self.create_contextified_token(tt);
        }

        // TODO: can we actually reach this branch?
        unreachable!()
    }

    /// Creates a new error token
    fn create_error(&mut self, kind: ErrorKind) -> Error<'a> {
        Error {
            loc: Location {
                line: self.line,
                offset: self.start,
                line_offset: self.line_idx,
            },
            kind,
            source: self.input,
        }
    }

    /// Creates a token based on the current location and a given lexeme
    fn create_contextified_token_with_lexeme(&mut self, ty: TokenType, lexeme: &'a [u8]) -> Token<'a> {
        Token {
            ty,
            loc: Location {
                line: self.line,
                offset: lexeme.as_ptr() as usize - self.input.as_ptr() as usize,
                line_offset: self.line_idx,
            },
            full: force_utf8_borrowed(lexeme),
        }
    }

    /// Returns the current lexeme
    fn get_lexeme(&self) -> &'a [u8] {
        &self.input[self.start..self.idx]
    }

    /// Slices into the source string
    fn subslice(&self, r: Range<usize>) -> &'a [u8] {
        &self.input[r]
    }

    /// Slices into the source string, but makes sure no panic occurs
    fn safe_subslice(&self, from: usize, to: usize) -> &'a [u8] {
        let from = from.max(0);
        let to = to.min(self.input.len());
        &self.input[from..to]
    }

    /// Advances the cursor
    fn advance(&mut self) {
        self.idx += 1;
    }

    /// Advances the cursor by n
    fn advance_n(&mut self, n: usize) {
        self.idx += n;
    }

    /// Expects the current byte to be `expected` and advances the stream if matched
    fn expect_and_skip(&mut self, expected: u8) -> bool {
        let cur = match self.current() {
            Some(c) => c,
            None => return false,
        };

        if !cur.eq(&expected) {
            return false;
        }

        self.advance();

        true
    }

    /// Reads a string literal
    ///
    /// This function expects to be one byte ahead of a quote
    fn read_string_literal(&mut self) -> Node<'a> {
        let quote = self.input[self.idx - 1];
        let mut found_quote = false;
        while !self.is_eof() {
            let cur = self.current_real();
            if cur == quote {
                self.advance();
                found_quote = true;
                break;
            }

            if cur == b'\n' {
                self.line += 1;
                self.line_idx = self.idx;
            }

            self.advance();
        }

        if !found_quote && self.is_eof() {
            return Node::Error(self.create_error(ErrorKind::UnexpectedEof));
        }

        let lexeme = self.subslice(self.start + 1..self.idx - 1);
        Node::Token(self.create_contextified_token_with_lexeme(TokenType::String, lexeme))
    }

    /// Reads a prefixed number literal (0x, 0b, 0o)
    fn read_prefixed_literal<P>(&mut self, ty: TokenType, predicate: P) -> Node<'a>
    where
        P: Fn(u8) -> bool,
    {
        // Skip prefix (0x)
        self.advance();

        while !self.is_eof() {
            let cur = self.current_real();

            if cur == b'_' || predicate(cur) {
                self.advance();
            } else {
                break;
            }
        }

        self.create_contextified_token(ty)
    }

    /// Reads a number literal
    fn read_number_literal(&mut self) -> Node<'a> {
        let mut is_float = false;
        let mut is_exp = false;

        while !self.is_eof() {
            let cur = self.current_real();

            match cur {
                b'.' => {
                    if is_float {
                        break;
                    }

                    is_float = true;
                }
                b'e' => {
                    if is_exp {
                        break;
                    }

                    is_exp = true;
                }
                _ => {
                    if !util::is_digit(cur) {
                        break;
                    }
                }
            }

            self.advance();
        }

        self.create_contextified_token(TokenType::NumberDec)
    }

    /// Reads an identifier and returns it as a node
    fn read_identifier(&mut self) -> Node<'a> {
        while !self.is_eof() {
            let cur = self.current_real();

            if !util::is_alpha(cur) {
                break;
            }

            self.advance();
        }

        let lexeme = self.get_lexeme();
        self.create_contextified_token(lexeme.into())
    }

    /// Iterates through the input string and yields the next node
    pub fn scan_next(&mut self) -> Option<Node<'a>> {
        self.skip_whitespaces();
        while self.current() == Some(b'/') {
            let index_before_skip = self.idx;
            self.skip_comments();

            // We need to manually break out of the loop if the index didn't change
            // This is the case when visiting a single slash
            if self.idx == index_before_skip {
                break;
            }

            self.skip_whitespaces();
        }
        self.skip_whitespaces();
        self.start = self.idx;

        let cur = match self.next_char() {
            Some(c) => c,
            None => return None,
        };

        Some(match cur {
            b'(' => self.create_contextified_token(TokenType::LeftParen),
            b')' => self.create_contextified_token(TokenType::RightParen),
            b'{' => self.create_contextified_token(TokenType::LeftBrace),
            b'}' => self.create_contextified_token(TokenType::RightBrace),
            b'[' => self.create_contextified_token(TokenType::LeftSquareBrace),
            b']' => self.create_contextified_token(TokenType::RightSquareBrace),
            b',' => self.create_contextified_token(TokenType::Comma),
            b'.' => self.create_contextified_token(TokenType::Dot),
            b'-' => self.create_contextified_conditional_token(
                Some(TokenType::Minus),
                &[(b"-", TokenType::Decrement), (b"=", TokenType::SubtractionAssignment)],
            ),
            b'+' => self.create_contextified_conditional_token(
                Some(TokenType::Plus),
                &[(b"+", TokenType::Increment), (b"=", TokenType::AdditionAssignment)],
            ),
            b'*' => self.create_contextified_conditional_token(
                Some(TokenType::Star),
                &[
                    (b"*=", TokenType::ExponentiationAssignment),
                    (b"*", TokenType::Exponentiation),
                    (b"=", TokenType::MultiplicationAssignment),
                ],
            ),
            b'|' => self.create_contextified_conditional_token(
                Some(TokenType::BitwiseOr),
                &[
                    (b"|=", TokenType::LogicalOrAssignment),
                    (b"=", TokenType::BitwiseOrAssignment),
                    (b"|", TokenType::LogicalOr),
                ],
            ),
            b'^' => self.create_contextified_conditional_token(
                Some(TokenType::BitwiseXor),
                &[(b"=", TokenType::BitwiseXorAssignment)],
            ),
            b'&' => self.create_contextified_conditional_token(
                Some(TokenType::BitwiseAnd),
                &[
                    (b"&=", TokenType::LogicalAndAssignment),
                    (b"=", TokenType::BitwiseAndAssignment),
                    (b"&", TokenType::LogicalAnd),
                ],
            ),
            b'>' => self.create_contextified_conditional_token(
                Some(TokenType::Greater),
                &[
                    (b">>=", TokenType::UnsignedRightShiftAssignment),
                    (b">=", TokenType::RightShiftAssignment),
                    (b">>", TokenType::UnsignedRightShift),
                    (b"=", TokenType::GreaterEqual),
                    (b">", TokenType::RightShift),
                ],
            ),
            b'<' => self.create_contextified_conditional_token(
                Some(TokenType::Less),
                &[
                    (b"<=", TokenType::LeftShiftAssignment),
                    (b"=", TokenType::LessEqual),
                    (b"<", TokenType::LeftShift),
                ],
            ),
            b'%' => self.create_contextified_conditional_token(
                Some(TokenType::Remainder),
                &[(b"=", TokenType::RemainderAssignment)],
            ),
            b'/' => self.create_contextified_conditional_token(
                Some(TokenType::Slash),
                &[(b"=", TokenType::DivisionAssignment)],
            ),
            b'!' => self.create_contextified_conditional_token(
                Some(TokenType::LogicalNot),
                &[(b"==", TokenType::StrictInequality), (b"=", TokenType::Inequality)],
            ),
            b'~' => self.create_contextified_token(TokenType::BitwiseNot),
            b'?' => self.create_contextified_conditional_token(
                Some(TokenType::Conditional),
                &[
                    (b"?=", TokenType::LogicalNullishAssignment),
                    (b"?", TokenType::NullishCoalescing),
                    (b".", TokenType::OptionalChaining),
                ],
            ),
            b'#' => self.create_contextified_token(TokenType::Hash),
            b':' => self.create_contextified_token(TokenType::Colon),
            b';' => self.create_contextified_token(TokenType::Semicolon),
            b'=' => self.create_contextified_conditional_token(
                Some(TokenType::Assignment),
                &[
                    (b"==", TokenType::StrictEquality),
                    (b"=", TokenType::Equality),
                    (b">", TokenType::Arrow),
                ],
            ),
            b'"' | b'\'' => self.read_string_literal(),
            _ => {
                if util::is_digit(cur) {
                    let is_prefixed = cur == b'0';

                    match (is_prefixed, self.current()) {
                        (true, Some(b'x' | b'X')) => {
                            self.read_prefixed_literal(TokenType::NumberHex, util::is_hex_digit)
                        }
                        (true, Some(b'b' | b'B')) => {
                            self.read_prefixed_literal(TokenType::NumberBin, util::is_binary_digit)
                        }
                        (true, Some(b'o' | b'O')) => {
                            self.read_prefixed_literal(TokenType::NumberOct, util::is_octal_digit)
                        }
                        _ => self.read_number_literal(),
                    }
                } else if util::is_identifier_start(cur) {
                    self.read_identifier()
                } else {
                    Node::Error(self.create_error(ErrorKind::UnknownCharacter(cur)))
                }
            }
        })
    }

    /// Skips any meaningless whitespaces
    fn skip_whitespaces(&mut self) {
        while !self.is_eof() {
            let ch = match self.current() {
                Some(c) => c,
                None => return,
            };

            match ch {
                b'\n' => {
                    self.line += 1;
                    self.line_idx = self.idx;
                }
                b'\r' | b'\t' | b' ' => {}
                _ => return,
            };

            self.advance();
        }
    }

    /// Skips any comments
    fn skip_comments(&mut self) {
        let cur = match self.current() {
            Some(c) => c,
            None => return,
        };

        if cur == b'/' {
            match self.peek() {
                Some(b'/') => self.skip_single_line_comment(),
                Some(b'*') => self.skip_multi_line_comment(),
                _ => {}
            };
        }
    }

    /// Skips a single line comment
    fn skip_single_line_comment(&mut self) {
        while !self.is_eof() {
            let ch = match self.current() {
                Some(c) => c,
                None => return,
            };

            if ch == b'\n' {
                self.line += 1;
                self.line_idx = self.idx;
                return;
            }

            self.advance();
        }
    }

    /// Skips a multi line comment
    fn skip_multi_line_comment(&mut self) {
        self.expect_and_skip(b'/');
        self.expect_and_skip(b'*');
        while !self.is_eof() {
            let ch = match self.current() {
                Some(c) => c,
                None => return,
            };

            if ch == b'\n' {
                self.line += 1;
                self.line_idx = self.idx;
            } else if ch == b'*' && self.peek() == Some(b'/') {
                self.advance_n(2);
                return;
            }

            self.advance();
        }
    }

    /// Drives this lexer to completion
    ///
    /// Calling this function will exhaust the lexer and return all nodes
    pub fn scan_all(self) -> Result<Vec<Token<'a>>, Vec<Error<'a>>> {
        let mut errors = Vec::new();
        let mut tokens = Vec::new();
        for node in self {
            match node {
                Node::Token(t) => tokens.push(t),
                Node::Error(e) => errors.push(e),
            }
        }

        // If there are errors, return them
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(tokens)
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.scan_next()
    }
}
