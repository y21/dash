pub use error::Error;
pub use matcher::Matcher;
pub use node::Node;
pub use parser::Parser;

pub mod error;
pub mod matcher;
pub mod node;
pub mod parser;
mod stream;
mod visitor;

pub type Regex = Vec<Node>;

#[cfg(test)]
#[test]
pub fn test() {
    use parser::Parser;

    use crate::matcher::Matcher;

    fn matches(regex: &str, input: &str) -> bool {
        let nodes = Parser::new(regex.as_bytes()).parse_all().unwrap();
        let mut matcher = Matcher::new(dbg!(&nodes), input.as_bytes());
        matcher.matches()
    }

    dbg!(matches("^a[\\db]{2,4}c$", "ab342c"));
}
