//! Static host process boundary (REQ-H-01).
use std::{
    io::{self, Write},
    process::ExitCode,
};
/// Keeps diagnostics off the image stream and classifies errors by origin (REQ-H-01).
fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match sisyphium_tabula::run(&args, &mut io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "sisyphium-tabula: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
