use dash_optimizer::OptLevel;
use dash_vm::eval::EvalError;
use dash_vm::Vm;
use rustyline::Editor;

use crate::util;

pub fn repl() -> anyhow::Result<()> {
    let mut rl = Editor::<()>::new();

    let mut vm = Vm::new(Default::default());
    let mut scope = vm.scope();

    while let Ok(input) = rl.readline("> ") {
        if input.is_empty() {
            continue;
        }

        rl.add_history_entry(&input);

        match scope.eval(&input, OptLevel::Aggressive) {
            Ok(value) => util::print_value(value.root(&mut scope), &mut scope).unwrap(),
            Err(EvalError::Exception(value)) => util::print_value(value, &mut scope).unwrap(),
            Err(e) => println!("{e}"),
        }

        scope.process_async_tasks();
    }

    Ok(())
}
