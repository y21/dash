use dash::vm::local::LocalScope;
use dash::vm::value::ops::abstractions::conversions::ValueConversion;
use dash_core as dash;

use anyhow::bail;
use anyhow::Context;
use clap::ArgMatches;

use crate::util;

pub fn eval(args: &ArgMatches) -> anyhow::Result<()> {
    let source = args.value_of("source").context("Missing source")?;
    let opt = util::opt_level_from_matches(args)?;

    match dash::eval(source, opt) {
        Ok((mut vm, value)) => {
            let mut scope = LocalScope::new(&mut vm);
            println!("{}", value.to_string(&mut scope).unwrap());
        }
        Err(err) => bail!("Error: {}", err),
    }

    Ok(())
}
