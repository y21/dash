use std::ffi::OsStr;
use std::ffi::OsString;
use std::panic;
use std::sync::atomic;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use clap::ArgMatches;
use dash_vm::eval::EvalError;
use dash_vm::local::LocalScope;
use dash_vm::params::VmParams;
use dash_vm::value::ops::abstractions::conversions::ValueConversion;
use dash_vm::Vm;
use futures_util::future;
use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;

use crate::util;

pub fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    let path = matches.value_of("path").unwrap_or("../test262/test");
    let verbose = matches.is_present("verbose");
    let files = util::get_all_files(OsStr::new(path))?;

    let tokio = tokio::runtime::Runtime::new()?;
    tokio.block_on(run_inner(files, verbose))?;

    Ok(())
}

async fn run_inner(files: Vec<OsString>, verbose: bool) -> anyhow::Result<()> {
    let setup: Arc<str> = {
        let sta = tokio::fs::read_to_string("../test262/harness/sta.js");
        let assert = tokio::fs::read_to_string("../test262/harness/assert.js");

        let (sta, assert) = future::join(sta, assert).await;

        let code = format!("{};\n{};\n", sta?, assert?);
        code.into()
    };

    #[derive(Default)]
    struct Counter {
        passes: AtomicU32,
        fails: AtomicU32,
        panics: AtomicU32,
    }

    let counter = Arc::new(Counter::default());

    for files in files.chunks(4) {
        let mut futs = FuturesUnordered::new();

        for file in files {
            let setup = Arc::clone(&setup);
            let counter = Arc::clone(&counter);

            let fut = async move {
                let result = run_test(&setup, file, verbose).await;

                let counter = match result {
                    RunResult::Pass => &counter.passes,
                    RunResult::Fail => &counter.fails,
                    RunResult::Panic => &counter.panics,
                };

                counter.fetch_add(1, atomic::Ordering::Relaxed);
            };

            futs.push(fut);
        }

        while let Some(()) = futs.next().await {}
    }

    let passes = counter.passes.load(atomic::Ordering::Relaxed);
    let fails = counter.fails.load(atomic::Ordering::Relaxed);
    let panics = counter.panics.load(atomic::Ordering::Relaxed);
    let rate = ((passes as f32) / (files.len() as f32)) * 100.0;
    println!("== Result ===");
    println!("Passes: {passes} ({rate:.2}%)",);
    println!("Fails: {fails}");
    println!("Panics: {panics}");

    Ok(())
}

#[derive(Debug)]
enum RunResult {
    Pass,
    Fail,
    Panic,
}

async fn run_test(setup: &str, path: &OsStr, verbose: bool) -> RunResult {
    let contents = tokio::fs::read_to_string(path).await.unwrap();
    let contents = format!("{setup}{contents}");

    let maybe_pass = panic::catch_unwind(move || {
        let mut vm = Vm::new(VmParams::default());
        match vm.eval(&contents, Default::default()) {
            Ok(_) => RunResult::Pass,
            Err(err) => {
                if verbose {
                    let s = match err {
                        EvalError::Compiler(c) => c.to_string(),
                        EvalError::Lexer(l) => format!("{l:?}"),
                        EvalError::Parser(p) => format!("{p:?}"),
                        EvalError::Exception(ex) => {
                            let mut sc = LocalScope::new(&mut vm);
                            match ex.to_string(&mut sc) {
                                Ok(s) => ToString::to_string(&s),
                                Err(err) => format!("{err:?}"),
                            }
                        }
                    };
                    println!("Error in {:?}: {s}", path.to_str());
                }
                RunResult::Fail
            }
        }
    });

    match maybe_pass {
        Ok(pass) => pass,
        Err(_) => RunResult::Panic,
    }
}
