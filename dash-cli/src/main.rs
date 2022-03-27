use anyhow::bail;
use clap::Arg;
use clap::Command;

mod cmd;

fn main() -> anyhow::Result<()> {
    let app = Command::new("dash")
        .about("Execute JavaScript code using the dash JavaScript engine")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("eval")
                .override_help("Evaluate a JavaScript source string")
                .arg(Arg::new("source").required(true)),
        )
        .subcommand(
            Command::new("run")
                .override_help("Run a JavaScript file")
                .arg(Arg::new("file").required(true))
                .arg(
                    Arg::new("timing")
                        .short('t')
                        .long("timing")
                        .takes_value(false),
                ),
        )
        .subcommand(Command::new("repl").override_help("Enter a JavaScript REPL"));

    let matches = app.get_matches();
    match matches.subcommand() {
        Some(("eval", args)) => cmd::eval(args),
        Some(("run", args)) => cmd::run(args),
        Some(("repl", _)) => cmd::repl(),
        _ => bail!("Unimplemented command"),
    }
}
