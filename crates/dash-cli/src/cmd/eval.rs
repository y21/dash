use dash::vm::Vm;
use dash_core as dash;

use anyhow::bail;
use anyhow::Context;
use clap::ArgMatches;

use crate::util;

pub fn eval(args: &ArgMatches) -> anyhow::Result<()> {
    let source = args.value_of("source").context("Missing source")?;
    let opt = util::opt_level_from_matches(args)?;

    let mut vm = Vm::new(Default::default());

    match vm.eval(source, opt) {
        Ok(value) | Err(dash::EvalError::VmError(value)) => util::print_value(value, &mut vm).unwrap(),
        Err(e) => bail!("{e}"),
    };

    Ok(())
}
