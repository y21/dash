use dash_middle::lexer::token::TokenType;
use dash_middle::parser::error::Error;
use dash_middle::parser::types::{LiteralType, TypeSegment};

use crate::Parser;

impl<'a, 'interner> Parser<'a, 'interner> {
    pub fn parse_type_segment(&mut self) -> Option<TypeSegment> {
        self.parse_union_type()
    }

    /// Parses a union type: foo | bar
    fn parse_union_type(&mut self) -> Option<TypeSegment> {
        let mut left = self.parse_intersection_type()?;

        while self.expect_token_type_and_skip(&[TokenType::BitwiseOr], false) {
            let right = self.parse_intersection_type()?;
            left = TypeSegment::Union(Box::new(left), Box::new(right));
        }

        Some(left)
    }

    /// Parses an intersection type: foo & bar
    fn parse_intersection_type(&mut self) -> Option<TypeSegment> {
        let mut left = self.parse_postfix_array()?;

        while self.expect_token_type_and_skip(&[TokenType::BitwiseAnd], false) {
            let right = self.parse_postfix_array()?;
            left = TypeSegment::Union(Box::new(left), Box::new(right));
        }

        Some(left)
    }

    /// Parse postfix array notation: foo[], foo[][], foo[][][], ...
    fn parse_postfix_array(&mut self) -> Option<TypeSegment> {
        let mut target = self.parse_generic_type()?;

        while self.expect_token_type_and_skip(&[TokenType::LeftSquareBrace], false) {
            self.expect_token_type_and_skip(&[TokenType::RightSquareBrace], true);
            target = TypeSegment::Array(Box::new(target));
        }

        Some(target)
    }

    /// Parses a generic type: foo<bar>
    fn parse_generic_type(&mut self) -> Option<TypeSegment> {
        let mut left = self.parse_primary_type()?;

        while self.expect_token_type_and_skip(&[TokenType::Less], false) {
            let mut args = Vec::new();

            while !self.expect_token_type_and_skip(&[TokenType::Greater], false) {
                if !args.is_empty() {
                    // separate types by comma
                    self.expect_token_type_and_skip(&[TokenType::Comma], true);
                }

                args.push(self.parse_type_segment()?);
            }

            left = TypeSegment::Generic(Box::new(left), args);
        }

        Some(left)
    }

    /// Parses a primary type: literals (true, false, Uint8Array)
    fn parse_primary_type(&mut self) -> Option<TypeSegment> {
        let cur = self.next()?;

        let seg = match cur.ty {
            TokenType::Identifier(cur) => TypeSegment::Literal(LiteralType::Identifier(cur)),
            _ => {
                let cur = self.previous().cloned()?;
                self.create_error(Error::UnknownToken(cur));
                return None;
            }
        };

        Some(seg)
    }
}
