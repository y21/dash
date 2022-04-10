use rustyline::Editor;
use dash_core as dash;


pub fn repl() -> anyhow::Result<()> {
    let mut rl = Editor::<()>::new();

    while let Ok(input) = rl.readline("> ") {
        if input.is_empty() {
            continue;
        }

        rl.add_history_entry(&input);

        match dash::eval(&input, Default::default()) {
            Ok((_vm, value)) => {
                println!("{:?}", value);
            }
            Err(err) => println!("{}", err),
        }
    }

    Ok(())
}
