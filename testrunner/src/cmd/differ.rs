use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, anyhow};
use bumpalo::Bump;

use crate::cmd::results::ResultsMap;
use crate::cmd::run::{Results, RunResult};

fn find_last_result_file() -> anyhow::Result<Option<PathBuf>> {
    match std::fs::create_dir("../target/test262-diff") {
        Ok(_) => {}
        Err(err) if let ErrorKind::AlreadyExists = err.kind() => {}
        Err(err) => return Err(anyhow!("failed to create test262-diff directory: {err}")),
    }

    let dir = match fs::read_dir("../target/test262-diff") {
        Ok(it) => it,
        Err(err) if let ErrorKind::NotFound = err.kind() => return Ok(None),
        Err(err) => return Err(err.into()),
    };

    let mut last_file: Option<(SystemTime, PathBuf)> = None;
    for entry in dir {
        let entry = entry.context("reading dir entry")?;
        let metadata = entry.metadata().context("reading metadata")?;
        let mtime = metadata.modified().context("reading modified time")?;

        if last_file.as_ref().is_none_or(|&(time, _)| mtime > time) {
            last_file = Some((mtime, entry.path()));
        }
    }

    Ok(last_file.map(|(_, path)| path))
}

pub fn strip_test262_prefix(path: &str) -> anyhow::Result<&str> {
    let (_, path) = path
        .split_once("test262/test")
        .context("path does not contain test262 in it")?;
    Ok(path)
}

fn write_results(results: &Results) -> anyhow::Result<()> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("getting current time")?;
    let filename = format!("../target/test262-diff/results-{}.txt", now.as_secs());
    let mut file = BufWriter::new(File::create(filename).context("creating results file")?);

    for (path, result) in results.results_map().iter() {
        file.write_all(path.as_bytes())
            .context("failed to write path component")?;

        file.write_all(b";").context("failed to write separator")?;

        let result_value = result as u8;

        file.write_all(&[result_value]).context("failed to write result")?;

        file.write_all(b"\n").context("failed to write newline")?;
    }

    Ok(())
}

fn parse_results_from_file<'bump>(bump: &'bump Bump, path: &Path) -> anyhow::Result<ResultsMap<'bump>> {
    let mut results = ResultsMap::new(ResultsMap::DEFAULT_CAPACITY);

    let file = BufReader::new(File::open(path).context("opening results file")?);
    for line in file.lines() {
        let line = line?;

        let (path, result) = line.split_once(';').context("splitting line into path and result")?;
        let &[byte] = result.as_bytes() else {
            return Err(anyhow!("result is not a single byte: {result}"));
        };
        let result = RunResult::from_u8(byte).context("resolving run result")?;
        results.insert(bump.alloc_str(path), result);
    }

    Ok(results)
}

pub fn diff_results_to_previous(bump: &Bump, results: &Results) -> anyhow::Result<()> {
    let last_file = find_last_result_file().context("finding last result file")?;

    write_results(results).context("writing current results")?;
    if let Some(last_file) = last_file {
        // TODO: might not need to collect into a ResultsMap, maybe we can just iterate
        let last_results = parse_results_from_file(bump, &last_file)?;
        let new_results = results.results_map();
        let mut missing_in_new = Vec::new();

        println!();

        for (path, old_result) in last_results.iter() {
            let new_result = new_results.get(path);
            match new_result {
                Some(new_result) => {
                    if new_result != old_result {
                        println!(
                            "[{before}->{now}] {path}",
                            before = old_result.styled(),
                            now = new_result.styled()
                        );
                    }
                }
                None => {
                    // Not present in the new result?
                    missing_in_new.push(path);
                }
            }
        }

        if !missing_in_new.is_empty() {
            println!();
            println!("Missing in new results:");
            for path in missing_in_new {
                println!("{path}");
            }
        }

        if new_results.len() != last_results.len() {
            println!();
            println!(
                "Results count changed!: {} -> {}",
                last_results.len(),
                new_results.len()
            );
        }
    }
    Ok(())
}
