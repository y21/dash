use std::{ffi::OsString, fs::DirEntry, io, path::PathBuf, str::FromStr};

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
fn run_test(path: OsString, mode: Mode) {
    let path_str = path.to_str().unwrap();
    let source = std::fs::read_to_string(path_str).unwrap();

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

    if pass {
        println!("{} {}", style(path_str).green(), style("passed").green());
    } else {
        println!("{} {}", style(path_str).red(), style("did not pass").red());
    }
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

fn main() {
    let opt = Args::from_args();

    let path = opt
        .path
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("../test262/test");

    let mode = Mode::from_args(&opt);

    let tests = get_all_files(&OsString::from_str(&path).unwrap()).unwrap();

    for test in tests {
        run_test(test, mode);
    }
}
