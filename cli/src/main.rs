use std::{
    borrow::Cow,
    fs,
    io::{self, Write},
    path::PathBuf,
};

use dash::{agent::Agent, vm::value::Value};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "dash")]
struct Args {
    #[structopt(name = "file", parse(from_os_str))]
    file: Option<PathBuf>,
    #[structopt(name = "eval", short = "e")]
    eval: Option<String>,
}

fn create_agent() -> impl Agent {
    runtime::agent(runtime::agent_flags::FS)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Args::from_args();

    if let Some(file) = &opt.file {
        let file = file.to_str().expect("Failed to parse file input string");

        let code = fs::read_to_string(file)?;

        if let Err((e, _vm)) = dash::eval(&code, Some(create_agent())) {
            println!("{}", e.to_string());
        }
    } else if let Some(eval) = &opt.eval {
        if let Err((e, _vm)) = dash::eval(eval, Some(create_agent())) {
            println!("{}", e.to_string());
        }
    } else {
        repl();
    }

    Ok(())
}

fn repl() {
    println!("Welcome to the dash REPL\nType JavaScript code and hit enter to evaluate it");

    loop {
        print!("> ");
        io::stdout().flush().expect("Failed to flush stdout");

        let s = &mut String::new();
        io::stdin().read_line(s).expect("Failed to read line");

        match dash::eval(s, Some(create_agent())) {
            Ok((result, _vm)) => {
                let result_ref = result.as_ref().map(|x| unsafe { x.borrow_unbounded() });
                let result_fmt = result_ref
                    .as_deref()
                    .map(|v| Value::inspect(v, 0))
                    .unwrap_or(Cow::Borrowed("undefined"));

                println!("{}", result_fmt);
            }
            Err((e, _vm)) => {
                println!("{}", e.to_string());
            }
        };
    }
}
