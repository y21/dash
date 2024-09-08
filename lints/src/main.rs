#![feature(rustc_private, let_chains, box_patterns, if_let_guard)]
#![deny(rust_2018_idioms)]

use missing_root::{MissingRoot, MISSING_ROOT};
use rustc_driver::{Callbacks, RunCompiler};
use rustc_session::config::{ErrorOutputType, OptLevel};
use rustc_session::EarlyDiagCtxt;

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
extern crate rustc_trait_selection;

mod missing_root;
mod utils;

struct RustcCallbacks;
impl Callbacks for RustcCallbacks {}

struct PrimaryCallbacks;
impl Callbacks for PrimaryCallbacks {
    fn config(&mut self, config: &mut rustc_interface::interface::Config) {
        config.register_lints = Some(Box::new(|_, lints| {
            lints.register_lints(&[MISSING_ROOT]);
            lints.register_late_pass(|_| Box::new(MissingRoot::default()));
        }));
        config.opts.unstable_opts.mir_opt_level = Some(0);
        config.opts.optimize = OptLevel::No;
    }
}

fn main() {
    let early_dcx = EarlyDiagCtxt::new(ErrorOutputType::default());
    rustc_driver::init_rustc_env_logger(&early_dcx);

    let mut args = rustc_driver::args::raw_args(&early_dcx).unwrap();

    if args.iter().any(|arg| arg == "--cap-lints") || !args.iter().any(|arg| arg.contains("dash_vm")) {
        // dependencies
        RunCompiler::new(&args[1..], &mut RustcCallbacks).run().unwrap();
    } else {
        args.extend(["--cfg", "dash_lints"].map(String::from));
        RunCompiler::new(&args[1..], &mut PrimaryCallbacks).run().unwrap();
    }
}
