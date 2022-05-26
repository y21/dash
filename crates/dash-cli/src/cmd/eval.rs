use anyhow::bail;
use anyhow::Context;
use clap::ArgMatches;
use dash_vm::eval::EvalError;
use dash_vm::Vm;

use crate::util;

pub fn eval(args: &ArgMatches) -> anyhow::Result<()> {
    let source = args.value_of("source").context("Missing source")?;
    let opt = util::opt_level_from_matches(args)?;

    let mut vm = Vm::new(Default::default());

    match vm.eval(source, opt) {
        Ok(value) | Err(EvalError::Exception(value)) => util::print_value(value, &mut vm).unwrap(),
        Err(e) => bail!("{e:?}"),
    };

    Ok(())
}
