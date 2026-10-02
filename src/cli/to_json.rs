use std::{error::Error, path::PathBuf, process::ExitCode};

use super::input::load_ubl_invoice;

#[derive(Debug, clap::Args)]
pub struct ToJsonArgs {
    /// Invoice XML file
    pub file: PathBuf,
}

pub fn run(args: &ToJsonArgs) -> Result<ExitCode, Box<dyn Error>> {
    let invoice = load_ubl_invoice(&args.file)?;

    let json = serde_json::to_string_pretty(&invoice)?;

    println!("{json}");

    Ok(ExitCode::SUCCESS)
}
