#[cfg(feature = "format")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum CharacterClassItem {
    Node(Node),
    Range(u8, u8),
}

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    AnyCharacter,
    MetaSequence(MetaSequence),
    Repetition {
        node: Box<Node>,
        min: usize,
        max: Option<usize>,
    },
    LiteralCharacter(u8),
    CharacterClass(Vec<CharacterClassItem>),
    Anchor(Anchor),
    Or(Vec<Node>, Vec<Node>),
    Optional(Box<Node>),
    Group(Vec<Node>),
}

impl Node {
    pub fn unbounded_max_repetition(node: Node, min: usize) -> Self {
        Self::Repetition {
            node: Box::new(node),
            min,
            max: None,
        }
    }
    pub fn repetition(node: Node, min: usize, max: usize) -> Self {
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
#[derive(Debug, Clone, PartialEq)]
pub enum MetaSequence {
    Digit,
    Word,
    Whitespace,
}

#[cfg_attr(feature = "format", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Anchor {
    StartOfString,
    EndOfString,
}
