use std::fs;

use anyhow::anyhow;
use anyhow::Context;
use clap::ArgMatches;
use dash_lexer::Lexer;
use dash_middle::interner::StringInterner;
use dash_middle::parser::error::IntoFormattableErrors;
use dash_parser::Parser;
use dash_typeck::TypeckCtxt;

pub fn check(arg: &ArgMatches) -> anyhow::Result<()> {
    let path = arg.value_of("file").context("Missing file")?;
    let source = fs::read_to_string(path)?;

    let interner = &mut StringInterner::new();
    let tokens = Lexer::new(interner, &source)
        .scan_all()
        .map_err(|e| anyhow!("{}", e.formattable(interner, &source, true)))?;

    let (ast, _counter) = Parser::new(interner, &source, tokens)
        .parse_all()
        .map_err(|e| anyhow!("{}", e.formattable(interner, &source, true)))?;

    let mut tcx = TypeckCtxt {
        interner,
        source: &source,
        errors: Vec::new(),
    };
    tcx.check_stmts(&ast);
    if !tcx.errors.is_empty() {
        println!("{}", tcx.errors.formattable(interner, &source, true));
    }

    Ok(())
}
