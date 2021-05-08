use std::{fs, path::PathBuf};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "dash")]
struct Args {
    #[structopt(name = "file", parse(from_os_str))]
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Args::from_args();
    let file = opt
        .file
        .to_str()
        .expect("Failed to parse file input string");

    let code = fs::read_to_string(file)?;

    if let Err(e) = dash::eval(code) {
        println!("{:?}", e);
    }

    Ok(())
}
