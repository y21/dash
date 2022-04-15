use dash::vm::local::LocalScope;
use dash::vm::value::ops::abstractions::conversions::ValueConversion;
use dash_core as dash;
use rustyline::Editor;

pub fn repl() -> anyhow::Result<()> {
    let mut rl = Editor::<()>::new();

    while let Ok(input) = rl.readline("> ") {
        if input.is_empty() {
            continue;
        }

        rl.add_history_entry(&input);

        match dash::eval(&input, Default::default()) {
            Ok((mut vm, value)) => {
                let mut scope = LocalScope::new(&mut vm);
                println!("{}", value.to_string(&mut scope).unwrap());
            }
            Err(err) => println!("Error: {}", err),
        }
    }

    Ok(())
}
