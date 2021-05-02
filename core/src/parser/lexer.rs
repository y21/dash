use std::ops::Range;

use super::token::{Location, Token, TokenType};
use crate::util;

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a [u8],
    tokens: Vec<Token<'a>>,
    idx: usize,
    line: usize,
    start: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            input: source.as_bytes(),
            tokens: Vec::new(),
            idx: 0,
            line: 0,
            start: 0,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.idx >= self.input.len()
    }

    pub fn next(&mut self) -> Option<u8> {
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

    pub fn create_contextified_token(&mut self, ty: TokenType) {
        let tok = Token {
            ty,
            loc: Location { line: self.line },
            full: self.get_lexeme(),
        };

        self.tokens.push(tok);
    }

    pub fn create_contextified_conditional_token(
        &mut self,
        default: Option<TokenType>,
        tokens: &[(&[u8], TokenType)],
    ) {
        for (expect, token) in tokens {
            let from = self.idx;
            let slice = self.safe_subslice(from, from + expect.len());

            if slice.eq(*expect) {
                self.create_contextified_token(*token);
                self.idx += expect.len();
                return;
            }
        }

        if let Some(tt) = default {
            self.create_contextified_token(tt);
        }
    }

    pub fn create_contextified_token_with_lexeme(&mut self, ty: TokenType, lexeme: &'a [u8]) {
        let tok = Token {
            ty,
            loc: Location { line: self.line },
            full: lexeme,
        };

        self.tokens.push(tok);
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

    pub fn read_string_literal(&mut self) {
        while !self.is_eof() {
            let cur = self.current_real();
            if cur == b'"' {
                break;
            }

            if cur == b'\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_eof() {
            // TODO: create error token
            unreachable!();
        }

        let lexeme = self.subslice(self.start + 1..self.idx - 1);
        self.create_contextified_token_with_lexeme(TokenType::String, lexeme)
    }

    pub fn read_number_literal(&mut self) {
        while !self.is_eof() {
            let cur = self.current_real();

            if !util::is_digit(cur) {
                break;
            }

            self.advance();
        }

        self.create_contextified_token(TokenType::Number)
    }

    pub fn read_identifier(&mut self) {
        while !self.is_eof() {
            let cur = self.current_real();

            if !util::is_alpha(cur) {
                break;
            }

            self.advance();
        }

        let lexeme = self.get_lexeme();
        self.create_contextified_token(lexeme.into());
    }

    pub fn scan_next(&mut self) {
        let cur = match self.next() {
            Some(c) => c,
            None => return,
        };

        match cur {
            b'(' => self.create_contextified_token(TokenType::LeftParen),
            b')' => self.create_contextified_token(TokenType::RightParen),
            b'{' => self.create_contextified_token(TokenType::LeftBrace),
            b'}' => self.create_contextified_token(TokenType::RightBrace),
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
                    (b"*", TokenType::Exponential),
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
                &[(b".", TokenType::OptionalChaining)],
            ),
            // b'?' => self.create_contextified_token(TokenType::Conditional),
            b';' => self.create_contextified_token(TokenType::Semicolon),
            b'=' => self.create_contextified_token(TokenType::Assignment), // TODO: this is obviously not safe to assume
            b'"' => self.read_string_literal(),
            b'\n' => self.line += 1,
            b' ' => {}
            _ => {
                if util::is_digit(cur) {
                    self.read_number_literal();
                } else if util::is_alpha(cur) {
                    self.read_identifier();
                } else {
                    panic!("Unknown token: {}", cur as char)
                }
            }
        };
    }

    pub fn scan_all(mut self) -> Vec<Token<'a>> {
        while !self.is_eof() {
            self.start = self.idx;
            self.scan_next();
        }

        self.tokens
    }
}
