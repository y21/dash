use clap::{App, SubCommand};

fn main() {
    let args = App::new("Dash")
        .about("Execute JavaScript code using the JavaScript engine dash")
        .subcommand(SubCommand::with_name("run"))
        .get_matches();
}
