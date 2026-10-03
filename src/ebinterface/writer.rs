use std::{io, string::FromUtf8Error};

use quick_xml::{
    Writer,
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
};

use rust_decimal::Decimal;
use thiserror::Error;

use crate::domain::{
    Address, AdjustmentKind, Currency, DocumentAdjustment, Invoice, InvoiceLine, LineAdjustment,
    Money, Party, PaymentInformation, PaymentMethod, VatBreakdown,
};

const EBINTERFACE_NAMESPACE: &str = "http://www.ebinterface.at/schema/6p1/";

type XmlWriter = Writer<Vec<u8>>;

#[derive(Debug, Error)]
pub enum WriterError {
    #[error("failed to write ebInterface XML: {0}")]
    Io(#[from] io::Error),

    #[error("generated XML is not valid UTF-8: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("{party} VAT identification number is required for ebInterface 6.1")]
    MissingVatId { party: &'static str },

    #[error("{party} address is required for ebInterface 6.1")]
    MissingAddress { party: &'static str },

    #[error("{party} city is required for ebInterface 6.1")]
    MissingCity { party: &'static str },

    #[error("{party} postal code is required for ebInterface 6.1")]
    MissingPostalCode { party: &'static str },

    #[error("{party} country code is required for ebInterface 6.1")]
    MissingCountryCode { party: &'static str },

    #[error("invoice line '{line_id}' requires a unit code for ebInterface 6.1")]
    MissingUnitCode { line_id: String },

    #[error("invoice line ID '{line_id}' cannot be represented as an ebInterface PositionNumber")]
    InvalidPositionNumber { line_id: String },

    #[error("invoice line '{line_id}' requires a VAT category for ebInterface 6.1")]
    MissingLineTaxCategory { line_id: String },

    #[error("invoice line '{line_id}' adjustment requires BaseAmount for ebInterface 6.1")]
    MissingLineAdjustmentBaseAmount { line_id: String },

    #[error(
        "invoice line '{line_id}' price base quantity unit '{base_unit}' cannot be represented with line unit '{line_unit}' in ebInterface 6.1"
    )]
    BaseQuantityUnitMismatch {
        line_id: String,
        line_unit: String,
        base_unit: String,
    },

    #[error("VAT breakdown requires a VAT category for ebInterface 6.1")]
    MissingVatCategory,

    #[error("document adjustment requires BaseAmount for ebInterface 6.1")]
    MissingAdjustmentBaseAmount,

    #[error("document adjustment requires tax information for ebInterface 6.1")]
    MissingAdjustmentTax,

    #[error("document adjustment requires a VAT category for ebInterface 6.1")]
    MissingAdjustmentTaxCategory,

    #[error("payment method '{method}' cannot currently be written as ebInterface 6.1")]
    UnsupportedPaymentMethod { method: &'static str },

    #[error("currency mismatch in {field}: expected {expected}, got {actual}")]
    CurrencyMismatch {
        field: &'static str,
        expected: String,
        actual: String,
    },

    #[error("value '{value}' in {field} has more than {max_scale} decimal places")]
    TooManyDecimalPlaces {
        field: &'static str,
        value: String,
        max_scale: u32,
    },
}

pub fn write_invoice(invoice: &Invoice) -> Result<String, WriterError> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let mut root = BytesStart::new("Invoice");

    root.push_attribute(("xmlns", EBINTERFACE_NAMESPACE));

    root.push_attribute(("GeneratingSystem", "invx"));

    root.push_attribute(("DocumentType", "Invoice"));

    root.push_attribute(("InvoiceCurrency", invoice.currency.as_str()));

    writer.write_event(Event::Start(root))?;

    write_text(&mut writer, "InvoiceNumber", invoice.id.as_str())?;

    write_text(&mut writer, "InvoiceDate", &invoice.issue_date.to_string())?;

    /*
     * Official InvoiceType order:
     *
     * ...
     * Biller
     * InvoiceRecipient
     * OrderingParty?
     * Details
     * ReductionAndSurchargeDetails?
     * Tax
     * TotalGrossAmount
     * PrepaidAmount?
     * RoundingAmount?
     * PayableAmount
     * PaymentMethod?
     * ...
     */

    write_party(&mut writer, "Biller", &invoice.seller, None, "seller")?;

    write_party(
        &mut writer,
        "InvoiceRecipient",
        &invoice.buyer,
        invoice.order_reference.as_deref(),
        "buyer",
    )?;

    write_details(&mut writer, invoice)?;

    if !invoice.adjustments.is_empty() {
        write_adjustments(&mut writer, invoice)?;
    }

    write_tax(&mut writer, invoice)?;

    write_decimal2(
        &mut writer,
        "TotalGrossAmount",
        invoice.totals.gross_amount.amount,
        "totals.gross_amount",
    )?;

    write_decimal2(
        &mut writer,
        "PayableAmount",
        invoice.totals.payable_amount.amount,
        "totals.payable_amount",
    )?;

    if let Some(payment) = &invoice.payment {
        write_payment(&mut writer, payment)?;
    }

    writer.write_event(Event::End(BytesEnd::new("Invoice")))?;

    Ok(String::from_utf8(writer.into_inner())?)
}

fn write_party(
    writer: &mut XmlWriter,
    element_name: &'static str,
    party: &Party,
    order_reference: Option<&str>,
    party_name: &'static str,
) -> Result<(), WriterError> {
    let vat_id = party
        .vat_id
        .as_deref()
        .ok_or(WriterError::MissingVatId { party: party_name })?;

    let address = party
        .address
        .as_ref()
        .ok_or(WriterError::MissingAddress { party: party_name })?;

    writer.write_event(Event::Start(BytesStart::new(element_name)))?;

    /*
     * AbstractPartyType sequence:
     *
     * VATIdentificationNumber
     * FurtherIdentification*
     * OrderReference?
     * Address?
     * Contact?
     */

    write_text(writer, "VATIdentificationNumber", vat_id)?;

    if let Some(order_reference) = order_reference {
        writer.write_event(Event::Start(BytesStart::new("OrderReference")))?;

        write_text(writer, "OrderID", order_reference)?;

        writer.write_event(Event::End(BytesEnd::new("OrderReference")))?;
    }

    write_address(writer, address, &party.name, party_name)?;

    writer.write_event(Event::End(BytesEnd::new(element_name)))?;

    Ok(())
}

fn write_address(
    writer: &mut XmlWriter,
    address: &Address,
    name: &str,
    party: &'static str,
) -> Result<(), WriterError> {
    let city = address
        .city
        .as_deref()
        .ok_or(WriterError::MissingCity { party })?;

    let postal_code = address
        .postal_code
        .as_deref()
        .ok_or(WriterError::MissingPostalCode { party })?;

    let country_code = address
        .country_code
        .as_deref()
        .ok_or(WriterError::MissingCountryCode { party })?;

    writer.write_event(Event::Start(BytesStart::new("Address")))?;

    /*
     * AddressType requires:
     *
     * Name
     * Town
     * ZIP
     * Country
     *
     * Street itself is optional.
     */

    write_text(writer, "Name", name)?;

    if let Some(street) = &address.street {
        write_text(writer, "Street", street)?;
    }

    write_text(writer, "Town", city)?;

    write_text(writer, "ZIP", postal_code)?;

    let mut country = BytesStart::new("Country");

    country.push_attribute(("CountryCode", country_code));

    writer.write_event(Event::Start(country))?;

    /*
     * The canonical model currently stores only
     * the country code, not a separate country name.
     * Using the code as the textual representation
     * avoids inventing localized country names.
     */
    writer.write_event(Event::Text(BytesText::new(country_code)))?;

    writer.write_event(Event::End(BytesEnd::new("Country")))?;

    writer.write_event(Event::End(BytesEnd::new("Address")))?;

    Ok(())
}

fn write_details(writer: &mut XmlWriter, invoice: &Invoice) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new("Details")))?;

    /*
     * Our canonical model has one flat line list.
     * Represent it as one ebInterface ItemList.
     */
    writer.write_event(Event::Start(BytesStart::new("ItemList")))?;

    for line in &invoice.lines {
        write_line(writer, line, &invoice.currency)?;
    }

    writer.write_event(Event::End(BytesEnd::new("ItemList")))?;

    writer.write_event(Event::End(BytesEnd::new("Details")))?;

    Ok(())
}

fn write_line(
    writer: &mut XmlWriter,
    line: &InvoiceLine,
    currency: &Currency,
) -> Result<(), WriterError> {
    ensure_currency(&line.unit_price, currency, "lines.unit_price")?;

    ensure_currency(&line.net_amount, currency, "lines.net_amount")?;

    let unit_code = line
        .unit_code
        .as_deref()
        .ok_or_else(|| WriterError::MissingUnitCode {
            line_id: line.id.clone(),
        })?;

    /*
     * PositionNumber is xs:positiveInteger.
     * Do not silently drop a canonical line ID
     * that cannot be represented.
     */
    let position_number = line
        .id
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| WriterError::InvalidPositionNumber {
            line_id: line.id.clone(),
        })?;

