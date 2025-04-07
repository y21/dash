use clap::{Arg, ArgAction, Command};

mod cmd;
mod util;

fn main() -> anyhow::Result<()> {
    let app = Command::new("testrunner")
        .about("Test coverage")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .override_help("Runs the testrunner")
                .arg(Arg::new("path").long("path"))
                .arg(
                    Arg::new("disable-threads")
                        .long("disable-threads")
                        .action(ArgAction::SetTrue)
                        .required(false),
                )
                .arg(Arg::new("verbose").long("verbose").action(ArgAction::SetTrue)),
        );

    match app.get_matches().subcommand() {
        Some(("run", args)) => cmd::run(args),
        _ => unreachable!(),
    }
}
