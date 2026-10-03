use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;
use thiserror::Error;

use crate::domain::{
    Address, AdjustmentKind, Currency, Invoice, InvoiceId, InvoiceLine, InvoiceTotals,
    LineAdjustment, Money, Party, PaymentAccount, PaymentInformation, PaymentMethod,
    PriceBaseQuantity, TaxInformation, VatBreakdown,
};

use super::model::{
    EbInterfaceBeneficiaryAccount, EbInterfaceInvoice, EbInterfaceLineAdjustment,
    EbInterfaceLineAdjustmentEntry, EbInterfaceLineItem, EbInterfaceParty, EbInterfaceTaxItem,
    EbInterfaceUniversalBankTransaction,
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

    #[error("line adjustment must contain Amount or Percentage in field {field}")]
    MissingLineAdjustmentAmount { field: &'static str },

    #[error("OtherVATableTaxListLineItem is not yet supported by the canonical model")]
    UnsupportedOtherVatAbleTaxLineItem,
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

fn map_line_adjustment(
    source: EbInterfaceLineAdjustment,
    kind: AdjustmentKind,
    currency: &Currency,
) -> Result<LineAdjustment, MappingError> {
    let (base_amount_field, percentage_field, amount_field) = match kind {
        AdjustmentKind::Allowance => (
            "Details.ItemList.ListLineItem.ReductionAndSurchargeListLineItemDetails.ReductionListLineItem.BaseAmount",
            "Details.ItemList.ListLineItem.ReductionAndSurchargeListLineItemDetails.ReductionListLineItem.Percentage",
            "Details.ItemList.ListLineItem.ReductionAndSurchargeListLineItemDetails.ReductionListLineItem.Amount",
        ),

        AdjustmentKind::Charge => (
            "Details.ItemList.ListLineItem.ReductionAndSurchargeListLineItemDetails.SurchargeListLineItem.BaseAmount",
            "Details.ItemList.ListLineItem.ReductionAndSurchargeListLineItemDetails.SurchargeListLineItem.Percentage",
            "Details.ItemList.ListLineItem.ReductionAndSurchargeListLineItemDetails.SurchargeListLineItem.Amount",
        ),
    };

    let base_amount_value = parse_decimal(&source.base_amount, base_amount_field)?;

    let percentage = source
        .percentage
        .as_deref()
        .map(|value| parse_decimal(value, percentage_field))
        .transpose()?;

    let explicit_amount = source
        .amount
        .as_deref()
        .map(|value| parse_decimal(value, amount_field))
        .transpose()?;

    /*
     * ebInterface permits Amount to be omitted
     * when Percentage is supplied.
     *
     * If both are supplied, Amount takes
     * precedence.
     *
     * We keep the exact Decimal result here and
     * avoid introducing a rounding rule in the
     * syntax-to-canonical mapping.
     */
    let amount_value = match (explicit_amount, percentage) {
        (Some(amount), _) => amount,

        (None, Some(percentage)) => base_amount_value * percentage / Decimal::new(100, 0),

        (None, None) => {
            return Err(MappingError::MissingLineAdjustmentAmount {
                field: amount_field,
            });
        }
    };

    let reason_code = source.classification.map(|classification| {
        /*
         * The classification value is
         * preserved as canonical
         * reason_code.
         *
         * ClassificationSchema remains
         * syntax-specific metadata and
         * is not represented by the
         * current canonical model.
         */
        classification.value
    });

    let reasons = source.comment.into_iter().collect();

    Ok(LineAdjustment {
        kind,

        amount: Money::new(amount_value, currency.clone()),

        base_amount: Some(Money::new(base_amount_value, currency.clone())),

        percentage,

        reason_code,

        reasons,
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
     * ebInterface has no separate unit attribute
     * on UnitPrice/@BaseQuantity.
     *
     * The base quantity therefore uses the same
     * unit as the invoiced Quantity when mapped
     * into the canonical model.
     */
    let quantity_unit = source.quantity.unit;

    let price_base_quantity = source
        .unit_price
        .base_quantity
        .as_deref()
        .map(|value| {
            Ok::<PriceBaseQuantity, MappingError>(PriceBaseQuantity {
                quantity: parse_decimal(
                    value,
                    "Details.ItemList.ListLineItem.UnitPrice.@BaseQuantity",
                )?,

                unit_code: Some(quantity_unit.clone()),
            })
        })
        .transpose()?;

    let unit_price = money(
        &source.unit_price.value,
        currency,
        "Details.ItemList.ListLineItem.UnitPrice",
    )?;

    /*
     * Preserve the original adjustment order.
     *
     * ebInterface 6.1 permits reductions and
     * surcharges to be mixed.
     */
    let mut adjustments = Vec::new();

    if let Some(details) = source.reduction_and_surcharge_details {
        for item in details.items {
            match item {
                EbInterfaceLineAdjustmentEntry::Reduction(adjustment) => {
                    adjustments.push(map_line_adjustment(
                        adjustment,
                        AdjustmentKind::Allowance,
                        currency,
                    )?);
                }

                EbInterfaceLineAdjustmentEntry::Surcharge(adjustment) => {
                    adjustments.push(map_line_adjustment(
                        adjustment,
                        AdjustmentKind::Charge,
                        currency,
                    )?);
                }

                EbInterfaceLineAdjustmentEntry::OtherVatAbleTax(_) => {
                    /*
                     * This contributes to the
                     * ebInterface line-net formula,
                     * so silently ignoring it would
                     * change invoice semantics.
                     */
                    return Err(MappingError::UnsupportedOtherVatAbleTaxLineItem);
                }
            }
        }
    }

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

        unit_code: Some(quantity_unit),

        unit_price,

        price_base_quantity,

        adjustments,

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
    source
        .iban
        .or(source.bank_account_nr)
        .map(|identifier| PaymentAccount { identifier })
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
         * ebInterface permits multiple ItemList
         * elements.
         *
         * The canonical model deliberately
         * flattens them into one invoice-lines
         * collection.
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
         * The current ebInterface subset does
         * not yet support document-level
         * ReductionAndSurchargeDetails.
         */
        let adjustments = Vec::new();

        let line_net_amount_value: Decimal = lines.iter().map(|line| line.net_amount.amount).sum();

        let tax_amount_value: Decimal = vat_breakdown.iter().map(|vat| vat.tax_amount.amount).sum();

        let line_net_amount = Money::new(line_net_amount_value, currency.clone());

        let allowance_amount = zero_money(&currency);

        let charge_amount = zero_money(&currency);

        /*
         * With no document-level allowances or
         * charges in the currently supported
         * subset, canonical net amount equals
         * the sum of line net amounts.
         *
         * Line-level adjustments are already
         * reflected in each LineItemAmount.
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
