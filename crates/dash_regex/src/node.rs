#[derive(Debug)]
pub enum Node {
    AnyCharacter,
    MetaSequence(MetaSequence),
    Repetition {
        node: Box<Node>,
        min: usize,
        max: Option<usize>,
    },
    LiteralCharacter(u8),
    CharacterClass(Vec<Node>),
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
}

#[derive(Debug)]
pub enum MetaSequence {
    Digit,
    Word,
    Whitespace,
}
