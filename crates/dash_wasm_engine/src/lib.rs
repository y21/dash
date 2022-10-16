pub mod parser;
pub mod vm;

#[cfg(test)]
#[test]
fn add() {
    use crate::parser::Parser;

    const WASM: &[u8] = include_bytes!("../add.wasm");
    let program = Parser::new(WASM).parse().unwrap();
    dbg!(program);
}
