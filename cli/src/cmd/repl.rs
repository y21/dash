use dash_middle::parser::error::IntoFormattableErrors;
use dash_optimizer::OptLevel;
use dash_rt::format_value;
use dash_rt::runtime::Runtime;
use dash_vm::eval::EvalError;
use dash_vm::value::Root;
use rustyline::DefaultEditor;

pub fn repl() -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;

    tokio::runtime::Runtime::new()?.block_on(async move {
        let mut rt = Runtime::new(None);

        while let Ok(input) = rl.readline("> ") {
            if input.is_empty() {
                continue;
            }

            rl.add_history_entry(&input).unwrap();

            rt.vm_mut().with_scope(|scope| {
                match scope.eval(&input, OptLevel::Aggressive) {
                    Ok(value) => println!("{}", format_value(value.root(scope), scope).unwrap()),
                    Err(EvalError::Exception(value)) => {
                        println!("{}", format_value(value.root(scope), scope).unwrap())
                    }
                    Err(EvalError::Middle(errs)) => println!("{}", errs.formattable(&input, true)),
                }

                scope.process_async_tasks();
            });
        }
    });

    Ok(())
}
