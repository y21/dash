use std::mem;

use crate::error::Error;
use crate::node::{Anchor, CharacterClassItem, GroupCaptureMode, MetaSequence, Node};

pub struct Parser<'a> {
    index: usize,
    input: &'a [u8],
    group_index: usize,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "format", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedRegex {
    pub nodes: Vec<Node>,
    pub group_count: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            index: 0,
            input,
            group_index: 1, // 0 is the entire match
        }
    }

    /// Advances the index and returns the previous byte
    pub fn next_byte(&mut self) -> Option<u8> {
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

    pub fn parse_all(mut self) -> Result<ParsedRegex, Error> {
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
        Ok(ParsedRegex {
            nodes,
            group_count: self.group_index,
        })
    }

    fn parse_primary(&mut self) -> Result<Node, Error> {
        let mut node = match self.next_byte() {
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

        match self.next_byte() {
            Some(b'+') => node = Node::unbounded_max_repetition(node, 1),
            Some(b'*') => node = Node::unbounded_max_repetition(node, 0),
            Some(b'?') => node = Node::optional(node),
            Some(b'{') => node = self.parse_bounded_repetition(node)?,
            _ => self.back(), // back undos the advance in next()
        }

        Ok(node)
    }

    fn read_u32(&mut self) -> Result<u32, Error> {
        let mut number = 0u32;
        while let Some(byte) = self.current() {
            match byte {
                b'0'..=b'9' => {
                    number = number.checked_mul(10).ok_or(Error::Overflow)?;
                    number = number.checked_add((byte - b'0') as u32).ok_or(Error::Overflow)?;

                    self.advance();
                }
                _ => return Ok(number),
            }
        }
        Err(Error::UnexpectedEof)
    }

    fn parse_bounded_repetition(&mut self, node: Node) -> Result<Node, Error> {
        let min = self.read_u32()?;
        match self.current() {
            Some(b',') => {
                self.advance();
                match self.current() {
                    Some(b'}') => {
                        self.advance();
                        Ok(Node::unbounded_max_repetition(node, min))
                    }
                    _ => {
                        let max = self.read_u32()?;
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
            match self.current() {
                Some(b']') => {
                    self.advance();
                    break;
                }
                Some(b'-') => {
                    self.advance();
                    match nodes.last() {
                        Some(&CharacterClassItem::Node(Node::LiteralCharacter(start))) => {
                            let end = self.next_byte().ok_or(Error::UnexpectedEof)?;
                            nodes.pop();
                            nodes.push(CharacterClassItem::Range(start, end));
                        }
                        _ => nodes.push(CharacterClassItem::Node(Node::LiteralCharacter(b'-'))),
                    }
                }
                _ => nodes.push(CharacterClassItem::Node(self.parse_primary()?)),
            }
        }

        Ok(Node::CharacterClass(nodes))
    }

    fn parse_group(&mut self) -> Result<Node, Error> {
        let mut nodes = Vec::new();
        // ?: = non-capturing group
        let capture_mode = if self.current() == Some(b'?') {
            self.advance();
            if self.current() == Some(b':') {
                self.advance();
                GroupCaptureMode::None
            } else {
                self.group_index += 1;
                GroupCaptureMode::Id(self.group_index - 1)
            }
        } else {
            self.group_index += 1;
            GroupCaptureMode::Id(self.group_index - 1)
        };

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

        Ok(Node::Group(capture_mode, nodes))
    }

    fn parse_escape(&mut self) -> Result<Node, Error> {
        match self.next_byte() {
            Some(b'd') => Ok(Node::MetaSequence(MetaSequence::Digit)),
            Some(b'w') => Ok(Node::MetaSequence(MetaSequence::Word)),
            Some(b's') => Ok(Node::MetaSequence(MetaSequence::Whitespace)),
            Some(other) => Ok(Node::LiteralCharacter(other)),
            None => Err(Error::UnexpectedEof),
        }
    }
}
