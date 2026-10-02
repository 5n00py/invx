use std::{error::Error, fs, path::Path};

use invx::{domain::Invoice, ubl::parser::parse_invoice};

pub fn load_ubl_invoice(path: &Path) -> Result<Invoice, Box<dyn Error>> {
    let xml = fs::read_to_string(path)?;

    let ubl = parse_invoice(&xml)?;

    let invoice = Invoice::try_from(ubl)?;

    Ok(invoice)
}
