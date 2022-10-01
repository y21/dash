use clap::Arg;
use clap::Command;

mod cmd;
mod util;

fn main() -> anyhow::Result<()> {
    let app = Command::new("testrunner")
        .about("Test coverage")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .override_help("Runs the testrunner")
                .arg(Arg::new("path").long("path").takes_value(true))
                .arg(Arg::new("verbose").long("verbose").takes_value(false)),
        );

    match app.get_matches().subcommand() {
        Some(("run", args)) => cmd::run(args),
        _ => unreachable!(),
    }
}
