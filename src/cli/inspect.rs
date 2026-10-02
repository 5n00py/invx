use std::{error::Error, path::PathBuf, process::ExitCode};

use invx::domain::{Address, Invoice};

use super::input::load_ubl_invoice;

#[derive(Debug, clap::Args)]
pub struct InspectArgs {
    /// Invoice XML file
    pub file: PathBuf,
}

pub fn run(args: &InspectArgs) -> Result<ExitCode, Box<dyn Error>> {
    let invoice = load_ubl_invoice(&args.file)?;

    print_invoice(&invoice);

    Ok(ExitCode::SUCCESS)
}

fn print_invoice(invoice: &Invoice) {
    println!("Document");
    println!("  Invoice ID: {}", invoice.id.as_str());
    println!("  Issue date: {}", invoice.issue_date);
    println!("  Currency:   {}", invoice.currency.as_str());

    println!();

    println!("Seller");
    println!("  {}", invoice.seller.name);

    if let Some(address) = &invoice.seller.address {
        print_address(address);
    }

    if let Some(vat_id) = &invoice.seller.vat_id {
        println!("  VAT ID: {vat_id}");
    }

    println!();

    println!("Buyer");
    println!("  {}", invoice.buyer.name);

    if let Some(address) = &invoice.buyer.address {
        print_address(address);
    }

    if let Some(vat_id) = &invoice.buyer.vat_id {
        println!("  VAT ID: {vat_id}");
    }

    println!();

    println!("Lines");

    for line in &invoice.lines {
        println!("  {}  {}", line.id, line.description);
        println!("     Quantity:   {}", line.quantity);
        println!(
            "     Unit price: {} {}",
            line.unit_price.amount,
            line.unit_price.currency.as_str()
        );
        println!(
            "     Net:        {} {}",
            line.net_amount.amount,
            line.net_amount.currency.as_str()
        );
        println!("     VAT rate:   {}%", line.tax.rate);
    }

    println!();

    println!("VAT");

    for vat in &invoice.vat_breakdown {
        println!(
            "  {}%: {} {}",
            vat.rate,
            vat.tax_amount.amount,
            vat.tax_amount.currency.as_str()
        );
    }

    println!();

    println!("Totals");
    println!(
        "  Net:      {} {}",
        invoice.totals.net_amount.amount,
        invoice.totals.net_amount.currency.as_str()
    );
    println!(
        "  VAT:      {} {}",
        invoice.totals.tax_amount.amount,
        invoice.totals.tax_amount.currency.as_str()
    );
    println!(
        "  Gross:    {} {}",
        invoice.totals.gross_amount.amount,
        invoice.totals.gross_amount.currency.as_str()
    );
    println!(
        "  Payable:  {} {}",
        invoice.totals.payable_amount.amount,
        invoice.totals.payable_amount.currency.as_str()
    );
}

fn print_address(address: &Address) {
    if let Some(street) = &address.street {
        println!("  {street}");
    }

    let city_line = match (&address.postal_code, &address.city) {
        (Some(postal_code), Some(city)) => Some(format!("{postal_code} {city}")),

        (Some(postal_code), None) => Some(postal_code.clone()),

        (None, Some(city)) => Some(city.clone()),

        (None, None) => None,
    };

    if let Some(city_line) = city_line {
        println!("  {city_line}");
    }

    if let Some(country_code) = &address.country_code {
        println!("  {country_code}");
    }
}
