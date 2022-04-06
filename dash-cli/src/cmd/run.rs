use std::fs;
use std::time::Instant;

use anyhow::bail;
use anyhow::Context;
use clap::ArgMatches;
use dash::vm::local::LocalScope;
use dash::vm::value::ops::abstractions::conversions::ValueConversion;

use crate::util;

pub fn run(args: &ArgMatches) -> anyhow::Result<()> {
    let path = args.value_of("file").context("Missing source")?;
    let source = fs::read_to_string(path).context("Failed to read source")?;
    let opt = util::opt_level_from_matches(args)?;

    let before = args.is_present("timing").then(|| Instant::now());

    match dash::eval(&source, opt) {
        Ok((mut vm, value)) => {
            let mut sc = LocalScope::new(&mut vm);
            println!("{}", value.to_string(&mut sc).unwrap());
        }
        Err(err) => bail!("{}", err),
    }

    if let Some(before) = before {
        println!("{:?}", before.elapsed());
    }

    Ok(())
}
