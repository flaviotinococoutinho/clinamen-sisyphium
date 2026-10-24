//! Native terminal executable entry point (REQ-H-01).
use std::{io, process::ExitCode};

/// Maps classified errors to the documented native exit statuses (REQ-H-01).
fn main() -> ExitCode {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    match sisyphium_textus::run(&args, &mut io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sisyphium-textus: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
