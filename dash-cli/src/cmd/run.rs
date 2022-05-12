use anyhow::bail;
use dash::optimizer::consteval::OptLevel;
use dash::EvalError;
use dash_core as dash;
use dash_rt::runtime::Runtime;
use dash_rt::state::State;
use std::fs;
use std::time::Instant;

use anyhow::Context;
use clap::ArgMatches;
use dash::vm::local::LocalScope;
use dash::vm::value::ops::abstractions::conversions::ValueConversion;

use crate::util;

pub fn run(args: &ArgMatches) -> anyhow::Result<()> {
    let path = args.value_of("file").context("Missing source")?;
    let source = fs::read_to_string(path).context("Failed to read source")?;
    let opt = util::opt_level_from_matches(args)?;

    let before = args.is_present("timing").then(|| Instant::now());

    let async_rt = tokio::runtime::Runtime::new()?;
    async_rt.block_on(inner(source, opt))?;

    if let Some(before) = before {
        println!("{:?}", before.elapsed());
    }

    Ok(())
}

async fn inner(source: String, opt: OptLevel) -> anyhow::Result<()> {
    let mut rt = Runtime::new().await;

    let value = match rt.eval(&source, opt) {
        Ok(val) | Err(EvalError::VmError(val)) => val,
        Err(e) => bail!("{e}"),
    };

    // TODO: EvalError::VmError should probably bail too?

    let mut sc = LocalScope::new(rt.vm_mut());
    println!("{}", value.to_string(&mut sc).unwrap());

    let state = State::try_from_vm(rt.vm()).unwrap();
    if state.needs_event_loop() {
        rt.run_event_loop().await;
    }

    Ok(())
}
