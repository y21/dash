use anyhow::bail;
use anyhow::Context;
use clap::ArgMatches;

use crate::util;

pub fn eval(args: &ArgMatches) -> anyhow::Result<()> {
    let source = args.value_of("source").context("Missing source")?;
    let opt = util::opt_level_from_matches(args)?;

    match dash::eval(source, opt) {
        Ok((_vm, value)) => {
            println!("{:?}", value);
        }
        Err(err) => bail!("{}", err),
    }

    Ok(())
}