    let category =
        line.tax
            .category_code
            .as_deref()
            .ok_or_else(|| WriterError::MissingLineTaxCategory {
                line_id: line.id.clone(),
            })?;

    writer.write_event(Event::Start(BytesStart::new("ListLineItem")))?;

    write_text(writer, "PositionNumber", &position_number.to_string())?;

    write_text(writer, "Description", &line.description)?;

    write_quantity(writer, line.quantity, unit_code)?;

    write_unit_price(writer, line, unit_code)?;

    if !line.adjustments.is_empty() {
        write_line_adjustments(writer, line, currency)?;
    }

    write_tax_item(
        writer,
        line.net_amount.amount,
        category,
        line.tax.rate,
        None,
        "lines.tax",
    )?;

    write_decimal2(
        writer,
        "LineItemAmount",
        line.net_amount.amount,
        "lines.net_amount",
    )?;

    writer.write_event(Event::End(BytesEnd::new("ListLineItem")))?;

    Ok(())
}

fn write_unit_price(
    writer: &mut XmlWriter,
    line: &InvoiceLine,
    line_unit: &str,
) -> Result<(), WriterError> {
    let value = format_decimal(line.unit_price.amount, 4, "lines.unit_price")?;

    let base_quantity_value = match &line.price_base_quantity {
        None => None,

        Some(base_quantity) => {
            /*
             * ebInterface represents
             * BaseQuantity only as an
             * attribute on UnitPrice.
             *
             * It cannot carry an independent
             * unit code. If the canonical
             * base quantity explicitly uses
             * another unit, writing it would
             * lose information.
             */
            if let Some(base_unit) = &base_quantity.unit_code
                && base_unit != line_unit
            {
                return Err(WriterError::BaseQuantityUnitMismatch {
                    line_id: line.id.clone(),

                    line_unit: line_unit.to_string(),

                    base_unit: base_unit.clone(),
                });
            }

            Some(format_decimal(
                base_quantity.quantity,
                4,
                "lines.price_base_quantity.quantity",
            )?)
        }
    };

    let mut element = BytesStart::new("UnitPrice");

    if let Some(base_quantity) = &base_quantity_value {
        element.push_attribute(("BaseQuantity", base_quantity.as_str()));
    }

    writer.write_event(Event::Start(element))?;

    writer.write_event(Event::Text(BytesText::new(&value)))?;

    writer.write_event(Event::End(BytesEnd::new("UnitPrice")))?;

    Ok(())
}

