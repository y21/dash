#[cfg(feature = "format")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum CharacterClassItem {
    Node(Node),
    Range(u8, u8),
}

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroupCaptureMode {
    /// `(?:...)`
    None,
    /// `(...)`
    Id(usize),
}

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    AnyCharacter,
    MetaSequence(MetaSequence),
    Repetition {
        node: Box<Node>,
        min: u32,
        max: Option<u32>,
    },
    LiteralCharacter(u8),
    CharacterClass(Vec<CharacterClassItem>),
    Anchor(Anchor),
    Or(Vec<Node>, Vec<Node>),
    Optional(Box<Node>),
    Group(GroupCaptureMode, Vec<Node>),
}

impl Node {
    pub fn unbounded_max_repetition(node: Node, min: u32) -> Self {
        Self::Repetition {
            node: Box::new(node),
            min,
            max: None,
        }
    }
    pub fn repetition(node: Node, min: u32, max: u32) -> Self {
        Self::Repetition {
            node: Box::new(node),
            min,
            max: Some(max),
        }
    }
    pub fn optional(node: Node) -> Self {
        Self::Optional(Box::new(node))
    }
}

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetaSequence {
    Digit,
    Word,
    Whitespace,
}

impl MetaSequence {
    pub fn matches(self, c: u8) -> bool {
        match self {
            MetaSequence::Digit => c.is_ascii_digit(),
            MetaSequence::Word => c.is_ascii_alphanumeric() || c == b'_',
            MetaSequence::Whitespace => c.is_ascii_whitespace(),
        }
    }
}

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    StartOfString,
    EndOfString,
}
