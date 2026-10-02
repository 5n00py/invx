use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;
use thiserror::Error;

use crate::domain::{
    Address, Currency, Invoice, InvoiceId, InvoiceLine, InvoiceTotals, Money, Party,
    PaymentAccount, PaymentInformation, PaymentMethod, TaxInformation, VatBreakdown,
};

use super::model::{
    EbInterfaceBeneficiaryAccount, EbInterfaceInvoice, EbInterfaceLineItem, EbInterfaceParty,
    EbInterfaceTaxItem, EbInterfaceUniversalBankTransaction,
};

#[derive(Debug, Error)]
pub enum MappingError {
    #[error("invalid date '{value}' in field {field}")]
    InvalidDate { field: &'static str, value: String },

    #[error("invalid decimal '{value}' in field {field}")]
    InvalidDecimal { field: &'static str, value: String },

    #[error("missing required field {field}")]
    MissingField { field: &'static str },

    #[error("multiple beneficiary accounts are not yet supported")]
    MultipleBeneficiaryAccounts,

    #[error("unsupported payment method")]
    UnsupportedPaymentMethod,

    #[error(
        "UnitPrice BaseQuantity '{value}' is not yet supported; only BaseQuantity 1 is supported"
    )]
    UnsupportedBaseQuantity { value: String },
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

fn money(value: &str, currency: &Currency, field: &'static str) -> Result<Money, MappingError> {
    Ok(Money::new(parse_decimal(value, field)?, currency.clone()))
}

fn zero_money(currency: &Currency) -> Money {
    Money::new(Decimal::ZERO, currency.clone())
}

fn map_party(source: EbInterfaceParty, address_field: &'static str) -> Result<Party, MappingError> {
    let address = source.address.ok_or(MappingError::MissingField {
        field: address_field,
    })?;

    let name = address.name.clone();

    Ok(Party {
        name,

        address: Some(Address {
            street: address.street,
            postal_code: Some(address.zip),
            city: Some(address.town),
            country_code: address.country.country_code,
        }),

        vat_id: Some(source.vat_identification_number),
    })
}

fn map_line(source: EbInterfaceLineItem, currency: &Currency) -> Result<InvoiceLine, MappingError> {
    let id = source.position_number.ok_or(MappingError::MissingField {
        field: "Details.ItemList.ListLineItem.PositionNumber",
    })?;

    let description = if source.descriptions.is_empty() {
        return Err(MappingError::MissingField {
            field: "Details.ItemList.ListLineItem.Description",
        });
    } else {
        source.descriptions.join("\n")
    };

    let quantity = parse_decimal(
        &source.quantity.value,
        "Details.ItemList.ListLineItem.Quantity",
    )?;

    /*
     * ebInterface UnitPrice may refer to a BaseQuantity other than 1.
     *
     * Our current canonical InvoiceLine.unit_price means price per one unit,
     * so silently ignoring BaseQuantity would be wrong.
     */
    if let Some(base_quantity) = &source.unit_price.base_quantity {
        let value = parse_decimal(
            base_quantity,
            "Details.ItemList.ListLineItem.UnitPrice.@BaseQuantity",
        )?;

        if value != Decimal::ONE {
            return Err(MappingError::UnsupportedBaseQuantity {
                value: base_quantity.clone(),
            });
        }
    }

    let unit_price = money(
        &source.unit_price.value,
        currency,
        "Details.ItemList.ListLineItem.UnitPrice",
    )?;

    let net_amount = money(
        &source.line_item_amount,
        currency,
        "Details.ItemList.ListLineItem.LineItemAmount",
    )?;

    let tax_rate = parse_decimal(
        &source.tax_item.tax_percent.value,
        "Details.ItemList.ListLineItem.TaxItem.TaxPercent",
    )?;

    Ok(InvoiceLine {
        id,
        description,
        quantity,
        unit_code: Some(source.quantity.unit),
        unit_price,
        net_amount,

        tax: TaxInformation {
            category_code: Some(source.tax_item.tax_percent.tax_category_code),
            rate: tax_rate,
        },
    })
}

fn map_vat_breakdown(
    source: EbInterfaceTaxItem,
    currency: &Currency,
) -> Result<VatBreakdown, MappingError> {
    let tax_amount = source.tax_amount.ok_or(MappingError::MissingField {
        field: "Tax.TaxItem.TaxAmount",
    })?;

    Ok(VatBreakdown {
        category_code: Some(source.tax_percent.tax_category_code),

        rate: parse_decimal(&source.tax_percent.value, "Tax.TaxItem.TaxPercent")?,

        taxable_amount: money(
            &source.taxable_amount,
            currency,
            "Tax.TaxItem.TaxableAmount",
        )?,

        tax_amount: money(&tax_amount, currency, "Tax.TaxItem.TaxAmount")?,
    })
}

fn map_account(source: EbInterfaceBeneficiaryAccount) -> Option<PaymentAccount> {
    source.iban.map(|identifier| PaymentAccount { identifier })
}

fn map_bank_transfer(
    source: EbInterfaceUniversalBankTransaction,
) -> Result<PaymentInformation, MappingError> {
    if source.beneficiary_accounts.len() > 1 {
        return Err(MappingError::MultipleBeneficiaryAccounts);
    }

    let payee_account = source
        .beneficiary_accounts
        .into_iter()
        .next()
        .and_then(map_account);

    let reference = source.payment_reference.map(|reference| reference.value);

    Ok(PaymentInformation {
        method: PaymentMethod::BankTransfer,
        means_code: None,
        reference,
        payee_account,
    })
}

impl TryFrom<EbInterfaceInvoice> for Invoice {
    type Error = MappingError;

    fn try_from(source: EbInterfaceInvoice) -> Result<Self, Self::Error> {
        let currency = Currency::new(source.invoice_currency);

        let issue_date = parse_date(&source.invoice_date, "InvoiceDate")?;

        let seller = map_party(source.biller, "Biller.Address")?;

        let order_reference = source
            .invoice_recipient
            .order_reference
            .as_ref()
            .map(|reference| reference.order_id.clone());

        let buyer = map_party(source.invoice_recipient, "InvoiceRecipient.Address")?;

        /*
         * ebInterface permits multiple ItemList elements.
         * The canonical model deliberately flattens them into one
         * invoice-lines collection.
         */
        let lines = source
            .details
            .item_lists
            .into_iter()
            .flat_map(|item_list| item_list.line_items)
            .map(|line| map_line(line, &currency))
            .collect::<Result<Vec<_>, _>>()?;

        let vat_breakdown = source
            .tax
            .tax_items
            .into_iter()
            .map(|tax| map_vat_breakdown(tax, &currency))
            .collect::<Result<Vec<_>, _>>()?;

        /*
         * The current ebInterface subset does not yet support
         * ReductionAndSurchargeDetails, so document-level
         * allowance and charge totals are zero.
         */
        let adjustments = Vec::new();

        let line_net_amount_value: Decimal = lines.iter().map(|line| line.net_amount.amount).sum();

        let tax_amount_value: Decimal = vat_breakdown.iter().map(|vat| vat.tax_amount.amount).sum();

        let line_net_amount = Money::new(line_net_amount_value, currency.clone());

        let allowance_amount = zero_money(&currency);

        let charge_amount = zero_money(&currency);

        /*
         * With no document-level allowances/charges in the
         * supported subset, canonical net amount equals the
         * sum of line net amounts.
         */
        let net_amount = Money::new(line_net_amount_value, currency.clone());

        let tax_amount = Money::new(tax_amount_value, currency.clone());

        let gross_amount = money(&source.total_gross_amount, &currency, "TotalGrossAmount")?;

        let payable_amount = money(&source.payable_amount, &currency, "PayableAmount")?;

        let totals = InvoiceTotals {
            line_net_amount,
            allowance_amount,
            charge_amount,

            net_amount,
            tax_amount,
            gross_amount,
            payable_amount,
        };

        let payment = match source.payment_method {
            None => None,

            Some(payment_method) => match payment_method.universal_bank_transaction {
                Some(bank_transfer) => Some(map_bank_transfer(bank_transfer)?),

                None => {
                    return Err(MappingError::UnsupportedPaymentMethod);
                }
            },
        };

        Ok(Invoice {
            id: InvoiceId::new(source.invoice_number),

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
