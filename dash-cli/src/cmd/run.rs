use std::fs;

use anyhow::bail;
use anyhow::Context;
use clap::ArgMatches;

pub fn run(args: &ArgMatches) -> anyhow::Result<()> {
    let path = args.value_of("file").context("Missing source")?;
    let source = fs::read_to_string(path).context("Failed to read source")?;

    match dash::eval(&source) {
        Ok((_vm, value)) => {
            println!("{:?}", value);
        }
        Err(err) => bail!("{}", err),
    }

    Ok(())
}
