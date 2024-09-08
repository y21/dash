use std::error::Error;
use std::process::Command;
use std::{env, fs, io};

fn main() -> Result<(), Box<dyn Error>> {
    let command = Command::new("cargo")
        .current_dir("tests/ui")
        .arg("-q")
        .arg("c")
        .env("RUSTC_WRAPPER", "../../../target/debug/lints")
        .output()?;

    assert_eq!(command.stdout, b"");
    let now = std::str::from_utf8(&command.stderr).expect("cargo c never emits invalid utf8");
    match fs::read_to_string("tests/ui/ui.stderr") {
        Ok(before) => {
            if before != now {
                if env::args().any(|arg| arg == "--bless") {
                    fs::write("tests/ui/ui.stderr", now)?;
                } else {
                    let diff = prettydiff::diff_lines(&before, now);
                    println!("{diff}");
                    return Err("stderr has changed! rerun with --bless to update".into());
                }
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            fs::write("tests/ui/ui.stderr", now)?;
        }
        Err(err) => return Err(err.into()),
    }

    Ok(())
}
