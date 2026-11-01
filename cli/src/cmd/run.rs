use dash_middle::parser::error::IntoFormattableErrors;
use dash_rt::format_value;
use dash_rt::runtime::Runtime;
use dash_vm::eval::EvalError;
use dash_vm::params::VmParams;
use dash_vm::value::Root;
use std::fs;
use std::time::Instant;

use anyhow::Context;
use clap::ArgMatches;

use crate::util::params_from_args;

pub fn run(args: &ArgMatches) -> anyhow::Result<()> {
    let path = args.get_one::<String>("file").context("Missing source")?;
    let nodejs = *args.get_one::<bool>("node").unwrap();
    let before = args.get_one::<bool>("timing").unwrap().then(Instant::now);
    let quiet = *args.get_one::<bool>("quiet").unwrap();
    let params = params_from_args(args)?;

    if nodejs {
        #[cfg(feature = "nodejs")]
        {
            let script_args = args
                .get_many::<String>("script_args")
                .unwrap_or_default()
                .map(String::as_str);

            dash_node_impl::run_with_nodejs_mnemnoics(dash_node_impl::NodeRunArgs {
                path,
                params,
                script_args,
            })?;
        }
        #[cfg(not(feature = "nodejs"))]
        {
            anyhow::bail!("dash needs to be compiled with the `nodejs` feature to support node-compat mode");
        }
    } else {
        run_normal_mode(path, quiet, params)?;
    }

    if let Some(before) = before {
        println!("\n{:?}", before.elapsed());
    }

    Ok(())
}

fn run_normal_mode(path: &str, quiet: bool, params: VmParams) -> anyhow::Result<()> {
    let source = fs::read_to_string(path).context("Failed to read source")?;

    let async_rt = tokio::runtime::Runtime::new()?;
    async_rt.block_on(inner(source, quiet, params))?;

    Ok(())
}

async fn inner(source: String, quiet: bool, params: VmParams) -> anyhow::Result<()> {
    let mut rt = Runtime::new(params);

    let module = dash_rt_modules::init_modules();
    rt.set_module_manager(module);

    let mut scope = rt.vm_mut().scope();
    let value = match scope.eval(&source) {
        Ok(val) => val.root(&mut scope),
        Err(EvalError::Exception(val)) => val.root(&mut scope),
        Err(EvalError::Middle(errs)) => {
            println!("{}", errs.formattable(&source, true));
            return Ok(());
        }
    };

    scope.process_async_tasks();

    // TODO: EvalError::VmError should probably bail too?

    if !quiet {
        println!("{}", format_value(value, &mut scope).unwrap());
    }

    drop(scope);
    rt.run_event_loop().await;

    Ok(())
}
