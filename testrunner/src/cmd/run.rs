use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::panic;
use std::sync::atomic;
use std::sync::atomic::AtomicU32;
use std::sync::Mutex;

use clap::ArgMatches;
use dash_vm::eval::EvalError;
use dash_vm::params::VmParams;
use dash_vm::Vm;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::util;

pub fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    let path = matches.value_of("path").unwrap_or("../test262/test");
    let verbose = matches.is_present("verbose");
    let files = util::get_all_files(OsStr::new(path))?;

    run_inner(files, verbose)?;

    Ok(())
}

fn run_inner(files: Vec<OsString>, verbose: bool) -> anyhow::Result<()> {
    let setup: String = {
        let sta = std::fs::read_to_string("../test262/harness/sta.js")?;
        let assert = std::fs::read_to_string("../test262/harness/assert.js")?;

        let code = format!("{sta};\n{assert};\n");
        code
    };

    #[derive(Default)]
    struct Counter {
        passes: AtomicU32,
        fails: AtomicU32,
        panics: AtomicU32,
    }

    let counter = Counter::default();
    let file_count = files.len();

    let tp = rayon::ThreadPoolBuilder::default().stack_size(8_000_000).build()?;
    tp.scope(|s| {
        for file in files {
            s.spawn(|_| {
                #[allow(clippy::redundant_locals)] // it's not redundant
                let file = file;
                let result = run_test(&setup, &file, verbose);

                let counter = match result {
                    RunResult::Pass => &counter.passes,
                    RunResult::Fail => &counter.fails,
                    RunResult::Panic => &counter.panics,
                };

                counter.fetch_add(1, atomic::Ordering::Relaxed);
            });
        }
    });

    let passes = counter.passes.load(atomic::Ordering::Relaxed);
    let fails = counter.fails.load(atomic::Ordering::Relaxed);
    let panics = counter.panics.load(atomic::Ordering::Relaxed);
    let rate = ((passes as f32) / (file_count as f32)) * 100.0;
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

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum NegativePhase {
    Parse,
    Resolution,
    Runtime,
}

#[derive(Deserialize)]
struct NegativeMetadata {
    #[allow(unused)]
    phase: NegativePhase,
    #[serde(rename = "type")]
    #[allow(unused)]
    ty: String,
}

#[derive(Deserialize)]
struct YamlMetadata {
    includes: Option<Vec<String>>,
    negative: Option<NegativeMetadata>,
}

fn extract_yaml_metadata(source: &str) -> Option<YamlMetadata> {
    let start = source.find("/*---")?;
    let end = source[start..].find("---*/")?;
    let full = &source[start + 6..start + end];
    let value = serde_yaml::from_str(full).unwrap();
    Some(value)
}

fn get_harness_code(path: &str) -> String {
    static CACHE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));
    let mut lock = CACHE.lock().unwrap();
    let code = lock
        .entry(path.into())
        .or_insert_with(|| std::fs::read_to_string(path).unwrap());
    code.clone()
}

fn run_test(setup: &str, path: &OsStr, verbose: bool) -> RunResult {
    let mut negative = None;
    let contents = std::fs::read_to_string(path).unwrap();
    let mut prelude = String::from(setup);
    if let Some(metadata) = extract_yaml_metadata(&contents) {
        if let Some(includes) = metadata.includes {
            for include in includes {
                let patched_file = format!("../test262/harness/{include}");
                prelude += &get_harness_code(&patched_file);
            }
        }
        negative = metadata.negative;
    }
    let contents = format!("{prelude}{contents}");

    let maybe_pass = panic::catch_unwind(move || {
        let mut vm = Vm::new(VmParams::default());
        match (vm.eval(&contents, Default::default()), negative.map(|n| n.phase)) {
            (Ok(_), None) => RunResult::Pass,
            (Ok(_), Some(..)) => RunResult::Fail,
            (Err(err), negative) => {
                let result = match (&err, negative) {
                    (EvalError::Middle(..), Some(NegativePhase::Parse | NegativePhase::Resolution)) => RunResult::Pass,
                    (EvalError::Middle(..), None) => RunResult::Fail,
                    (EvalError::Exception(..), Some(NegativePhase::Runtime)) => RunResult::Pass,
                    (EvalError::Exception(..), None) => RunResult::Fail,
                    (_, Some(..)) => RunResult::Fail,
                };

                if let RunResult::Fail = result {
                    if verbose {
                        let s = match &err {
                            EvalError::Middle(errs) => format!("{errs:?}"),
                            EvalError::Exception(_ex) => {
                                // let mut sc = LocalScope::new(&mut vm);
                                // match ex.to_string(&mut sc) {
                                //     Ok(s) => ToString::to_string(&s),
                                //     Err(err) => format!("{err:?}"),
                                // }

                                // displaying certain JS error "structures" like above causes a weird stack overflow.
                                // requires further investigation. for now just display some hardcoded string
                                "<js error>".into()
                            }
                        };
                        println!("Error in {:?}: {s}", path.to_str());
                    }
                }

                result
            }
        }
    });

    match maybe_pass {
        Ok(res) => res,
        Err(_) => {
            println!("Panic in {}", path.to_str().unwrap());
            RunResult::Panic
        }
    }
}
