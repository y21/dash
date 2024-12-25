use anyhow::Context;
use clap::ArgMatches;
use dash_middle::parser::error::IntoFormattableErrors;
use dash_optimizer::OptLevel;
use dash_rt::format_value;
use dash_rt::runtime::Runtime;
use dash_vm::eval::EvalError;
use dash_vm::value::Root;

pub fn eval(args: &ArgMatches) -> anyhow::Result<()> {
    let source = args.value_of("source").context("Missing source")?;
    let opt = *args.get_one::<OptLevel>("opt").unwrap();

    tokio::runtime::Runtime::new()?.block_on(async move {
        let mut runtime = Runtime::new(None);

        runtime.vm_mut().with_scope(|scope| {
            match scope.eval(source, opt) {
                Ok(value) => println!("{}", format_value(value.root(scope), scope).unwrap()),
                Err(EvalError::Exception(value)) => {
                    println!("{}", format_value(value.root(scope), scope).unwrap())
                }
                Err(EvalError::Middle(errs)) => println!("{}", errs.formattable(source, true)),
            };
        });

        runtime.run_event_loop().await;
    });

    Ok(())
}
