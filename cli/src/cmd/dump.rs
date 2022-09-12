use std::fs;

use anyhow::anyhow;
use anyhow::Context;
use clap::ArgMatches;

use crate::util;

pub fn dump(arg: &ArgMatches) -> anyhow::Result<()> {
    let dump_bytecode = arg.is_present("bytecode");

    let opt = util::opt_level_from_matches(arg)?;
    let path = arg.value_of("file").context("Missing file")?;
    let source = fs::read_to_string(path)?;

    let tokens = dash_lexer::Lexer::new(&source)
        .scan_all()
        .map_err(|_| anyhow!("Failed to lex source string"))?;

    let ast = dash_parser::Parser::new(&source, tokens)
        .parse_all()
        .map_err(|_| anyhow!("Failed to parse source string"))?;

    let bytecode = dash_compiler::FunctionCompiler::new(opt)
        .compile_ast(ast, true)
        .map_err(|_| anyhow!("Failed to compile source string"))?;

    if dump_bytecode {
        let out = dash_decompiler::decompile(&bytecode.cp, &bytecode.instructions)?;
        println!("{out}");
    }

    Ok(())
}
