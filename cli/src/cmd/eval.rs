use anyhow::Context;
use clap::ArgMatches;
use dash_middle::parser::error::IntoFormattableErrors;
use dash_rt::format_value;
use dash_vm::eval::EvalError;
use dash_vm::value::Root;
use dash_vm::Vm;

use crate::util;

pub fn eval(args: &ArgMatches) -> anyhow::Result<()> {
    let source = args.value_of("source").context("Missing source")?;
    let opt = util::opt_level_from_matches(args)?;

    let mut vm = Vm::new(Default::default());
    let mut scope = vm.scope();

    match scope.eval(source, opt) {
        Ok(value) => println!("{}", format_value(value.root(&mut scope), &mut scope).unwrap()),
        Err(EvalError::Exception(value)) => {
            println!("{}", format_value(value.root(&mut scope), &mut scope).unwrap())
        }
        Err(EvalError::Middle(errs)) => println!("{}", errs.formattable(source, true)),
    };

    scope.process_async_tasks();

    Ok(())
}
