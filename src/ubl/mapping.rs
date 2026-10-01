use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;
use thiserror::Error;

use crate::domain::{
    Currency, Invoice, InvoiceId, InvoiceLine, InvoiceTotals, Money, Party, TaxInformation,
    VatBreakdown,
};

use super::model::{UblAmount, UblInvoice};

#[derive(Debug, Error)]
pub enum MappingError {
    #[error("invalid date '{value}' in field {field}")]
    InvalidDate { field: &'static str, value: String },

    #[error("invalid decimal '{value}' in field {field}")]
    InvalidDecimal { field: &'static str, value: String },

    #[error("currency mismatch in field {field}: expected {expected}, found {actual}")]
    CurrencyMismatch {
        field: &'static str,
        expected: String,
        actual: String,
    },
}

fn parse_date(value: &str, field: &'static str) -> Result<NaiveDate, MappingError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| MappingError::InvalidDate {
        field,
        value: value.to_string(),
    })
}

fn parse_decimal(value: &str, field: &'static str) -> Result<Decimal, MappingError> {
    Decimal::from_str(value).map_err(|_| MappingError::InvalidDecimal {
        field,
        value: value.to_string(),
    })
}

fn map_amount(
    amount: UblAmount,
    expected_currency: &Currency,
    field: &'static str,
) -> Result<Money, MappingError> {
    if amount.currency_id != expected_currency.as_str() {
        return Err(MappingError::CurrencyMismatch {
            field,
            expected: expected_currency.as_str().to_string(),
            actual: amount.currency_id,
        });
    }

    let value = parse_decimal(&amount.value, field)?;

    Ok(Money::new(value, expected_currency.clone()))
}

impl TryFrom<UblInvoice> for Invoice {
    type Error = MappingError;

    fn try_from(source: UblInvoice) -> Result<Self, Self::Error> {
        let currency = Currency::new(source.document_currency_code);

        let issue_date = parse_date(&source.issue_date, "IssueDate")?;

        let seller = Party {
            name: source.accounting_supplier_party.party.party_name.name,
            address: None,
            vat_id: None,
        };

        let buyer = Party {
            name: source.accounting_customer_party.party.party_name.name,
            address: None,
            vat_id: None,
        };

        let mut lines = Vec::new();

        for line in source.invoice_lines {
            lines.push(InvoiceLine {
                id: line.id,

                description: line.item.description,

                quantity: parse_decimal(
                    &line.invoiced_quantity.value,
                    "InvoiceLine.InvoicedQuantity",
                )?,

                unit_code: line.invoiced_quantity.unit_code,

                unit_price: map_amount(
                    line.price.price_amount,
                    &currency,
                    "InvoiceLine.Price.PriceAmount",
                )?,

                net_amount: map_amount(
                    line.line_extension_amount,
                    &currency,
                    "InvoiceLine.LineExtensionAmount",
                )?,

                tax: TaxInformation {
                    rate: parse_decimal(
                        &line.item.classified_tax_category.percent,
                        "InvoiceLine.Item.ClassifiedTaxCategory.Percent",
                    )?,
                },
            });
        }

        let mut vat_breakdown = Vec::new();

        for subtotal in source.tax_total.tax_subtotals {
            vat_breakdown.push(VatBreakdown {
                rate: parse_decimal(
                    &subtotal.tax_category.percent,
                    "TaxSubtotal.TaxCategory.Percent",
                )?,

                taxable_amount: map_amount(
                    subtotal.taxable_amount,
                    &currency,
                    "TaxSubtotal.TaxableAmount",
                )?,

                tax_amount: map_amount(subtotal.tax_amount, &currency, "TaxSubtotal.TaxAmount")?,
            });
        }

        let totals = InvoiceTotals {
            net_amount: map_amount(
                source.legal_monetary_total.tax_exclusive_amount,
                &currency,
                "LegalMonetaryTotal.TaxExclusiveAmount",
            )?,

            tax_amount: map_amount(source.tax_total.tax_amount, &currency, "TaxTotal.TaxAmount")?,

            gross_amount: map_amount(
                source.legal_monetary_total.tax_inclusive_amount,
                &currency,
                "LegalMonetaryTotal.TaxInclusiveAmount",
            )?,

            payable_amount: map_amount(
                source.legal_monetary_total.payable_amount,
                &currency,
                "LegalMonetaryTotal.PayableAmount",
            )?,
        };

        Ok(Invoice {
            id: InvoiceId::new(source.id),
            issue_date,
            currency,

            seller,
            buyer,

            order_reference: None,

            lines,
            vat_breakdown,

            totals,
        })
    }
}
