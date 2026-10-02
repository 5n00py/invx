use std::{error::Error, path::PathBuf, process::ExitCode};

use invx::domain::{Address, AdjustmentKind, Invoice, PaymentMethod};

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

    if let Some(order_reference) = &invoice.order_reference {
        println!("  Order ref:  {order_reference}");
    }

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

        if let Some(unit_code) = &line.unit_code {
            println!("     Unit code:  {unit_code}");
        }

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

        match &line.tax.category_code {
            Some(category) => {
                println!("     VAT:        {} / {}%", category, line.tax.rate);
            }

            None => {
                println!("     VAT rate:   {}%", line.tax.rate);
            }
        }

        println!();
    }

    println!("VAT");

    for vat in &invoice.vat_breakdown {
        match &vat.category_code {
            Some(category) => {
                println!(
                    "  {} / {}%: {} {}",
                    category,
                    vat.rate,
                    vat.tax_amount.amount,
                    vat.tax_amount.currency.as_str()
                );
            }

            None => {
                println!(
                    "  {}%: {} {}",
                    vat.rate,
                    vat.tax_amount.amount,
                    vat.tax_amount.currency.as_str()
                );
            }
        }

        println!(
            "     Taxable: {} {}",
            vat.taxable_amount.amount,
            vat.taxable_amount.currency.as_str()
        );
    }

    if !invoice.adjustments.is_empty() {
        println!();

        println!("Adjustments");

        for adjustment in &invoice.adjustments {
            let kind = match adjustment.kind {
                AdjustmentKind::Allowance => "Allowance",
                AdjustmentKind::Charge => "Charge",
            };

            println!("  {kind}");

            for reason in &adjustment.reasons {
                println!("     Reason:     {reason}");
            }

            if let Some(reason_code) = &adjustment.reason_code {
                println!("     Reason code: {reason_code}");
            }

            if let Some(percentage) = adjustment.percentage {
                println!("     Percentage: {percentage}%");
            }

            if let Some(base_amount) = &adjustment.base_amount {
                println!(
                    "     Base amount: {} {}",
                    base_amount.amount,
                    base_amount.currency.as_str()
                );
            }

            println!(
                "     Amount:      {} {}",
                adjustment.amount.amount,
                adjustment.amount.currency.as_str()
            );

            if let Some(tax) = &adjustment.tax {
                match &tax.category_code {
                    Some(category) => {
                        println!("     VAT:         {} / {}%", category, tax.rate);
                    }

                    None => {
                        println!("     VAT rate:    {}%", tax.rate);
                    }
                }
            }

            println!();
        }
    }

    if let Some(payment) = &invoice.payment {
        println!("Payment");

        let method = match payment.method {
            PaymentMethod::BankTransfer => "Bank transfer",
            PaymentMethod::Other => "Other",
        };

        println!("  Method:     {method}");

        if let Some(means_code) = &payment.means_code {
            println!("  Means code: {means_code}");
        }

        if let Some(reference) = &payment.reference {
            println!("  Reference:  {reference}");
        }

        if let Some(account) = &payment.payee_account {
            println!("  Account:    {}", account.identifier);
        }

        println!();
    }

    println!("Totals");

    println!(
        "  Line net:   {} {}",
        invoice.totals.line_net_amount.amount,
        invoice.totals.line_net_amount.currency.as_str()
    );

    println!(
        "  Allowances: {} {}",
        invoice.totals.allowance_amount.amount,
        invoice.totals.allowance_amount.currency.as_str()
    );

    println!(
        "  Charges:    {} {}",
        invoice.totals.charge_amount.amount,
        invoice.totals.charge_amount.currency.as_str()
    );

    println!(
        "  Net:        {} {}",
        invoice.totals.net_amount.amount,
        invoice.totals.net_amount.currency.as_str()
    );

    println!(
        "  VAT:        {} {}",
        invoice.totals.tax_amount.amount,
        invoice.totals.tax_amount.currency.as_str()
    );

    println!(
        "  Gross:      {} {}",
        invoice.totals.gross_amount.amount,
        invoice.totals.gross_amount.currency.as_str()
    );

    println!(
        "  Payable:    {} {}",
        invoice.totals.payable_amount.amount,
        invoice.totals.payable_amount.currency.as_str()
    );
}

fn print_address(address: &Address) {
    if let Some(street) = &address.street {
        println!("  {street}");
    }

    match (&address.postal_code, &address.city) {
        (Some(postal_code), Some(city)) => {
            println!("  {postal_code} {city}");
        }

        (Some(postal_code), None) => {
            println!("  {postal_code}");
        }

        (None, Some(city)) => {
            println!("  {city}");
        }

        (None, None) => {}
    }

    if let Some(country_code) = &address.country_code {
        println!("  {country_code}");
    }
}
