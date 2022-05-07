use dash::vm::local::LocalScope;
use dash::vm::value::ops::abstractions::conversions::ValueConversion;
use dash::vm::Vm;
use dash_core as dash;
use rustyline::Editor;

pub fn repl() -> anyhow::Result<()> {
    let mut rl = Editor::<()>::new();

    let mut vm = Vm::new(Default::default());

    while let Ok(input) = rl.readline("> ") {
        if input.is_empty() {
            continue;
        }

        rl.add_history_entry(&input);

        match vm.eval(&input, Default::default()) {
            Ok(value) => {
                let mut scope = LocalScope::new(&mut vm);
                println!("{}", value.to_string(&mut scope).unwrap());
            }
            Err(err) => println!("Error: {}", err),
        }
    }

    Ok(())
}
