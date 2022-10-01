use anyhow::bail;
use clap::Arg;
use clap::Command;

mod cmd;
mod util;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let opt_level = Arg::new("opt")
        .short('o')
        .long("opt")
        .default_value("1")
        .possible_values(["0", "1", "2"]);

    let app = Command::new("dash")
        .about("Execute JavaScript code using the dash JavaScript engine")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("eval")
                .override_help("Evaluate a JavaScript source string")
                .arg(Arg::new("source").required(true))
                .arg(opt_level.clone()),
        )
        .subcommand(
            Command::new("run")
                .override_help("Run a JavaScript file")
                .arg(Arg::new("file").required(true))
                .arg(Arg::new("timing").short('t').long("timing").takes_value(false))
                .arg(Arg::new("quiet").short('q').long("quiet").takes_value(false))
                .arg(opt_level.clone()),
        )
        .subcommand(Command::new("repl").override_help("Enter a JavaScript REPL"))
        .subcommand(
            Command::new("dump")
                .override_help("Dumps intermediate code representation")
                .arg(Arg::new("file").required(true))
                .arg(Arg::new("ir").long("ir").takes_value(false))
                .arg(Arg::new("ast").long("ast").takes_value(false))
                .arg(Arg::new("js").long("js").takes_value(false))
                .arg(Arg::new("bytecode").long("bytecode").takes_value(false))
                .arg(opt_level),
        );

    let matches = app.get_matches();
    match matches.subcommand() {
        Some(("eval", args)) => cmd::eval(args),
        Some(("run", args)) => cmd::run(args),
        Some(("repl", _)) => cmd::repl(),
        Some(("dump", args)) => cmd::dump(args),
        _ => bail!("Unimplemented command"),
    }
}
