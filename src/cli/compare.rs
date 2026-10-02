use std::{error::Error, path::PathBuf, process::ExitCode};

use invx::compare::compare;

use super::input::load_invoice;

#[derive(Debug, clap::Args)]
pub struct CompareArgs {
    /// First invoice
    pub left: PathBuf,

    /// Second invoice
    pub right: PathBuf,
}

pub fn run(args: &CompareArgs) -> Result<ExitCode, Box<dyn Error>> {
    let left = load_invoice(&args.left)?;

    let right = load_invoice(&args.right)?;

    let result = compare(&left, &right);

    if result.is_equal() {
        println!("EQUAL: {} == {}", args.left.display(), args.right.display(),);

        return Ok(ExitCode::SUCCESS);
    }

    println!(
        "DIFFERENT: {} <> {}",
        args.left.display(),
        args.right.display(),
    );

    println!();

    for difference in &result.differences {
        println!("{}", difference.path);

        println!(
            "  left:  {}",
            difference.left.as_deref().unwrap_or("<missing>")
        );

        println!(
            "  right: {}",
            difference.right.as_deref().unwrap_or("<missing>")
        );

        println!();
    }

    Ok(ExitCode::from(1))
}