fn write_line_adjustments(
    writer: &mut XmlWriter,
    line: &InvoiceLine,
    currency: &Currency,
) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new(
        "ReductionAndSurchargeListLineItemDetails",
    )))?;

    for adjustment in &line.adjustments {
        write_line_adjustment(writer, line, adjustment, currency)?;
    }

    writer.write_event(Event::End(BytesEnd::new(
        "ReductionAndSurchargeListLineItemDetails",
    )))?;

    Ok(())
}

fn write_line_adjustment(
    writer: &mut XmlWriter,
    line: &InvoiceLine,
    adjustment: &LineAdjustment,
    currency: &Currency,
) -> Result<(), WriterError> {
    ensure_currency(&adjustment.amount, currency, "lines.adjustments.amount")?;

    let base_amount = adjustment.base_amount.as_ref().ok_or_else(|| {
        WriterError::MissingLineAdjustmentBaseAmount {
            line_id: line.id.clone(),
        }
    })?;

    ensure_currency(base_amount, currency, "lines.adjustments.base_amount")?;

    let element_name = match adjustment.kind {
        AdjustmentKind::Allowance => "ReductionListLineItem",

        AdjustmentKind::Charge => "SurchargeListLineItem",
    };

    writer.write_event(Event::Start(BytesStart::new(element_name)))?;

    /*
     * ReductionAndSurchargeBaseType:
     *
     * BaseAmount
     * Percentage?
     * Amount?
     * Comment?
     * Classification?
     */

    write_decimal2(
        writer,
        "BaseAmount",
        base_amount.amount,
        "lines.adjustments.base_amount",
    )?;

    if let Some(percentage) = adjustment.percentage {
        write_decimal2(
            writer,
            "Percentage",
            percentage,
            "lines.adjustments.percentage",
        )?;
    }

    /*
     * Canonical LineAdjustment always has an
     * explicit amount, so preserve it instead
     * of forcing the target to recalculate it.
     */
    write_decimal2(
        writer,
        "Amount",
        adjustment.amount.amount,
        "lines.adjustments.amount",
    )?;

    if !adjustment.reasons.is_empty() {
        let comment = adjustment.reasons.join("; ");

        write_text(writer, "Comment", &comment)?;
    }

    if let Some(reason_code) = &adjustment.reason_code {
        /*
         * The canonical model currently retains
         * the classification value but not an
         * ebInterface ClassificationSchema.
         */
        write_text(writer, "Classification", reason_code)?;
    }

    writer.write_event(Event::End(BytesEnd::new(element_name)))?;

    Ok(())
}

