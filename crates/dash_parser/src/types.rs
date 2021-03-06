use dash_middle::lexer::token::TokenType;
use dash_middle::parser::error::ErrorKind;
use dash_middle::parser::types::LiteralType;
use dash_middle::parser::types::TypeSegment;

use crate::Parser;

pub trait TypeParser<'a> {
    fn parse_type_segment(&mut self) -> Option<TypeSegment<'a>>;

    /// Parses a union type: foo | bar
    fn parse_union_type(&mut self) -> Option<TypeSegment<'a>>;

    /// Parses an intersection type: foo & bar
    fn parse_intersection_type(&mut self) -> Option<TypeSegment<'a>>;

    /// Parse postfix array notation: foo[], foo[][], foo[][][], ...
    fn parse_postfix_array(&mut self) -> Option<TypeSegment<'a>>;

    /// Parses a primary type: literals (true, false, Uint8Array)
    fn parse_primary_type(&mut self) -> Option<TypeSegment<'a>>;
}

impl<'a> TypeParser<'a> for Parser<'a> {
    fn parse_type_segment(&mut self) -> Option<TypeSegment<'a>> {
        self.parse_union_type()
    }

    fn parse_union_type(&mut self) -> Option<TypeSegment<'a>> {
        let mut left = self.parse_intersection_type()?;

        while self.expect_and_skip(&[TokenType::BitwiseOr], false) {
            let right = self.parse_intersection_type()?;
            left = TypeSegment::Union(Box::new(left), Box::new(right));
        }

        Some(left)
    }

    fn parse_intersection_type(&mut self) -> Option<TypeSegment<'a>> {
        let mut left = self.parse_postfix_array()?;

        while self.expect_and_skip(&[TokenType::BitwiseOr], false) {
            let right = self.parse_postfix_array()?;
            left = TypeSegment::Union(Box::new(left), Box::new(right));
        }

        Some(left)
    }

    fn parse_postfix_array(&mut self) -> Option<TypeSegment<'a>> {
        let mut target = self.parse_primary_type()?;

        while self.expect_and_skip(&[TokenType::EmptySquareBrace], false) {
            target = TypeSegment::Array(Box::new(target));
        }

        Some(target)
    }

    fn parse_primary_type(&mut self) -> Option<TypeSegment<'a>> {
        let (full, ty) = {
            let cur = self.next()?;
            (cur.full, cur.ty)
        };

        let seg = match ty {
            TokenType::Identifier => TypeSegment::Literal(LiteralType::Identifier(full)),
            _ => {
                let cur = self.previous().cloned()?;
                self.create_error(ErrorKind::UnknownToken(cur));
                return None;
            }
        };

        Some(seg)
    }
}
