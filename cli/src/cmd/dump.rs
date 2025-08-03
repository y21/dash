use std::io::Write;
use std::{fs, io};

use anyhow::anyhow;
use clap::ArgMatches;
use dash_compiler::transformations;
use dash_middle::interner::StringInterner;
use dash_middle::parser::error::IntoFormattableErrors;
use dash_middle::parser::statement::ScopeId;
use dash_optimizer::OptLevel;
use dash_optimizer::consteval::ConstFunctionEvalCtx;
use dash_optimizer::type_infer::name_res;

pub fn dump(arg: &ArgMatches) -> anyhow::Result<()> {
    let dump_ir = *arg.get_one::<bool>("ir").unwrap();
    let dump_ast = *arg.get_one::<bool>("ast").unwrap();
    let dump_js = *arg.get_one::<bool>("js").unwrap();
    let dump_bytecode = *arg.get_one::<bool>("bytecode").unwrap();
    let dump_tokens = *arg.get_one::<bool>("tokens").unwrap();
    let dump_types = *arg.get_one::<bool>("types").unwrap();

    let opt = *arg.get_one::<OptLevel>("opt").unwrap();
    let path = arg.get_one::<String>("file").unwrap();
    let source = fs::read_to_string(path)?;

    let interner = &mut StringInterner::new();

    let tokens = dash_lexer::Lexer::new(interner, &source)
        .scan_all()
        .map_err(|e| anyhow!("{}", e.formattable(&source, true)))?;

    if dump_tokens {
        println!("{tokens:#?}");
    }

    let (mut ast, scope_counter, local_counter) = dash_parser::Parser::new(interner, &source, tokens)
        .parse_all()
        .map_err(|err| anyhow!("{}", err.formattable(&source, true)))?;

    transformations::ast_patch_implicit_return(&mut ast);

    let nameres = name_res(&ast, scope_counter.len(), local_counter.len());

    if dump_types {
        for local in nameres.scopes[ScopeId::ROOT].expect_function().locals.iter() {
            let ty = local.inferred_type().borrow();
            println!("{}: {ty:?}", interner.resolve(local.name));
        }
    }

    if opt.enabled() {
        let mut cfx = ConstFunctionEvalCtx::new(&nameres.scopes, interner, opt);
        cfx.visit_many_statements(&mut ast);
    }

    if dump_ast {
        println!("{ast:#?}");
    }

    if dump_js {
        for node in &ast {
            println!("{node}");
        }
    }

    let bytecode = dash_compiler::FunctionCompiler::new(&source, opt, nameres, scope_counter, interner)
        .compile_ast(ast, true)
        .map_err(|err| anyhow!("{}", [err].formattable(&source, true)))?;

    if dump_bytecode {
        let buffer = dash_middle::compiler::format::serialize(bytecode)?;
        io::stdout().write_all(&buffer)?;
        return Ok(());
    }

    if dump_ir {
        let out = dash_decompiler::decompile(interner, &bytecode.cp, &bytecode.instructions)?;
        println!("{out}");
    }

    Ok(())
}
