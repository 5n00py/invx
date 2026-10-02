use std::{error::Error, path::PathBuf, process::ExitCode};

use invx::validation::{Severity, ValidationProfile, validate};

use super::input::load_ubl_invoice;

#[derive(Debug, clap::Args)]
pub struct ValidateArgs {
    /// Validation profile
    #[arg(long, value_enum, default_value = "core")]
    pub profile: ProfileArg,

    /// Invoice XML file
    pub file: PathBuf,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ProfileArg {
    Core,
    En16931Subset,
}

impl From<ProfileArg> for ValidationProfile {
    fn from(profile: ProfileArg) -> Self {
        match profile {
            ProfileArg::Core => ValidationProfile::Core,

            ProfileArg::En16931Subset => ValidationProfile::En16931Subset,
        }
    }
}

impl ProfileArg {
    fn label(self) -> &'static str {
        match self {
            ProfileArg::Core => "core",
            ProfileArg::En16931Subset => "en16931-subset",
        }
    }
}

pub fn run(args: &ValidateArgs) -> Result<ExitCode, Box<dyn Error>> {
    let invoice = load_ubl_invoice(&args.file)?;

    let result = validate(&invoice, args.profile.into());

    if result.is_valid() {
        println!("VALID [{}]: {}", args.profile.label(), args.file.display());
        return Ok(ExitCode::SUCCESS);
    }

    println!(
        "INVALID [{}]: {}",
        args.profile.label(),
        args.file.display()
    );

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
