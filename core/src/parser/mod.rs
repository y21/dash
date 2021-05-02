pub mod expr;
pub mod lexer;
pub mod parser;
pub mod statement;
pub mod token;
pub mod value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parser() {
        let src = r#"let x = 1+2;"#;

        let tokens = lexer::Lexer::new(src).scan_all();

        let mut parser = parser::Parser::new(tokens);

        dbg!(parser.parse());
    }
}
