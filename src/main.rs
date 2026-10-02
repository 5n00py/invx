mod cli;

use std::{error::Error, process::ExitCode};

use clap::Parser;

use cli::{Cli, Command};

fn main() -> ExitCode {
    match run() {
        Ok(exit_code) => exit_code,

        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate(args) => cli::validate::run(&args),
    }
}
