use anyhow::bail;
use anyhow::Context;
use clap::ArgMatches;

pub fn eval(args: &ArgMatches) -> anyhow::Result<()> {
    let source = args.value_of("source").context("Missing source")?;

    match dash::eval(source) {
        Ok((_vm, value)) => {
            println!("{:?}", value);
        }
        Err(err) => bail!("{}", err),
    }

    Ok(())
}
