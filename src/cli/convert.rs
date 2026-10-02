use std::{error::Error, fs, path::PathBuf, process::ExitCode};

use clap::ValueEnum;

use invx::{ebinterface::writer as ebinterface_writer, ubl::writer as ubl_writer};

use super::input::load_invoice;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TargetFormat {
    #[value(name = "ubl")]
    Ubl,

    #[value(name = "ebinterface-6p1")]
    EbInterface6p1,
}

#[derive(Debug, clap::Args)]
pub struct ConvertArgs {
    /// Target invoice format
    #[arg(long, value_enum)]
    pub to: TargetFormat,

    /// Input invoice
    pub file: PathBuf,

    /// Write converted invoice to a file instead of stdout
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub fn run(args: &ConvertArgs) -> Result<ExitCode, Box<dyn Error>> {
    let invoice = load_invoice(&args.file)?;

    let xml = match args.to {
        TargetFormat::Ubl => ubl_writer::write_invoice(&invoice)?,

        TargetFormat::EbInterface6p1 => ebinterface_writer::write_invoice(&invoice)?,
    };

    match &args.output {
        Some(path) => {
            fs::write(path, xml)?;
        }

        None => {
            print!("{xml}");
        }
    }

    Ok(ExitCode::SUCCESS)
}
