use std::{error::Error, path::PathBuf, process::ExitCode};

use invx::validation::{Severity, validate_invoice};

use super::input::load_ubl_invoice;

#[derive(Debug, clap::Args)]
pub struct ValidateArgs {
    /// Invoice XML file
    pub file: PathBuf,
}

pub fn run(args: &ValidateArgs) -> Result<ExitCode, Box<dyn Error>> {
    let invoice = load_ubl_invoice(&args.file)?;

    let result = validate_invoice(&invoice);

    if result.is_valid() {
        println!("VALID: {}", args.file.display());
        return Ok(ExitCode::SUCCESS);
    }

    println!("INVALID: {}", args.file.display());

    for violation in &result.violations {
        let severity = severity_label(&violation.severity);

        match &violation.field {
            Some(field) => {
                println!(
                    "{severity} {} [{field}] {}",
                    violation.code, violation.message
                );
            }

            None => {
                println!("{severity} {} {}", violation.code, violation.message);
            }
        }
    }

    Ok(ExitCode::from(1))
}

fn severity_label(severity: &Severity) -> &'static str {
    match severity {
        Severity::Error => "ERROR",
        Severity::Warning => "WARNING",
        Severity::Info => "INFO",
    }
}
