use anyhow::Context;
use clap::ArgMatches;
use dash_optimizer::OptLevel;
use dash_vm::localscope::LocalScope;
use dash_vm::value::ops::abstractions::conversions::ValueConversion;
use dash_vm::value::Value;
use dash_vm::Vm;

pub fn opt_level_from_matches(args: &ArgMatches) -> anyhow::Result<OptLevel> {
    args.value_of("opt")
        .and_then(OptLevel::from_level)
        .context("Invalid opt level")
}

pub fn print_value(value: Value, vm: &mut Vm) -> Result<(), Value> {
    let mut scope = vm.scope();
    let s = value.to_string(&mut scope)?;
    println!("{s}");
    Ok(())
}
