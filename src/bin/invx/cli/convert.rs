use std::{error::Error, fs, path::PathBuf, process::ExitCode};

use clap::ValueEnum;

use invx::conversion::{
    self, ConversionDiagnostics, ConversionError, ConversionImpact, TargetFormat,
};

use super::input::load_invoice;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TargetFormatArg {
    #[value(name = "ubl")]
    Ubl,

    #[value(name = "ebinterface-6p1")]
    EbInterface6p1,
}

impl From<TargetFormatArg> for TargetFormat {
    fn from(value: TargetFormatArg) -> Self {
        match value {
            TargetFormatArg::Ubl => Self::Ubl,

            TargetFormatArg::EbInterface6p1 => Self::EbInterface6p1,
        }
    }
}

#[derive(Debug, clap::Args)]
pub struct ConvertArgs {
    /// Target invoice format
    #[arg(long, value_enum)]
    pub to: TargetFormatArg,

    /// Input invoice
    pub file: PathBuf,

    /// Write converted invoice to a file instead of stdout
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub fn run(args: &ConvertArgs) -> Result<ExitCode, Box<dyn Error>> {
    let invoice = load_invoice(&args.file)?;

    let target = TargetFormat::from(args.to);

    match conversion::convert(&invoice, target) {
        Ok(output) => {
            print_diagnostics(&output.diagnostics);

            match &args.output {
                Some(path) => {
                    fs::write(path, output.xml)?;
                }

                None => {
                    print!("{}", output.xml);
                }
            }

            Ok(ExitCode::SUCCESS)
        }

        Err(ConversionError::Incompatible {
            target,
            diagnostics,
        }) => {
            eprintln!("CANNOT CONVERT to {target}");

            eprintln!();

            print_diagnostics(&diagnostics);

            Ok(ExitCode::from(1))
        }

        Err(error) => Err(Box::new(error)),
    }
}

fn print_diagnostics(diagnostics: &ConversionDiagnostics) {
    for issue in &diagnostics.issues {
        let impact = match issue.impact {
            ConversionImpact::Blocking => "BLOCKING",

            ConversionImpact::Lossy => "LOSSY",

            ConversionImpact::Informational => "INFO",
        };

        eprintln!("{impact} {} [{}]", issue.code, issue.path,);

        eprintln!("  {}", issue.message);

        eprintln!();
    }
}
