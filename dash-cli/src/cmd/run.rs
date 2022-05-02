use dash::compiler::StaticImportKind;
use dash::vm::params::VmParams;
use dash::vm::value::Value;
use dash::vm::Vm;
use dash_core as dash;
use std::fs;
use std::time::Instant;

use anyhow::bail;
use anyhow::Context;
use clap::ArgMatches;
use dash::vm::local::LocalScope;
use dash::vm::value::ops::abstractions::conversions::ValueConversion;

use crate::util;

fn import_callback(_vm: &mut Vm, _ty: StaticImportKind, path: &str) -> Result<Value, Value> {
    Ok(Value::String(format!("Hi module {path}").into()))
}

pub fn run(args: &ArgMatches) -> anyhow::Result<()> {
    let path = args.value_of("file").context("Missing source")?;
    let source = fs::read_to_string(path).context("Failed to read source")?;
    let opt = util::opt_level_from_matches(args)?;

    let before = args.is_present("timing").then(|| Instant::now());

    let params = VmParams::new().set_static_import_callback(import_callback);

    match dash::eval(&source, opt, params) {
        Ok((mut vm, value)) => {
            let mut sc = LocalScope::new(&mut vm);
            println!("{}", value.to_string(&mut sc).unwrap());
        }
        Err(err) => bail!("{}", err),
    }

    if let Some(before) = before {
        println!("{:?}", before.elapsed());
    }

    Ok(())
}
