use clap::{Parser, Subcommand};

pub mod input;
pub mod inspect;
pub mod to_json;
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
    /// Inspect an invoice
    Inspect(inspect::InspectArgs),

    /// Validate an invoice
    Validate(validate::ValidateArgs),

    /// Convert an invoice to canonical JSON
    ToJson(to_json::ToJsonArgs),
}
