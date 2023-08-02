use dash_optimizer::OptLevel;
use dash_rt::runtime::Runtime;
use dash_rt::state::State;
use dash_vm::eval::EvalError;
use std::fs;
use std::str::FromStr;
use std::time::Instant;

use anyhow::Context;
use clap::ArgMatches;

use crate::util;

pub fn run(args: &ArgMatches) -> anyhow::Result<()> {
    let path = args.value_of("file").context("Missing source")?;
    let initial_gc_threshold = args
        .value_of("initial-gc-threshold")
        .map(<usize as FromStr>::from_str)
        .transpose()?;

    let source = fs::read_to_string(path).context("Failed to read source")?;
    let opt = util::opt_level_from_matches(args)?;

    let before = args.is_present("timing").then(Instant::now);

    let async_rt = tokio::runtime::Runtime::new()?;
    async_rt.block_on(inner(source, opt, args.is_present("quiet"), initial_gc_threshold))?;

    if let Some(before) = before {
        println!("\n{:?}", before.elapsed());
    }

    Ok(())
}

async fn inner(source: String, opt: OptLevel, quiet: bool, initial_gc_threshold: Option<usize>) -> anyhow::Result<()> {
    let mut rt = Runtime::new(initial_gc_threshold).await;

    let module = dash_rt_modules::init_modules();
    rt.set_module_manager(module);

    let mut scope = rt.vm_mut().scope();
    let value = match scope.eval(&source, opt) {
        Ok(val) => val.root(&mut scope),
        Err(EvalError::Exception(val)) => val, // TODO: this should really also be Unrooted
        Err(e) => {
            println!("{e}");
            return Ok(());
        }
    };

    scope.process_async_tasks();

    // TODO: EvalError::VmError should probably bail too?

    if !quiet {
        util::print_value(value, &mut scope).unwrap();
    }

    let state = State::from_vm(&scope);
    if state.needs_event_loop() {
        drop(scope);
        rt.run_event_loop().await;
    }

    Ok(())
}
