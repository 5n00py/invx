use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;
use thiserror::Error;

use crate::domain::{
    Address, AdjustmentKind, Currency, DocumentAdjustment, Invoice, InvoiceId, InvoiceLine,
    InvoiceTotals, Money, Party, PaymentAccount, PaymentInformation, TaxInformation, VatBreakdown,
};

use super::model::{UblAllowanceCharge, UblAmount, UblInvoice, UblParty, UblPaymentMeans};

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

    #[error("multiple payment means are not yet supported")]
    MultiplePaymentMeans,
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

fn zero_money(currency: &Currency) -> Money {
    Money::new(Decimal::ZERO, currency.clone())
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

fn map_party(source: UblParty) -> Party {
    let address = source.postal_address.map(|address| Address {
        street: address.street_name,
        postal_code: address.postal_zone,
        city: address.city_name,
        country_code: address
            .country
            .and_then(|country| country.identification_code),
    });

    let vat_id = source
        .party_tax_schemes
        .into_iter()
        .find(|tax_scheme| {
            tax_scheme
                .tax_scheme
                .as_ref()
                .and_then(|scheme| scheme.id.as_deref())
                == Some("VAT")
        })
        .and_then(|tax_scheme| tax_scheme.company_id);

    Party {
        name: source.party_name.name,
        address,
        vat_id,
    }
}

fn map_payment(source: UblPaymentMeans) -> PaymentInformation {
    let payee_account = source
        .payee_financial_account
        .and_then(|account| account.id.map(|identifier| PaymentAccount { identifier }));

    PaymentInformation {
        means_code: source.payment_means_code,
        reference: source.payment_id,
        payee_account,
    }
}

fn map_adjustment(
    source: UblAllowanceCharge,
    currency: &Currency,
) -> Result<DocumentAdjustment, MappingError> {
    let kind = if source.charge_indicator {
        AdjustmentKind::Charge
    } else {
        AdjustmentKind::Allowance
    };

    let amount = map_amount(source.amount, currency, "AllowanceCharge.Amount")?;

    let base_amount = source
        .base_amount
        .map(|amount| map_amount(amount, currency, "AllowanceCharge.BaseAmount"))
        .transpose()?;

    let percentage = source
        .multiplier_factor_numeric
        .map(|value| parse_decimal(&value, "AllowanceCharge.MultiplierFactorNumeric"))
        .transpose()?;

    let tax = source
        .tax_category
        .map(|tax| {
            let rate = parse_decimal(&tax.percent, "AllowanceCharge.TaxCategory.Percent")?;

            Ok::<_, MappingError>(TaxInformation {
                category_code: tax.id,
                rate,
            })
        })
        .transpose()?;

    Ok(DocumentAdjustment {
        kind,
        amount,
        base_amount,
        percentage,
        reason_code: source.reason_code,
        reasons: source.reasons,
        tax,
    })
}

impl TryFrom<UblInvoice> for Invoice {
    type Error = MappingError;

    fn try_from(source: UblInvoice) -> Result<Self, Self::Error> {
        let currency = Currency::new(source.document_currency_code);

        let issue_date = parse_date(&source.issue_date, "IssueDate")?;

        let seller = map_party(source.accounting_supplier_party.party);

        let buyer = map_party(source.accounting_customer_party.party);

        let order_reference = source.order_reference.map(|reference| reference.id);

        let payment = match source.payment_means.len() {
            0 => None,

            1 => source.payment_means.into_iter().next().map(map_payment),

            _ => {
                return Err(MappingError::MultiplePaymentMeans);
            }
        };

        let mut lines = Vec::new();

        for line in source.invoice_lines {
            let tax_rate = parse_decimal(
                &line.item.classified_tax_category.percent,
                "InvoiceLine.Item.ClassifiedTaxCategory.Percent",
            )?;

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
                    category_code: line.item.classified_tax_category.id,
                    rate: tax_rate,
                },
            });
        }

        /*
         * Pull these nested structures out before consuming
         * their individual fields below.
         */
        let tax_total = source.tax_total;
        let monetary_total = source.legal_monetary_total;

        let mut vat_breakdown = Vec::new();

        for subtotal in tax_total.tax_subtotals {
            let rate = parse_decimal(
                &subtotal.tax_category.percent,
                "TaxSubtotal.TaxCategory.Percent",
            )?;

            vat_breakdown.push(VatBreakdown {
                category_code: subtotal.tax_category.id,

                rate,

                taxable_amount: map_amount(
                    subtotal.taxable_amount,
                    &currency,
                    "TaxSubtotal.TaxableAmount",
                )?,

                tax_amount: map_amount(subtotal.tax_amount, &currency, "TaxSubtotal.TaxAmount")?,
            });
        }

        let adjustments = source
            .allowance_charges
            .into_iter()
            .map(|adjustment| map_adjustment(adjustment, &currency))
            .collect::<Result<Vec<_>, _>>()?;

        let line_net_amount = map_amount(
            monetary_total.line_extension_amount,
            &currency,
            "LegalMonetaryTotal.LineExtensionAmount",
        )?;

        let allowance_amount = monetary_total
            .allowance_total_amount
            .map(|amount| map_amount(amount, &currency, "LegalMonetaryTotal.AllowanceTotalAmount"))
            .transpose()?
            .unwrap_or_else(|| zero_money(&currency));

        let charge_amount = monetary_total
            .charge_total_amount
            .map(|amount| map_amount(amount, &currency, "LegalMonetaryTotal.ChargeTotalAmount"))
            .transpose()?
            .unwrap_or_else(|| zero_money(&currency));

        let net_amount = map_amount(
            monetary_total.tax_exclusive_amount,
            &currency,
            "LegalMonetaryTotal.TaxExclusiveAmount",
        )?;

        let tax_amount = map_amount(tax_total.tax_amount, &currency, "TaxTotal.TaxAmount")?;

        let gross_amount = map_amount(
            monetary_total.tax_inclusive_amount,
            &currency,
            "LegalMonetaryTotal.TaxInclusiveAmount",
        )?;

        let payable_amount = map_amount(
            monetary_total.payable_amount,
            &currency,
            "LegalMonetaryTotal.PayableAmount",
        )?;

        let totals = InvoiceTotals {
            line_net_amount,
            allowance_amount,
            charge_amount,

            net_amount,
            tax_amount,
            gross_amount,
            payable_amount,
        };

        Ok(Invoice {
            id: InvoiceId::new(source.id),
            issue_date,
            currency,

            seller,
            buyer,

            order_reference,

            lines,
            vat_breakdown,

            payment,

            totals,

            adjustments,
        })
    }
}
