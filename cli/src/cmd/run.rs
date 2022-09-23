use dash_optimizer::OptLevel;
use dash_rt::module::ModuleLoader;
use dash_rt::runtime::Runtime;
use dash_rt::state::State;
use dash_vm::eval::EvalError;
use std::fs;
use std::time::Instant;

use anyhow::Context;
use clap::ArgMatches;

use crate::util;

pub fn run(args: &ArgMatches) -> anyhow::Result<()> {
    let path = args.value_of("file").context("Missing source")?;
    let source = fs::read_to_string(path).context("Failed to read source")?;
    let opt = util::opt_level_from_matches(args)?;

    let before = args.is_present("timing").then(|| Instant::now());

    let async_rt = tokio::runtime::Runtime::new()?;
    async_rt.block_on(inner(source, opt, args.is_present("quiet")))?;

    if let Some(before) = before {
        println!("{:?}", before.elapsed());
    }

    Ok(())
}

async fn inner(source: String, opt: OptLevel, quiet: bool) -> anyhow::Result<()> {
    let mut rt = Runtime::new().await;

    let module = dash_rt_http::HttpModule
        .or(dash_rt_fs::FsModule)
        .or(dash_rt_fetch::FetchModule);

    rt.set_module_manager(Box::new(module));

    let value = match rt.eval(&source, opt) {
        Ok(val) | Err(EvalError::Exception(val)) => val,
        Err(e) => {
            println!("{e}");
            return Ok(());
        }
    };

    rt.vm_mut().process_async_tasks();

    // TODO: EvalError::VmError should probably bail too?

    if !quiet {
        util::print_value(value, rt.vm_mut()).unwrap();
    }

    let state = State::from_vm(rt.vm());
    if state.needs_event_loop() {
        rt.run_event_loop().await;
    }

    Ok(())
}
