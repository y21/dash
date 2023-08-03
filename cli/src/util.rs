use std::cell::OnceCell;

use anyhow::Context;
use clap::ArgMatches;
use dash_compiler::FunctionCompiler;
use dash_middle::compiler::CompileResult;
use dash_optimizer::OptLevel;
use dash_vm::frame::Exports;
use dash_vm::frame::Frame;
use dash_vm::localscope::LocalScope;
use dash_vm::value::ops::abstractions::conversions::ValueConversion;
use dash_vm::value::Value;

pub fn opt_level_from_matches(args: &ArgMatches) -> anyhow::Result<OptLevel> {
    args.value_of("opt")
        .and_then(OptLevel::from_level)
        .context("Invalid opt level")
}

pub fn print_value(value: Value, scope: &mut LocalScope) -> Result<(), Value> {
    thread_local! {
        // Cache bytecode so we can avoid recompiling it every time
        // We can be even smarter if we need to -- cache the whole value at callsite
        static INSPECT_BC: OnceCell<CompileResult> = OnceCell::new();
    }

    let inspect_bc = INSPECT_BC.with(|tls| {
        let inspect = tls.get_or_init(|| {
            FunctionCompiler::compile_str(include_str!("../../crates/dash_rt/js/inspect.js"), Default::default())
                .unwrap()
        });
        inspect.clone()
    });

    let Exports {
        default: Some(inspect_fn),
        ..
    } = scope.execute_module(Frame::from_compile_result(inspect_bc)).unwrap()
    else {
        panic!("inspect module did not have a default export");
    };

    let result = inspect_fn
        .root(scope)
        .apply(scope, Value::undefined(), vec![value])
        .unwrap()
        .to_string(scope)
        .unwrap();

    println!("{result}");

    Ok(())
}
