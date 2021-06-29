use std::{
    ffi::OsString,
    fs::DirEntry,
    io::{self, Read},
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

// Some tests are ignored because they either cause a segmentation fault or get stuck in an infinite loop
// which causes the testrunner to never finish
const IGNORED_TESTS: &[&str] = &[
    // stuck in infinite loop
    "language/block-scope/leave/for-loop-block-let-declaration-only-shadows-outer-parameter-value-1.js",
    "language/block-scope/leave/verify-context-in-for-loop-block.js",
    "language/module-code/instn-resolve-err-syntax-1_FIXTURE.js",
    "language/module-code/instn-resolve-order-depth-syntax_FIXTURE.js",
    "language/module-code/instn-resolve-order-src-syntax_FIXTURE.js",
    "language/statements/continue/12.7-1.js",
    "language/statements/for/12.6.3_2-3-a-ii-14.js",
    "language/statements/for-in/identifier-let-allowed-as-lefthandside-expression-not-strict.js"
];

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

        if IGNORED_TESTS
            .iter()
            .any(|t| path.to_str().unwrap().ends_with(t))
        {
            continue;
        }

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

enum RunResult {
    Pass,
    Fail,
    Panic,
}

impl RunResult {
    pub fn from_pass(pass: bool) -> Self {
        if pass {
            Self::Pass
        } else {
            Self::Fail
        }
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, RunResult::Fail | RunResult::Panic)
    }
}

/// Runs a test case
async fn run_test(path: OsString, mode: Mode, verbose: bool) -> RunResult {
    let path_str = path.to_str().unwrap();
    let source = tokio::fs::read_to_string(path_str).await.unwrap();

    let maybe_pass = std::panic::catch_unwind(|| {
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

        pass
    });

    let pass = match maybe_pass {
        Ok(pass) => RunResult::from_pass(pass),
        _ => RunResult::Panic,
    };

    if verbose {
        if pass.is_fail() {
            println!("{} {}", style(path_str).red(), style("did not pass").red());
        } else {
            println!("{} {}", style(path_str).green(), style("passed").green());
        }
    }

    pass
}

#[derive(StructOpt, Debug)]
struct Args {
    /// Path to test262
    #[structopt(name = "path", parse(from_os_str))]
    path: Option<PathBuf>,
    /// Only test parsing
    #[structopt(long = "parser")]
    parser: bool,
    /// Only test lexing
    #[structopt(long = "lexer")]
    lexer: bool,
    /// Only test compiling
    #[structopt(long = "compiler")]
    compiler: bool,
    /// Verbose testing (prints test path and other debugging information)
    #[structopt(long = "verbose", short = "v")]
    verbose: bool,
    /// Disables multithreaded testing. Can help with debugging.
    #[structopt(long = "singlethreaded", short = "st")]
    single_threaded: bool,
    /// "Pauses" after a failed test and waits for the user to hit return
    #[structopt(long = "step")]
    step: bool,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Lex,
    Parse,
    Compile,
    Interpret,
}

impl Mode {
    pub fn to_stages(&self) -> &[&'static str] {
        match self {
            Self::Lex => &["Lex"],
            Self::Parse => &["Lex", "Parse"],
            Self::Compile => &["Lex", "Parse", "Compile"],
            Self::Interpret => &["Lex", "Parse", "Compile", "Interpret"],
        }
    }
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
    panic: AtomicU32,
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
    let total_tests = tests.len();

    if opt.step && !opt.single_threaded {
        eprintln!("Step mode may only be used in singlethreaded mode");
        return;
    }

    if opt.verbose {
        println!("Running {} tests in 3 seconds", tests.len());
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    let counter = Arc::new(Counter {
        pass: AtomicU32::new(0),
        fail: AtomicU32::new(0),
        panic: AtomicU32::new(0),
    });

    let mut futures = Vec::new();

    let hook = std::panic::take_hook();
    if !opt.verbose {
        std::panic::set_hook(Box::new(|_info| {}));
    }

    for test in tests {
        let verbose = opt.verbose;
        let single_threaded = opt.single_threaded;
        let step = opt.step;
        let counter = Arc::clone(&counter);

        let fut = async move {
            let result = run_test(test, mode, verbose).await;

            let count: &AtomicU32;

            match result {
                RunResult::Pass => count = &counter.pass,
                RunResult::Fail => count = &counter.fail,
                RunResult::Panic => count = &counter.panic,
            };

            count.fetch_add(1, Ordering::Relaxed);

            if result.is_fail() && step {
                io::stdin().read(&mut [0]).unwrap();
            }
        };

        if single_threaded {
            fut.await;
        } else {
            futures.push(fut);
        }
    }

    futures::future::join_all(futures).await;

    std::panic::set_hook(hook);

    let pass = counter.pass.load(Ordering::Relaxed);
    let fail = counter.fail.load(Ordering::Relaxed);
    let panic = counter.panic.load(Ordering::Relaxed);
    let ignored = IGNORED_TESTS.len();

    println!(
        "Stages: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n-------\nConformance: {:.2}%",
        mode.to_stages().join(", "),
        style("Passed").green(),
        pass,
        style("Failed").yellow(),
        fail,
        style("Panics").red(),
        panic,
        style("Ignored").magenta(),
        ignored,
        (pass as f32 / total_tests as f32) * 100f32
    );
}
