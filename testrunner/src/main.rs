use std::{
    ffi::OsString,
    fs::DirEntry,
    io,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use console::style;
use dash::{
    compiler::compiler::Compiler,
    parser::{lexer::Lexer, parser::Parser},
};
use structopt::StructOpt;

/// Returns a vector of path strings
fn get_all_files(dir: &OsString) -> io::Result<Vec<OsString>> {
    let mut ve = Vec::new();

    let read_dir = std::fs::read_dir(dir)?;

    for entry in read_dir {
        let entry: DirEntry = entry?;

        let path = OsString::from(format!(
            "{}/{}",
            dir.as_os_str().to_str().unwrap(),
            entry.file_name().as_os_str().to_str().unwrap()
        ));

        let ty = entry.file_type()?;
        if ty.is_file() {
            ve.push(path);
        } else if ty.is_dir() {
            let files = get_all_files(&path)?;
            ve.extend(files);
        }
    }

    Ok(ve)
}

/// Runs a test case
async fn run_test(path: OsString, mode: Mode, verbose: bool) -> bool {
    let path_str = path.to_str().unwrap();
    let source = tokio::fs::read_to_string(path_str).await.unwrap();

    let pass = match mode {
        Mode::Lex => Lexer::new(&source).scan_all().is_ok(),
        Mode::Parse => Lexer::new(&source)
            .scan_all()
            .map(|tok| Parser::new(&source, tok).parse_all().is_ok())
            .unwrap_or(false),
        Mode::Compile => {
            if let Ok(tokens) = Lexer::new(&source).scan_all() {
                if let Ok(stmts) = Parser::new(&source, tokens).parse_all() {
                    Compiler::<()>::new(stmts, None, false).compile().is_ok()
                } else {
                    false
                }
            } else {
                false
            }
        }
        Mode::Interpret => dash::eval::<()>(&source, None).is_ok(),
    };

    if verbose {
        if pass {
            println!("{} {}", style(path_str).green(), style("passed").green());
        } else {
            println!("{} {}", style(path_str).red(), style("did not pass").red());
        }
    }

    pass
}

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(name = "path", parse(from_os_str))]
    path: Option<PathBuf>,
    #[structopt(long = "parser")]
    parser: bool,
    #[structopt(long = "lexer")]
    lexer: bool,
    #[structopt(long = "compiler")]
    compiler: bool,
    #[structopt(long = "verbose", short = "v")]
    verbose: bool,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Lex,
    Parse,
    Compile,
    Interpret,
}

impl Mode {
    pub fn from_args(args: &Args) -> Self {
        if args.compiler {
            Self::Compile
        } else if args.parser {
            Self::Parse
        } else if args.lexer {
            Self::Lex
        } else {
            Self::Interpret
        }
    }
}

struct Counter {
    pass: AtomicU32,
    fail: AtomicU32,
}

#[tokio::main]
async fn main() {
    let opt = Args::from_args();

    let path = opt
        .path
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("../test262/test");

    let mode = Mode::from_args(&opt);

    let tests = get_all_files(&OsString::from_str(&path).unwrap()).unwrap();

    if opt.verbose {
        println!("Running {} tests in 3 seconds", tests.len());
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    let counter = Arc::new(Counter {
        pass: AtomicU32::new(0),
        fail: AtomicU32::new(0),
    });

    let mut futures = Vec::new();

    for test in tests {
        let verbose = opt.verbose;
        let counter = Arc::clone(&counter);
        let fut = async move {
            if run_test(test, mode, verbose).await {
                counter.pass.fetch_add(1, Ordering::Relaxed);
            } else {
                counter.fail.fetch_add(1, Ordering::Relaxed);
            }
        };

        futures.push(fut);
    }

    futures::future::join_all(futures).await;

    let pass = counter.pass.load(Ordering::Relaxed);
    let fail = counter.fail.load(Ordering::Relaxed);
    println!("{} tests passed, {} tests failed", pass, fail);
}
