use anyhow::bail;
use dash::vm::Vm;
use dash::EvalError;
use dash_core as dash;
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

        match vm.eval(&input, Default::default()) {
            Ok(value) | Err(EvalError::VmError(value)) => util::print_value(value, &mut vm).unwrap(),
            Err(e) => bail!("{e}"),
        }
    }

    Ok(())
}
