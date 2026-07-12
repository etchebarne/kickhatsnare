mod api;
mod transport;

use std::{io, process::ExitCode};

use kickhatsnare_core::Core;

fn main() -> ExitCode {
    let stdin = io::stdin();
    let stdout = io::stdout();

    match transport::serve(stdin.lock(), stdout.lock(), &mut Core::new()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("server transport failed: {error}");
            ExitCode::FAILURE
        }
    }
}
