use std::mem;

use crate::error::Error;
use crate::node::Anchor;
use crate::node::MetaSequence;
use crate::node::Node;

pub struct Parser<'a> {
    index: usize,
    input: &'a [u8],
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { index: 0, input }
    }

    pub fn next(&mut self) -> Option<u8> {
        let byte = self.input.get(self.index);
        self.index += 1;
        byte.copied()
    }

    pub fn advance(&mut self) {
        self.index += 1;
    }

    pub fn back(&mut self) {
        self.index -= 1;
    }

    pub fn current(&self) -> Option<u8> {
        self.input.get(self.index).copied()
    }

    pub fn is_eof(&self) -> bool {
        self.index >= self.input.len()
    }

    pub fn parse_all(mut self) -> Result<Vec<Node>, Error> {
        let mut nodes = Vec::new();
        while !self.is_eof() {
            if let Some(b'|') = self.current() {
                self.advance();

                let left = mem::take(&mut nodes);
                let mut right = Vec::new();

                while !self.is_eof() {
                    if let Some(b'|') = self.current() {
                        // handle in outer loop
                        break;
                    }
                    right.push(self.parse_primary()?);
                }
                nodes.push(Node::Or(left, right));
            } else {
                nodes.push(self.parse_primary()?);
            }
        }
        Ok(nodes)
    }

    fn parse_primary(&mut self) -> Result<Node, Error> {
        let mut node = match self.next() {
            Some(b'.') => Ok(Node::AnyCharacter),
            Some(b'\\') => self.parse_escape(),
            Some(b'[') => self.parse_character_class(),
            Some(b'(') => self.parse_group(),
            // Anchor = Return early as they cannot have quantifiers
            Some(b'^') => return Ok(Node::Anchor(Anchor::StartOfString)),
            Some(b'$') => return Ok(Node::Anchor(Anchor::EndOfString)),
            Some(other) => Ok(Node::LiteralCharacter(other)),
            None => Err(Error::UnexpectedEof),
        }?;

        match self.next() {
            Some(b'+') => node = Node::unbounded_max_repetition(node, 1),
            Some(b'*') => node = Node::unbounded_max_repetition(node, 0),
            Some(b'?') => node = Node::optional(node),
            Some(b'{') => node = self.parse_bounded_repetition(node)?,
            _ => self.back(), // back undos the advance in next()
        }

        Ok(node)
    }

    fn read_int(&mut self) -> Result<usize, Error> {
        let mut number = 0;
        while let Some(byte) = self.current() {
            match byte {
                b'0'..=b'9' => {
                    number = number * 10 + (byte - b'0') as usize;
                    self.advance();
                }
                _ => return Ok(number),
            }
        }
        Err(Error::UnexpectedEof)
    }

    fn parse_bounded_repetition(&mut self, node: Node) -> Result<Node, Error> {
        let min = self.read_int()?;
        match self.current() {
            Some(b',') => {
                self.advance();
                match self.current() {
                    Some(b'}') => {
                        self.advance();
                        Ok(Node::unbounded_max_repetition(node, min))
                    }
                    _ => {
                        let max = self.read_int()?;
                        self.advance(); // }
                        Ok(Node::repetition(node, min, max))
                    }
                }
            }
            Some(b'}') => {
                self.advance();
                Ok(Node::repetition(node, min, min))
            }
            Some(other) => Err(Error::UnexpectedChar(other)),
            None => Err(Error::UnexpectedEof),
        }
    }

    fn parse_character_class(&mut self) -> Result<Node, Error> {
        let mut nodes = Vec::new();

        while !self.is_eof() {
            if let Some(b']') = self.current() {
                self.advance();
                break;
            }

            nodes.push(self.parse_primary()?);
        }

        Ok(Node::CharacterClass(nodes))
    }

    fn parse_group(&mut self) -> Result<Node, Error> {
        let mut nodes = Vec::new();

        while !self.is_eof() {
            match self.current() {
                Some(b')') => {
                    self.advance();
                    break;
                }
                Some(b'|') => {
                    self.advance();
                    let left = mem::take(&mut nodes);
                    let mut right = Vec::new();

                    while !self.is_eof() {
                        if let Some(b')' | b'|') = self.current() {
                            // handle in outer loop
                            break;
                        }
                        right.push(self.parse_primary()?);
                    }
                    nodes.push(Node::Or(left, right));
                }
                _ => nodes.push(self.parse_primary()?),
            }
        }

        Ok(Node::Group(nodes))
    }

    fn parse_escape(&mut self) -> Result<Node, Error> {
        match self.next() {
            Some(b'd') => Ok(Node::MetaSequence(MetaSequence::Digit)),
            Some(b'w') => Ok(Node::MetaSequence(MetaSequence::Word)),
            Some(b's') => Ok(Node::MetaSequence(MetaSequence::Whitespace)),
            Some(other) => Ok(Node::LiteralCharacter(other)),
            None => Err(Error::UnexpectedEof),
        }
    }
}
