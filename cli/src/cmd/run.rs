use dash_middle::parser::error::IntoFormattableErrors;
use dash_optimizer::OptLevel;
use dash_rt::format_value;
use dash_rt::runtime::Runtime;
use dash_rt::state::State;
use dash_vm::eval::EvalError;
use dash_vm::value::Root;
use std::fs;
use std::str::FromStr;
use std::time::Instant;

use anyhow::Context;
use clap::ArgMatches;

pub fn run(args: &ArgMatches) -> anyhow::Result<()> {
    let path = args.value_of("file").context("Missing source")?;
    let nodejs = args.is_present("node");
    let initial_gc_threshold = args
        .value_of("initial-gc-threshold")
        .map(<usize as FromStr>::from_str)
        .transpose()?;
    let opt = *args.get_one::<OptLevel>("opt").unwrap();
    let before = args.is_present("timing").then(Instant::now);
    let quiet = args.is_present("quiet");

    if nodejs {
        #[cfg(feature = "nodejs")]
        {
            dash_node_impl::run_with_nodejs_mnemnoics(path, opt, initial_gc_threshold)?;
        }
        #[cfg(not(feature = "nodejs"))]
        {
            anyhow::bail!("dash needs to be compiled with the `nodejs` feature to support node-compat mode");
        }
    } else {
        run_normal_mode(path, opt, quiet, initial_gc_threshold)?;
    }

    if let Some(before) = before {
        println!("\n{:?}", before.elapsed());
    }

    Ok(())
}

fn run_normal_mode(path: &str, opt: OptLevel, quiet: bool, initial_gc_threshold: Option<usize>) -> anyhow::Result<()> {
    let source = fs::read_to_string(path).context("Failed to read source")?;

    let async_rt = tokio::runtime::Runtime::new()?;
    async_rt.block_on(inner(source, opt, quiet, initial_gc_threshold))?;

    Ok(())
}

async fn inner(source: String, opt: OptLevel, quiet: bool, initial_gc_threshold: Option<usize>) -> anyhow::Result<()> {
    let mut rt = Runtime::new(initial_gc_threshold).await;

    let module = dash_rt_modules::init_modules();
    rt.set_module_manager(module);

    let mut scope = rt.vm_mut().scope();
    let value = match scope.eval(&source, opt) {
        Ok(val) => val.root(&mut scope),
        Err(EvalError::Exception(val)) => val.root(&mut scope),
        Err(EvalError::Middle(errs)) => {
            println!("{}", errs.formattable(&source, true));
            return Ok(());
        }
    };

    scope.process_async_tasks();

    // TODO: EvalError::VmError should probably bail too?

    if !quiet {
        println!("{}", format_value(value, &mut scope).unwrap());
    }

    let state = State::from_vm_mut(&mut scope);
    if state.needs_event_loop() {
        drop(scope);
        rt.run_event_loop().await;
    }

    Ok(())
}
