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

        while self.eat(TokenType::BitwiseOr, false).is_some() {
            let right = self.parse_intersection_type()?;
            left = TypeSegment::Union(Box::new(left), Box::new(right));
        }

        Some(left)
    }

    /// Parses an intersection type: foo & bar
    fn parse_intersection_type(&mut self) -> Option<TypeSegment> {
        let mut left = self.parse_postfix_array()?;

        while self.eat(TokenType::BitwiseAnd, false).is_some() {
            let right = self.parse_postfix_array()?;
            left = TypeSegment::Union(Box::new(left), Box::new(right));
        }

        Some(left)
    }

    /// Parse postfix array notation: foo[], foo[][], foo[][][], ...
    fn parse_postfix_array(&mut self) -> Option<TypeSegment> {
        let mut target = self.parse_generic_type()?;

        while self.eat(TokenType::LeftSquareBrace, false).is_some() {
            self.eat(TokenType::RightSquareBrace, true)?;
            target = TypeSegment::Array(Box::new(target));
        }

        Some(target)
    }

    /// Parses a generic type: foo<bar>
    fn parse_generic_type(&mut self) -> Option<TypeSegment> {
        let mut left = self.parse_primary_type()?;

        while self.eat(TokenType::Less, false).is_some() {
            let mut args = Vec::new();

            while self.eat(TokenType::Greater, false).is_none() {
                if !args.is_empty() {
                    // separate types by comma
                    self.eat(TokenType::Comma, true)?;
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
