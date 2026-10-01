mod cli;

use std::{error::Error, fs, path::Path, process::ExitCode};

use clap::Parser;

use cli::{Cli, Command};

use invx::{
    domain::Invoice,
    ubl::parser::parse_invoice,
    validation::{Severity, validate_invoice},
};

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
        Command::Validate { file } => run_validate(&file),
    }
}

fn load_ubl_invoice(path: &Path) -> Result<Invoice, Box<dyn Error>> {
    let xml = fs::read_to_string(path)?;

    let ubl = parse_invoice(&xml)?;

    let invoice = Invoice::try_from(ubl)?;

    Ok(invoice)
}

fn run_validate(path: &Path) -> Result<ExitCode, Box<dyn Error>> {
    let invoice = load_ubl_invoice(path)?;

    let result = validate_invoice(&invoice);

    if result.is_valid() {
        println!("VALID: {}", path.display());

        return Ok(ExitCode::SUCCESS);
    }

    println!("INVALID: {}", path.display());

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
