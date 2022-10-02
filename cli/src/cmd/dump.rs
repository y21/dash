use std::fs;
use std::io;
use std::io::Write;

use anyhow::anyhow;
use anyhow::Context;
use clap::ArgMatches;

use crate::util;

pub fn dump(arg: &ArgMatches) -> anyhow::Result<()> {
    let dump_ir = arg.is_present("ir");
    let dump_ast = arg.is_present("ast");
    let dump_js = arg.is_present("js");
    let dump_bytecode = arg.is_present("bytecode");
    let dump_tokens = arg.is_present("tokens");

    let opt = util::opt_level_from_matches(arg)?;
    let path = arg.value_of("file").context("Missing file")?;
    let source = fs::read_to_string(path)?;

    let tokens = dash_lexer::Lexer::new(&source)
        .scan_all()
        .map_err(|e| anyhow!("Failed to lex source string: {e:?}"))?;

    if dump_tokens {
        println!("{:#?}", tokens);
    }

    let mut ast = dash_parser::Parser::new(&source, tokens)
        .parse_all()
        .map_err(|_| anyhow!("Failed to parse source string"))?;

    dash_optimizer::optimize_ast(&mut ast, opt);

    if dump_ast {
        println!("{:#?}", ast);
    }

    if dump_js {
        for node in &ast {
            println!("{node}");
        }
    }

    let bytecode = dash_compiler::FunctionCompiler::new(opt)
        .compile_ast(ast, true)
        .map_err(|_| anyhow!("Failed to compile source string"))?;

    if dump_bytecode {
        let buffer = dash_middle::compiler::format::serialize(bytecode)?;
        io::stdout().write_all(&buffer)?;
        return Ok(());
    }

    if dump_ir {
        let out = dash_decompiler::decompile(&bytecode.cp, &bytecode.instructions)?;
        println!("{out}");
    }

    Ok(())
}
