mod api;
mod transport;

use std::{env, io, path::PathBuf, process::ExitCode};

use kickhatsnare_core::Core;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("server failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let data_directory = data_directory()?;
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut core = Core::open(data_directory)?;
    transport::serve(stdin.lock(), stdout.lock(), &mut core)?;
    Ok(())
}

fn data_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut arguments = env::args_os().skip(1);
    if arguments.next().as_deref() != Some(std::ffi::OsStr::new("--data-dir")) {
        return Err("usage: kickhatsnare-server --data-dir <path>".into());
    }
    let path = arguments.next().ok_or("missing value for --data-dir")?;
    if arguments.next().is_some() {
        return Err("unexpected server argument".into());
    }
    Ok(PathBuf::from(path))
}
