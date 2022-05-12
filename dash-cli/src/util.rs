use dash::vm::local::LocalScope;
use dash::vm::value::ops::abstractions::conversions::ValueConversion;
use dash::vm::value::Value;
use dash::vm::Vm;
use dash_core as dash;

use anyhow::Context;
use clap::ArgMatches;
use dash::optimizer::consteval::OptLevel;

pub fn opt_level_from_matches(args: &ArgMatches) -> anyhow::Result<OptLevel> {
    args.value_of("opt")
        .and_then(OptLevel::from_level)
        .context("Invalid opt level")
}

pub fn print_value(value: Value, vm: &mut Vm) -> Result<(), Value> {
    let mut scope = LocalScope::new(vm);
    let s = value.to_string(&mut scope)?;
    println!("{s}");
    Ok(())
}
