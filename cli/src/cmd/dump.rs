use std::fs;
use std::io;
use std::io::Write;

use anyhow::anyhow;
use anyhow::Context;
use clap::ArgMatches;
use dash_middle::parser::statement::VariableDeclarationName;
use dash_optimizer::consteval::Eval;
use dash_optimizer::context::OptimizerContext;
use dash_optimizer::type_infer::TypeInferCtx;

use crate::util;

pub fn dump(arg: &ArgMatches) -> anyhow::Result<()> {
    let dump_ir = arg.is_present("ir");
    let dump_ast = arg.is_present("ast");
    let dump_js = arg.is_present("js");
    let dump_bytecode = arg.is_present("bytecode");
    let dump_tokens = arg.is_present("tokens");
    let dump_types = arg.is_present("types");

    let opt = util::opt_level_from_matches(arg)?;
    let path = arg.value_of("file").context("Missing file")?;
    let source = fs::read_to_string(path)?;

    let tokens = dash_lexer::Lexer::new(&source)
        .scan_all()
        .map_err(|e| anyhow!("Failed to lex source string: {e:?}"))?;

    if dump_tokens {
        println!("{:#?}", tokens);
    }

    let (mut ast, counter) = dash_parser::Parser::new(&source, tokens)
        .parse_all()
        .map_err(|_| anyhow!("Failed to parse source string"))?;

    let tcx = TypeInferCtx::new(counter);

    if dump_types {
        let mut cx = OptimizerContext::new();
        ast.fold(&mut cx, true);

        for local in cx.scope_mut().locals() {
            if let VariableDeclarationName::Identifier(ident) = local.binding().name {
                let ty = local.inferred_type().borrow();
                println!("{ident}: {ty:?}");
            }
        }
    } else {
        dash_optimizer::optimize_ast(&mut ast, opt);
    }

    if dump_ast {
        println!("{:#?}", ast);
    }

    if dump_js {
        for node in &ast {
            println!("{node}");
        }
    }

    let bytecode = dash_compiler::FunctionCompiler::new(opt, tcx)
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
