use anyhow::Context;
use clap::ArgMatches;
use dash_vm::eval::EvalError;
use dash_vm::Vm;

use crate::util;

pub fn eval(args: &ArgMatches) -> anyhow::Result<()> {
    let source = args.value_of("source").context("Missing source")?;
    let opt = util::opt_level_from_matches(args)?;

    let mut vm = Vm::new(Default::default());
    let mut scope = vm.scope();

    match scope.eval(source, opt) {
        Ok(value) => util::print_value(value.root(&mut scope), &mut scope).unwrap(),
        Err(EvalError::Exception(value)) => util::print_value(value.root(&mut scope), &mut scope).unwrap(),
        Err(e) => println!("{e}"),
    };

    scope.process_async_tasks();

    Ok(())
}
