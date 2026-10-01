use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "invx",
    version,
    about = "Semantic e-invoice inspection, validation and conversion"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Validate an invoice
    Validate {
        /// Invoice XML file
        file: PathBuf,
    },
}