fn write_quantity(
    writer: &mut XmlWriter,
    quantity: Decimal,
    unit_code: &str,
) -> Result<(), WriterError> {
    let value = format_decimal(quantity, 4, "lines.quantity")?;

    let mut element = BytesStart::new("Quantity");

    element.push_attribute(("Unit", unit_code));

    writer.write_event(Event::Start(element))?;

    writer.write_event(Event::Text(BytesText::new(&value)))?;

    writer.write_event(Event::End(BytesEnd::new("Quantity")))?;

    Ok(())
}

fn write_adjustments(writer: &mut XmlWriter, invoice: &Invoice) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new(
        "ReductionAndSurchargeDetails",
    )))?;

    for adjustment in &invoice.adjustments {
        write_adjustment(writer, adjustment, &invoice.currency)?;
    }

    writer.write_event(Event::End(BytesEnd::new("ReductionAndSurchargeDetails")))?;

    Ok(())
}

fn write_adjustment(
    writer: &mut XmlWriter,
    adjustment: &DocumentAdjustment,
    currency: &Currency,
) -> Result<(), WriterError> {
    ensure_currency(&adjustment.amount, currency, "adjustments.amount")?;

    let base_amount = adjustment
        .base_amount
        .as_ref()
        .ok_or(WriterError::MissingAdjustmentBaseAmount)?;

    ensure_currency(base_amount, currency, "adjustments.base_amount")?;

    let tax = adjustment
        .tax
        .as_ref()
        .ok_or(WriterError::MissingAdjustmentTax)?;

    let category = tax
        .category_code
        .as_deref()
        .ok_or(WriterError::MissingAdjustmentTaxCategory)?;

    let element_name = match adjustment.kind {
        AdjustmentKind::Allowance => "Reduction",

        AdjustmentKind::Charge => "Surcharge",
    };

    writer.write_event(Event::Start(BytesStart::new(element_name)))?;

    /*
     * ReductionAndSurchargeBaseType order:
     *
     * BaseAmount
     * Percentage?
     * Amount?
     * Comment?
     * Classification?
     * Extension?
     *
     * ReductionAndSurchargeType then adds TaxItem.
     */

    write_decimal2(
        writer,
        "BaseAmount",
        base_amount.amount,
        "adjustments.base_amount",
    )?;

    if let Some(percentage) = adjustment.percentage {
        write_decimal2(writer, "Percentage", percentage, "adjustments.percentage")?;
    }

    write_decimal2(
        writer,
        "Amount",
        adjustment.amount.amount,
        "adjustments.amount",
    )?;

    if !adjustment.reasons.is_empty() {
        let comment = adjustment.reasons.join("; ");

        write_text(writer, "Comment", &comment)?;
    }

    if let Some(reason_code) = &adjustment.reason_code {
        write_text(writer, "Classification", reason_code)?;
    }

    /*
     * In ebInterface the TaxItem on a reduction or
     * surcharge describes the VAT treatment of the
     * adjustment itself.
     *
     * TaxAmount is optional, so we do not manufacture
     * one here.
     */
    write_tax_item(
        writer,
        adjustment.amount.amount,
        category,
        tax.rate,
        None,
        "adjustments.tax",
    )?;

    writer.write_event(Event::End(BytesEnd::new(element_name)))?;

    Ok(())
}

fn write_tax(writer: &mut XmlWriter, invoice: &Invoice) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new("Tax")))?;

    for vat in &invoice.vat_breakdown {
        write_vat_breakdown(writer, vat, &invoice.currency)?;
    }

    writer.write_event(Event::End(BytesEnd::new("Tax")))?;

    Ok(())
}

fn write_vat_breakdown(
    writer: &mut XmlWriter,
    vat: &VatBreakdown,
    currency: &Currency,
) -> Result<(), WriterError> {
    ensure_currency(
        &vat.taxable_amount,
        currency,
        "vat_breakdown.taxable_amount",
    )?;

    ensure_currency(&vat.tax_amount, currency, "vat_breakdown.tax_amount")?;

    let category = vat
        .category_code
        .as_deref()
        .ok_or(WriterError::MissingVatCategory)?;

    write_tax_item(
        writer,
        vat.taxable_amount.amount,
        category,
        vat.rate,
        Some(vat.tax_amount.amount),
        "vat_breakdown",
    )
}

