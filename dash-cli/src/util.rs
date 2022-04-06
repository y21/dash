use anyhow::Context;
use clap::ArgMatches;
use dash::optimizer::consteval::OptLevel;

pub fn opt_level_from_matches(args: &ArgMatches) -> anyhow::Result<OptLevel> {
    args.value_of("opt")
        .and_then(OptLevel::from_level)
        .context("Invalid opt level")
}
