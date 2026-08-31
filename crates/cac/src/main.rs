mod cli;
mod commands;
mod error;
mod source;

use std::process::ExitCode;

use clap::Parser;

use cli::Cli;

fn main() -> ExitCode {
    match Cli::parse().run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
