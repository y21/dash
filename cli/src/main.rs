use std::backtrace::{Backtrace, BacktraceStatus};

use anyhow::{Context, bail};
use clap::{Arg, Command};
use colorful::Colorful;
use dash_optimizer::OptLevel;

mod cmd;
mod util;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let opt_level = Arg::new("opt")
        .short('O')
        .long("opt")
        .value_parser(|val: &_| OptLevel::from_level(val).context("unknown opt level"))
        .default_value("1")
        .possible_values(["0", "1", "2"]);

    let nodejs = Arg::new("node").long("node").takes_value(false);

    let initial_gc_threshold = Arg::new("initial-gc-threshold")
        .help("Sets the initial GC object threshold, i.e. the RSS at which the first GC cycle triggers.")
        .long("initial-gc-threshold")
        .takes_value(true)
        .required(false);

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
                .arg(opt_level.clone())
                .arg(nodejs)
                .arg(initial_gc_threshold.clone()),
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
                .arg(Arg::new("tokens").long("tokens").takes_value(false))
                .arg(Arg::new("types").long("types").takes_value(false))
                .arg(opt_level),
        );

    std::panic::set_hook(Box::new(|info| {
        eprintln!("{}\n", "dash has unexpectedly panicked! this is a bug!".red().bold());

        eprintln!("{info}");

        let backtrace = Backtrace::capture();
        match backtrace.status() {
            BacktraceStatus::Captured => {
                eprintln!("--- begin of backtrace ---");
                eprintln!("{backtrace}");
            }
            BacktraceStatus::Disabled => {
                eprintln!("set RUST_BACKTRACE=1 to print a backtrace");
            }
            BacktraceStatus::Unsupported => {
                eprintln!("backtraces are not supported on this platform");
            }
            _ => {
                eprintln!("backtraces are not available");
            }
        }
    }));

    let matches = app.get_matches();
    match matches.subcommand() {
        Some(("eval", args)) => cmd::eval(args),
        Some(("run", args)) => cmd::run(args),
        Some(("repl", _)) => cmd::repl(),
        Some(("dump", args)) => cmd::dump(args),
        _ => bail!("Unimplemented command"),
    }
}
