use dash_optimizer::OptLevel;
use dash_vm::eval::EvalError;
use dash_vm::Vm;
use rustyline::Editor;

use crate::util;

pub fn repl() -> anyhow::Result<()> {
    let mut rl = Editor::<()>::new();

    let mut vm = Vm::new(Default::default());

    while let Ok(input) = rl.readline("> ") {
        if input.is_empty() {
            continue;
        }

        rl.add_history_entry(&input);

        match vm.eval(&input, OptLevel::Aggressive) {
            Ok(value) | Err(EvalError::Exception(value)) => util::print_value(value, &mut vm).unwrap(),
            Err(e) => println!("{e}"),
        }

        vm.process_async_tasks();
    }

    Ok(())
}
