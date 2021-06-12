use std::{env, ffi::OsString, fs::DirEntry, io, str::FromStr};

use console::style;

/// Returns a vector of path strings
fn get_all_files(dir: &OsString) -> io::Result<Vec<OsString>> {
    let mut ve = Vec::new();

    let read_dir = std::fs::read_dir(dir)?;

    for entry in read_dir {
        let entry: DirEntry = entry?;

        let path = OsString::from(format!(
            "{}/{}",
            dir.as_os_str().to_str().unwrap(),
            entry.file_name().as_os_str().to_str().unwrap()
        ));

        let ty = entry.file_type()?;
        if ty.is_file() {
            ve.push(path);
        } else if ty.is_dir() {
            let files = get_all_files(&path)?;
            ve.extend(files);
        }
    }

    Ok(ve)
}

/// Runs a test case
fn run_test(path: OsString) {
    let path_str = path.to_str().unwrap();
    let source = std::fs::read_to_string(path_str).unwrap();

    let pass = dash::eval::<()>(&source, None).is_ok();
    if pass {
        println!("{} {}", style(path_str).green(), style("passed").green());
    } else {
        println!("{} {}", style(path_str).red(), style("did not pass").red());
    }
}

fn main() {
    let tests_path = env::var("TEST_PATH").unwrap_or_else(|_| String::from("../test262/test"));
    println!("Path to tests: {}", style(&tests_path).green());

    let tests = get_all_files(&OsString::from_str(&tests_path).unwrap()).unwrap();

    for test in tests {
        run_test(test);
    }
}