fn write_tax_item(
    writer: &mut XmlWriter,
    taxable_amount: Decimal,
    category: &str,
    rate: Decimal,
    tax_amount: Option<Decimal>,
    field: &'static str,
) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new("TaxItem")))?;

    write_decimal2(writer, "TaxableAmount", taxable_amount, field)?;

    write_tax_percent(writer, rate, category, field)?;

    if let Some(tax_amount) = tax_amount {
        write_decimal2(writer, "TaxAmount", tax_amount, field)?;
    }

    writer.write_event(Event::End(BytesEnd::new("TaxItem")))?;

    Ok(())
}

fn write_tax_percent(
    writer: &mut XmlWriter,
    rate: Decimal,
    category: &str,
    field: &'static str,
) -> Result<(), WriterError> {
    let value = format_decimal(rate, 2, field)?;

    let mut element = BytesStart::new("TaxPercent");

    element.push_attribute(("TaxCategoryCode", category));

    writer.write_event(Event::Start(element))?;

    writer.write_event(Event::Text(BytesText::new(&value)))?;

    writer.write_event(Event::End(BytesEnd::new("TaxPercent")))?;

    Ok(())
}

fn write_payment(writer: &mut XmlWriter, payment: &PaymentInformation) -> Result<(), WriterError> {
    if payment.method != PaymentMethod::BankTransfer {
        return Err(WriterError::UnsupportedPaymentMethod { method: "other" });
    }

    writer.write_event(Event::Start(BytesStart::new("PaymentMethod")))?;

    writer.write_event(Event::Start(BytesStart::new("UniversalBankTransaction")))?;

    if let Some(account) = &payment.payee_account {
        writer.write_event(Event::Start(BytesStart::new("BeneficiaryAccount")))?;

        /*
         * Our canonical account currently stores only
         * an identifier, not its identifier scheme.
         *
         * If it looks like an IBAN, preserve that useful
         * ebInterface representation. Otherwise use the
         * generic BankAccountNr element.
         */
        if looks_like_iban(&account.identifier) {
            write_text(writer, "IBAN", &account.identifier)?;
        } else {
            write_text(writer, "BankAccountNr", &account.identifier)?;
        }

        writer.write_event(Event::End(BytesEnd::new("BeneficiaryAccount")))?;
    }

    if let Some(reference) = &payment.reference {
        write_text(writer, "PaymentReference", reference)?;
    }

    writer.write_event(Event::End(BytesEnd::new("UniversalBankTransaction")))?;

    writer.write_event(Event::End(BytesEnd::new("PaymentMethod")))?;

    Ok(())
}

fn write_decimal2(
    writer: &mut XmlWriter,
    element_name: &'static str,
    value: Decimal,
    field: &'static str,
) -> Result<(), WriterError> {
    let value = format_decimal(value, 2, field)?;

    write_text(writer, element_name, &value)
}

fn format_decimal(
    value: Decimal,
    max_scale: u32,
    field: &'static str,
) -> Result<String, WriterError> {
    if value.scale() <= max_scale {
        return Ok(value.to_string());
    }

    /*
     * 10.000 is representable as Decimal2 even
     * though its current scale is 3.
     */
    let normalized = value.normalize();

    if normalized.scale() <= max_scale {
        return Ok(normalized.to_string());
    }

    Err(WriterError::TooManyDecimalPlaces {
        field,
        value: value.to_string(),
        max_scale,
    })
}

fn write_text(
    writer: &mut XmlWriter,
    element_name: &'static str,
    value: &str,
) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new(element_name)))?;

    writer.write_event(Event::Text(BytesText::new(value)))?;

    writer.write_event(Event::End(BytesEnd::new(element_name)))?;

    Ok(())
}

fn ensure_currency(
    money: &Money,
    invoice_currency: &Currency,
    field: &'static str,
) -> Result<(), WriterError> {
    if money.currency != *invoice_currency {
        return Err(WriterError::CurrencyMismatch {
            field,

            expected: invoice_currency.as_str().to_string(),

            actual: money.currency.as_str().to_string(),
        });
    }

    Ok(())
}

fn looks_like_iban(value: &str) -> bool {
    let compact = value
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    let bytes = compact.as_bytes();

    if !(15..=34).contains(&bytes.len()) {
        return false;
    }

    bytes[0].is_ascii_alphabetic()
        && bytes[1].is_ascii_alphabetic()
        && bytes[2].is_ascii_digit()
        && bytes[3].is_ascii_digit()
        && bytes[4..].iter().all(|byte| byte.is_ascii_alphanumeric())
}
