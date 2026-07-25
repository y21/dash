use std::collections::HashMap;
use std::panic;
use std::sync::atomic::AtomicU32;
use std::sync::{Mutex, MutexGuard, atomic};

use anyhow::Context;
use bumpalo::Bump;
use clap::ArgMatches;
use dash_vm::Vm;
use dash_vm::eval::EvalError;
use dash_vm::params::VmParams;
use dash_vm::value::Root;
use dash_vm::value::ops::conversions::ValueConversion;
use once_cell::sync::Lazy;
use owo_colors::{Style, Styled};
use rayon::ThreadPoolBuilder;
use serde::Deserialize;

use crate::cmd::differ::{diff_results_to_previous, strip_test262_prefix};
use crate::cmd::results::ResultsMap;
use crate::util;

pub fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    let bump = Bump::new();

    let path = matches.get_one::<String>("path");
    let path = path.map_or("../test262/test", |v| &**v);
    let verbose = *matches.get_one::<bool>("verbose").unwrap();
    let single_threaded = *matches.get_one::<bool>("disable-threads").unwrap();
    let files = if path.ends_with(".js") {
        vec![&*bump.alloc_str(path)]
    } else {
        util::get_all_files(&bump, path)?
    };

    run_inner(&bump, files, verbose, single_threaded)?;

    Ok(())
}

#[derive(Debug)]
pub struct Results<'bump> {
    passes: AtomicU32,
    fails: AtomicU32,
    panics: AtomicU32,
    results: Mutex<ResultsMap<'bump>>,
}

impl<'bump> Results<'bump> {
    pub fn new() -> Self {
        Self {
            passes: AtomicU32::new(0),
            fails: AtomicU32::new(0),
            panics: AtomicU32::new(0),
            results: Mutex::new(ResultsMap::new(ResultsMap::DEFAULT_CAPACITY)),
        }
    }

    pub fn register(&self, path: &'bump str, result: RunResult) {
        match result {
            RunResult::Pass => self.passes.fetch_add(1, atomic::Ordering::Relaxed),
            RunResult::Fail => self.fails.fetch_add(1, atomic::Ordering::Relaxed),
            RunResult::Panic => self.panics.fetch_add(1, atomic::Ordering::Relaxed),
        };

        self.results.lock().unwrap().insert(path, result);
    }

    pub fn results_map(&self) -> MutexGuard<'_, ResultsMap<'bump>> {
        self.results.lock().unwrap()
    }
}

fn run_inner<'bump>(
    bump: &'bump Bump,
    files: Vec<&'bump str>,
    verbose: bool,
    single_threaded: bool,
) -> anyhow::Result<()> {
    let setup: String = {
        let sta = std::fs::read_to_string("../test262/harness/sta.js")?;
        let assert = std::fs::read_to_string("../test262/harness/assert.js")?;

        let code = format!("{sta};\n{assert};\n");
        code
    };

    let results = Results::new();
    let file_count = files.len();

    let run_file = |file: &'bump str| {
        let result = run_test(&setup, file, verbose);

        results.register(strip_test262_prefix(file).unwrap_or(file), result);
    };

    if single_threaded {
        for file in files {
            run_file(&file);
        }
    } else {
        let tp = ThreadPoolBuilder::default().stack_size(8_000_000).build()?;
        tp.scope(|s| {
            for file in files {
                s.spawn(move |_| {
                    run_file(&file);
                });
            }
        });
    }

    let passes = results.passes.load(atomic::Ordering::Relaxed);
    let fails = results.fails.load(atomic::Ordering::Relaxed);
    let panics = results.panics.load(atomic::Ordering::Relaxed);
    let rate = ((passes as f32) / (file_count as f32)) * 100.0;
    println!("== Result ===");
    println!("Passes: {passes} ({rate:.2}%)",);
    println!("Fails: {fails}");
    println!("Panics: {panics}");

    diff_results_to_previous(bump, &results).context("diffing results to previous")?;
    Ok(())
}

macro_rules! define_run_result_enum {
    (
        $($variant:ident = $value:expr),*
    ) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum RunResult {
            $($variant = $value),*
        }

        impl RunResult {
            pub fn from_u8(value: u8) -> Option<Self> {
                match value {
                    $($value => Some(RunResult::$variant),)*
                    _ => None,
                }
            }
        }
    };
}
define_run_result_enum! {
    Pass = 0,
    Fail = 1,
    Panic = 2
}

impl RunResult {
    pub fn styled(self) -> Styled<&'static str> {
        match self {
            RunResult::Pass => Style::new().green().style("OK"),
            RunResult::Fail => Style::new().red().style("FAIL"),
            RunResult::Panic => Style::new().red().bright_yellow().style("PANIC"),
        }
    }
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

fn run_test(setup: &str, path: &str, verbose: bool) -> RunResult {
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
                            EvalError::Exception(ex) => {
                                let mut scope = vm.scope();
                                let ex = ex.root(&mut scope);
                                ex.to_js_string(&mut scope)
                                    .map(|s| s.res(&scope).to_owned())
                                    .unwrap_or_else(|_| "<js error>".into())
                            }
                        };
                        println!("Error in {:?}: {s}", path);
                    }
                }

                result
            }
        }
    });

    match maybe_pass {
        Ok(res) => res,
        Err(_) => {
            println!("Panic in {}", path);
            RunResult::Panic
        }
    }
}
