use clap::{Parser, Subcommand};

pub mod input;
pub mod validate;

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
    Validate(validate::ValidateArgs),
}
