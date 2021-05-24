use std::ops::Range;

use super::token::{Location, Token, TokenType};
use crate::util;

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a [u8],
    idx: usize,
    line: usize,
    start: usize,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub loc: Location,
}

#[derive(Debug)]
pub enum ErrorKind {
    UnknownCharacter(u8),
    UnexpectedEof,
}

pub enum Node<'a> {
    Token(Token<'a>),
    Error(Error),
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            input: source.as_bytes(),
            idx: 0,
            line: 1,
            start: 0,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.idx >= self.input.len()
    }

    pub fn next_char(&mut self) -> Option<u8> {
        let cur = self.current()?;
        self.advance();
        Some(cur)
    }

    pub fn current(&self) -> Option<u8> {
        self.input.get(self.idx).copied()
    }

    pub fn current_real(&self) -> u8 {
        self.input[self.idx]
    }

    pub fn create_contextified_token(&mut self, ty: TokenType) -> Node<'a> {
        Node::Token(Token {
            ty,
            loc: Location { line: self.line },
            full: self.get_lexeme(),
        })
    }

    pub fn create_contextified_conditional_token(
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

    pub fn create_error(&mut self, kind: ErrorKind) -> Error {
        Error {
            loc: Location { line: self.line },
            kind,
        }
    }

    pub fn create_contextified_token_with_lexeme(
        &mut self,
        ty: TokenType,
        lexeme: &'a [u8],
    ) -> Token<'a> {
        Token {
            ty,
            loc: Location { line: self.line },
            full: lexeme,
        }
    }

    pub fn get_lexeme(&self) -> &'a [u8] {
        &self.input[self.start..self.idx]
    }

    pub fn subslice(&self, r: Range<usize>) -> &'a [u8] {
        &self.input[r]
    }

    pub fn safe_subslice(&self, from: usize, to: usize) -> &'a [u8] {
        let from = from.max(0);
        let to = to.min(self.input.len());
        &self.input[from..to]
    }

    pub fn advance(&mut self) {
        self.idx += 1;
    }

    pub fn expect_and_skip(&mut self, expected: u8) -> bool {
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

    pub fn read_string_literal(&mut self) -> Node<'a> {
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
            }

            self.advance();
        }

        if !found_quote && self.is_eof() {
            return Node::Error(self.create_error(ErrorKind::UnexpectedEof));
        }

        let lexeme = self.subslice(self.start + 1..self.idx - 1);
        Node::Token(self.create_contextified_token_with_lexeme(TokenType::String, lexeme))
    }

    pub fn read_number_literal(&mut self) -> Node<'a> {
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

        self.create_contextified_token(TokenType::Number)
    }

    pub fn read_identifier(&mut self) -> Node<'a> {
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

    pub fn scan_next(&mut self) -> Option<Node<'a>> {
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
                &[
                    (b"-", TokenType::Decrement),
                    (b"=", TokenType::SubtractionAssignment),
                ],
            ),
            b'+' => self.create_contextified_conditional_token(
                Some(TokenType::Plus),
                &[
                    (b"+", TokenType::Increment),
                    (b"=", TokenType::AdditionAssignment),
                ],
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
            // TODO: comments...
            b'/' => self.create_contextified_conditional_token(
                Some(TokenType::Slash),
                &[(b"=", TokenType::DivisionAssignment)],
            ),
            b'!' => self.create_contextified_conditional_token(
                Some(TokenType::LogicalNot),
                &[
                    (b"==", TokenType::StrictInequality),
                    (b"=", TokenType::Inequality),
                ],
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
            b':' => self.create_contextified_token(TokenType::Colon),
            b';' => self.create_contextified_token(TokenType::Semicolon),
            b'=' => self.create_contextified_conditional_token(
                Some(TokenType::Assignment),
                &[
                    (b"==", TokenType::StrictEquality),
                    (b"=", TokenType::Equality),
                ],
            ),
            b'"' | b'\'' => self.read_string_literal(),
            _ => {
                if util::is_digit(cur) {
                    self.read_number_literal()
                } else if util::is_alpha(cur) {
                    self.read_identifier()
                } else {
                    Node::Error(self.create_error(ErrorKind::UnknownCharacter(cur)))
                }
            }
        })
    }

    pub fn skip_whitespaces(&mut self) {
        while !self.is_eof() {
            let ch = if let Some(c) = self.current() {
                c
            } else {
                return;
            };

            if ch == b'\n' {
                self.line += 1;
            } else if ch != b' ' {
                return;
            }

            self.advance();
        }
    }

    pub fn scan_all(self) -> Result<Vec<Token<'a>>, Vec<Error>> {
        let mut errors = Vec::new();
        let mut tokens = Vec::new();
        for node in self {
            match node {
                Node::Token(t) => tokens.push(t),
                Node::Error(e) => errors.push(e),
            }
        }

        // If there are errors, return them
        if errors.len() > 0 {
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
