use anyhow::Context;
use clap::ArgMatches;
use dash_optimizer::OptLevel;

pub fn opt_level_from_matches(args: &ArgMatches) -> anyhow::Result<OptLevel> {
    args.value_of("opt")
        .and_then(OptLevel::from_level)
        .context("Invalid opt level")
}
