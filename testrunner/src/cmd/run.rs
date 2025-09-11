use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::panic;
use std::sync::atomic::AtomicU32;
use std::sync::{Mutex, atomic};

use clap::ArgMatches;
use dash_vm::Vm;
use dash_vm::eval::EvalError;
use dash_vm::params::VmParams;
use dash_vm::value::propertykey::PropertyKey;
use dash_vm::value::string::JsString;
use dash_vm::value::{Root, Unpack, ValueKind};
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::util;

pub fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    let path = matches.get_one::<String>("path");
    let path = path.map_or("../test262/test", |v| &**v);
    let slevel = matches.get_one::<String>("level").unwrap_or(&"".to_string()).clone();
    let c = matches.get_one::<bool>("color").unwrap().clone();
    let single_threaded = *matches.get_one::<bool>("disable-threads").unwrap();
    let files = if path.ends_with(".js") {
        vec![OsString::from(path)]
    } else {
        util::get_all_files(OsStr::new(path))?
    };

    let level = slevel.contains('o') as u8 | ((slevel.contains('e') as u8) << 1) | ((slevel.contains('p') as u8) << 2);
    // println!("{level:b}");
    run_inner(files, level, single_threaded, c)?;

    Ok(())
}

fn run_inner(files: Vec<OsString>, level: u8, single_threaded: bool, c: bool) -> anyhow::Result<()> {
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

    let run_file = |file: &OsString| {
        let result = run_test(&setup, file, level, c);

        let counter = match result {
            RunResult::Pass => &counter.passes,
            RunResult::Fail => &counter.fails,
            RunResult::Panic => &counter.panics,
        };

        counter.fetch_add(1, atomic::Ordering::Relaxed);
    };

    if single_threaded {
        for file in files {
            run_file(&file);
        }
    } else {
        let tp = rayon::ThreadPoolBuilder::default().stack_size(8_000_000).build()?;
        tp.scope(|s| {
            for file in files {
                s.spawn(move |_| {
                    run_file(&file);
                });
            }
        });
    }

    let passes = counter.passes.load(atomic::Ordering::Relaxed);
    let fails = counter.fails.load(atomic::Ordering::Relaxed);
    let panics = counter.panics.load(atomic::Ordering::Relaxed);
    let rate = ((passes as f32) / (file_count as f32)) * 100.0;
    println!("== Result ==");
    println!("   {}OK{} {passes} ({rate:.2}%)", ansi(c, 32), ansi(c, 0));
    println!("  {}ERR{} {fails}", ansi(c, 33), ansi(c, 0));
    println!("{}PANIC{} {panics}", ansi(c, 31), ansi(c, 0));

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

fn ansi(color: bool, n: u8) -> String {
    if !color { String::new() } else { format!("\x1b[{n}m") }
}

fn run_test(setup: &str, path: &OsStr, level: u8, c: bool) -> RunResult {
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
    let label = path.to_string_lossy().to_string();
    panic::set_hook(Box::new(move |p| {
        if level & 0b100 != 0 {
            println!(
                "{}PANIC{} {label}: {} {}{}{}",
                ansi(c, 31),
                ansi(c, 0),
                p.payload()
                    .downcast_ref::<&str>()
                    .map(|x| x.to_string())
                    .or_else(|| p.payload().downcast_ref::<String>().cloned())
                    .unwrap(),
                ansi(c, 2),
                p.location().unwrap(),
                ansi(c, 0)
            );
        }
    }));
    let label = path.to_string_lossy().to_string();
    let maybe_pass = panic::catch_unwind(move || {
        let mut vm = Vm::new(VmParams::default());
        match (vm.eval(&contents, Default::default()), negative.map(|n| n.phase)) {
            (Ok(_), None) => {
                if level & 1 != 0 {
                    println!(
                        "   {}OK{} {label}",
                        ansi(c, 32),
                        ansi(c, 0)
                    )
                }
                RunResult::Pass
            }
            (Ok(_), Some(..)) => RunResult::Fail,
            (Err(err), negative) => {
                let result = match (&err, negative) {
                    (EvalError::Middle(..), Some(NegativePhase::Parse | NegativePhase::Resolution)) => RunResult::Pass,
                    (EvalError::Middle(..), None) => RunResult::Fail,
                    (EvalError::Exception(..), Some(NegativePhase::Runtime)) => RunResult::Pass,
                    (EvalError::Exception(..), None) => RunResult::Fail,
                    (_, Some(..)) => RunResult::Fail,
                };

                {
                    match result {
                        RunResult::Pass => {
                            if level & 1 != 0 {
                                println!(
                                    "   {}OK{} {label}",
                                    ansi(c, 32),
                                    ansi(c, 0)
                                )
                            }
                        }
                        RunResult::Fail => {
                            let s = match &err {
                                EvalError::Middle(errs) => format!("{errs:?}"),
                                EvalError::Exception(ex) => {
                                    let mut scope = vm.scope();
                                    let t = ex.root(&mut scope);
                                    let tos = scope.interner.intern("message");
                                    if let ValueKind::Object(u) = t.unpack() {
                                        let z = u
                                            .get_own_property(
                                                PropertyKey::from_js_string(JsString::from_sym(tos), &mut scope),
                                                &mut scope,
                                            )
                                            .unwrap();
                                        if let ValueKind::String(zr) = z.root(&mut scope).unpack() {
                                            format!("{}", zr.res(&mut scope))
                                        } else {
                                            format!("?")
                                        }
                                    } else {
                                        format!("?")
                                    }

                                    // displaying certain JS error "structures" like above causes a weird stack overflow.
                                    // requires further investigation. for now just display some hardcoded string
                                    // "<js error>".into()
                                }
                            };
                            if level & 0b10 != 0 {
                                println!(
                                    "  {}ERR{} {label}: {s}",
                                    ansi(c, 33),
                                    ansi(c, 0),
                                );
                            }
                        }
                        RunResult::Panic => {}
                    }
                }

                result
            }
        }
    });

    match maybe_pass {
        Ok(res) => res,
        Err(_) => RunResult::Panic,
    }
}
