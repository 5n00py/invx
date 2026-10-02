use std::{io, string::FromUtf8Error};

use quick_xml::{
    Writer,
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
};

use rust_decimal::Decimal;
use thiserror::Error;

use crate::domain::{
    Address, AdjustmentKind, Currency, DocumentAdjustment, Invoice, InvoiceLine, Money, Party,
    PaymentInformation, PaymentMethod, TaxInformation, VatBreakdown,
};

const UBL_INVOICE_NAMESPACE: &str = "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2";

const CAC_NAMESPACE: &str =
    "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2";

const CBC_NAMESPACE: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2";

type XmlWriter = Writer<Vec<u8>>;

#[derive(Debug, Error)]
pub enum WriterError {
    #[error("failed to write UBL XML: {0}")]
    Io(#[from] io::Error),

    #[error("generated XML is not valid UTF-8: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("currency mismatch in {field}: expected {expected}, got {actual}")]
    CurrencyMismatch {
        field: &'static str,
        expected: String,
        actual: String,
    },

    #[error("payment method requires a UBL PaymentMeansCode")]
    MissingPaymentMeansCode,
}

pub fn write_invoice(invoice: &Invoice) -> Result<String, WriterError> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let mut root = BytesStart::new("Invoice");

    root.push_attribute(("xmlns", UBL_INVOICE_NAMESPACE));

    root.push_attribute(("xmlns:cac", CAC_NAMESPACE));

    root.push_attribute(("xmlns:cbc", CBC_NAMESPACE));

    writer.write_event(Event::Start(root))?;

    write_text(&mut writer, "cbc:ID", invoice.id.as_str())?;

    write_text(
        &mut writer,
        "cbc:IssueDate",
        &invoice.issue_date.to_string(),
    )?;

    write_text(
        &mut writer,
        "cbc:DocumentCurrencyCode",
        invoice.currency.as_str(),
    )?;

    if let Some(order_reference) = &invoice.order_reference {
        writer.write_event(Event::Start(BytesStart::new("cac:OrderReference")))?;

        write_text(&mut writer, "cbc:ID", order_reference)?;

        writer.write_event(Event::End(BytesEnd::new("cac:OrderReference")))?;
    }

    write_party(&mut writer, "cac:AccountingSupplierParty", &invoice.seller)?;

    write_party(&mut writer, "cac:AccountingCustomerParty", &invoice.buyer)?;

    if let Some(payment) = &invoice.payment {
        write_payment(&mut writer, payment)?;
    }

    for adjustment in &invoice.adjustments {
        write_adjustment(&mut writer, adjustment, &invoice.currency)?;
    }

    write_tax_total(&mut writer, invoice)?;

    write_totals(&mut writer, invoice)?;

    for line in &invoice.lines {
        write_line(&mut writer, line, &invoice.currency)?;
    }

    writer.write_event(Event::End(BytesEnd::new("Invoice")))?;

    Ok(String::from_utf8(writer.into_inner())?)
}

fn write_party(
    writer: &mut XmlWriter,
    wrapper: &'static str,
    party: &Party,
) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new(wrapper)))?;

    writer.write_event(Event::Start(BytesStart::new("cac:Party")))?;

    writer.write_event(Event::Start(BytesStart::new("cac:PartyName")))?;

    write_text(writer, "cbc:Name", &party.name)?;

    writer.write_event(Event::End(BytesEnd::new("cac:PartyName")))?;

    if let Some(address) = &party.address {
        write_address(writer, address)?;
    }

    if let Some(vat_id) = &party.vat_id {
        writer.write_event(Event::Start(BytesStart::new("cac:PartyTaxScheme")))?;

        write_text(writer, "cbc:CompanyID", vat_id)?;

        write_tax_scheme(writer)?;

        writer.write_event(Event::End(BytesEnd::new("cac:PartyTaxScheme")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("cac:Party")))?;

    writer.write_event(Event::End(BytesEnd::new(wrapper)))?;

    Ok(())
}

fn write_address(writer: &mut XmlWriter, address: &Address) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new("cac:PostalAddress")))?;

    if let Some(street) = &address.street {
        write_text(writer, "cbc:StreetName", street)?;
    }

    if let Some(city) = &address.city {
        write_text(writer, "cbc:CityName", city)?;
    }

    if let Some(postal_code) = &address.postal_code {
        write_text(writer, "cbc:PostalZone", postal_code)?;
    }

    if let Some(country_code) = &address.country_code {
        writer.write_event(Event::Start(BytesStart::new("cac:Country")))?;

        write_text(writer, "cbc:IdentificationCode", country_code)?;

        writer.write_event(Event::End(BytesEnd::new("cac:Country")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("cac:PostalAddress")))?;

    Ok(())
}

fn write_payment(writer: &mut XmlWriter, payment: &PaymentInformation) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new("cac:PaymentMeans")))?;

    let means_code = match &payment.means_code {
        Some(code) => code.as_str(),

        /*
         * A canonical bank transfer can be
         * represented as generic UBL credit
         * transfer code 30 if the source format
         * had no more specific UBL code.
         */
        None if payment.method == PaymentMethod::BankTransfer => "30",

        None => {
            return Err(WriterError::MissingPaymentMeansCode);
        }
    };

    write_text(writer, "cbc:PaymentMeansCode", means_code)?;

    if let Some(reference) = &payment.reference {
        write_text(writer, "cbc:PaymentID", reference)?;
    }

    if let Some(account) = &payment.payee_account {
        writer.write_event(Event::Start(BytesStart::new("cac:PayeeFinancialAccount")))?;

        write_text(writer, "cbc:ID", &account.identifier)?;

        writer.write_event(Event::End(BytesEnd::new("cac:PayeeFinancialAccount")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("cac:PaymentMeans")))?;

    Ok(())
}

fn write_adjustment(
    writer: &mut XmlWriter,
    adjustment: &DocumentAdjustment,
    invoice_currency: &Currency,
) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new("cac:AllowanceCharge")))?;

    let charge_indicator = match adjustment.kind {
        AdjustmentKind::Allowance => "false",

        AdjustmentKind::Charge => "true",
    };

    write_text(writer, "cbc:ChargeIndicator", charge_indicator)?;

    if let Some(reason_code) = &adjustment.reason_code {
        write_text(writer, "cbc:AllowanceChargeReasonCode", reason_code)?;
    }

    for reason in &adjustment.reasons {
        write_text(writer, "cbc:AllowanceChargeReason", reason)?;
    }

    if let Some(percentage) = adjustment.percentage {
        write_text(
            writer,
            "cbc:MultiplierFactorNumeric",
            &percentage.to_string(),
        )?;
    }

    write_money(
        writer,
        "cbc:Amount",
        &adjustment.amount,
        invoice_currency,
        "adjustments.amount",
    )?;

    if let Some(base_amount) = &adjustment.base_amount {
        write_money(
            writer,
            "cbc:BaseAmount",
            base_amount,
            invoice_currency,
            "adjustments.base_amount",
        )?;
    }

    if let Some(tax) = &adjustment.tax {
        write_tax_category(writer, "cac:TaxCategory", tax)?;
    }

    writer.write_event(Event::End(BytesEnd::new("cac:AllowanceCharge")))?;

    Ok(())
}

fn write_tax_total(writer: &mut XmlWriter, invoice: &Invoice) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new("cac:TaxTotal")))?;

    write_money(
        writer,
        "cbc:TaxAmount",
        &invoice.totals.tax_amount,
        &invoice.currency,
        "totals.tax_amount",
    )?;

    for vat in &invoice.vat_breakdown {
        write_vat_breakdown(writer, vat, &invoice.currency)?;
    }

    writer.write_event(Event::End(BytesEnd::new("cac:TaxTotal")))?;

    Ok(())
}

fn write_vat_breakdown(
    writer: &mut XmlWriter,
    vat: &VatBreakdown,
    invoice_currency: &Currency,
) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new("cac:TaxSubtotal")))?;

    write_money(
        writer,
        "cbc:TaxableAmount",
        &vat.taxable_amount,
        invoice_currency,
        "vat_breakdown.taxable_amount",
    )?;

    write_money(
        writer,
        "cbc:TaxAmount",
        &vat.tax_amount,
        invoice_currency,
        "vat_breakdown.tax_amount",
    )?;

    let tax = TaxInformation {
        category_code: vat.category_code.clone(),
        rate: vat.rate,
    };

    write_tax_category(writer, "cac:TaxCategory", &tax)?;

    writer.write_event(Event::End(BytesEnd::new("cac:TaxSubtotal")))?;

    Ok(())
}

fn write_totals(writer: &mut XmlWriter, invoice: &Invoice) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new("cac:LegalMonetaryTotal")))?;

    /*
     * Keep UBL MonetaryTotal element order.
     */

    write_money(
        writer,
        "cbc:LineExtensionAmount",
        &invoice.totals.line_net_amount,
        &invoice.currency,
        "totals.line_net_amount",
    )?;

    write_money(
        writer,
        "cbc:TaxExclusiveAmount",
        &invoice.totals.net_amount,
        &invoice.currency,
        "totals.net_amount",
    )?;

    write_money(
        writer,
        "cbc:TaxInclusiveAmount",
        &invoice.totals.gross_amount,
        &invoice.currency,
        "totals.gross_amount",
    )?;

    if invoice.totals.allowance_amount.amount != Decimal::ZERO
        || invoice
            .adjustments
            .iter()
            .any(|adjustment| adjustment.kind == AdjustmentKind::Allowance)
    {
        write_money(
            writer,
            "cbc:AllowanceTotalAmount",
            &invoice.totals.allowance_amount,
            &invoice.currency,
            "totals.allowance_amount",
        )?;
    }

    if invoice.totals.charge_amount.amount != Decimal::ZERO
        || invoice
            .adjustments
            .iter()
            .any(|adjustment| adjustment.kind == AdjustmentKind::Charge)
    {
        write_money(
            writer,
            "cbc:ChargeTotalAmount",
            &invoice.totals.charge_amount,
            &invoice.currency,
            "totals.charge_amount",
        )?;
    }

    write_money(
        writer,
        "cbc:PayableAmount",
        &invoice.totals.payable_amount,
        &invoice.currency,
        "totals.payable_amount",
    )?;

    writer.write_event(Event::End(BytesEnd::new("cac:LegalMonetaryTotal")))?;

    Ok(())
}

fn write_line(
    writer: &mut XmlWriter,
    line: &InvoiceLine,
    invoice_currency: &Currency,
) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new("cac:InvoiceLine")))?;

    write_text(writer, "cbc:ID", &line.id)?;

    write_quantity(writer, line)?;

    write_money(
        writer,
        "cbc:LineExtensionAmount",
        &line.net_amount,
        invoice_currency,
        "lines.net_amount",
    )?;

    writer.write_event(Event::Start(BytesStart::new("cac:Item")))?;

    write_text(writer, "cbc:Description", &line.description)?;

    write_tax_category(writer, "cac:ClassifiedTaxCategory", &line.tax)?;

    writer.write_event(Event::End(BytesEnd::new("cac:Item")))?;

    writer.write_event(Event::Start(BytesStart::new("cac:Price")))?;

    write_money(
        writer,
        "cbc:PriceAmount",
        &line.unit_price,
        invoice_currency,
        "lines.unit_price",
    )?;

    writer.write_event(Event::End(BytesEnd::new("cac:Price")))?;

    writer.write_event(Event::End(BytesEnd::new("cac:InvoiceLine")))?;

    Ok(())
}

fn write_quantity(writer: &mut XmlWriter, line: &InvoiceLine) -> Result<(), WriterError> {
    let mut element = BytesStart::new("cbc:InvoicedQuantity");

    if let Some(unit_code) = &line.unit_code {
        element.push_attribute(("unitCode", unit_code.as_str()));
    }

    writer.write_event(Event::Start(element))?;

    writer.write_event(Event::Text(BytesText::new(&line.quantity.to_string())))?;

    writer.write_event(Event::End(BytesEnd::new("cbc:InvoicedQuantity")))?;

    Ok(())
}

fn write_tax_category(
    writer: &mut XmlWriter,
    element_name: &'static str,
    tax: &TaxInformation,
) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new(element_name)))?;

    if let Some(category_code) = &tax.category_code {
        write_text(writer, "cbc:ID", category_code)?;
    }

    write_text(writer, "cbc:Percent", &tax.rate.to_string())?;

    write_tax_scheme(writer)?;

    writer.write_event(Event::End(BytesEnd::new(element_name)))?;

    Ok(())
}

fn write_tax_scheme(writer: &mut XmlWriter) -> Result<(), WriterError> {
    writer.write_event(Event::Start(BytesStart::new("cac:TaxScheme")))?;

    write_text(writer, "cbc:ID", "VAT")?;

    writer.write_event(Event::End(BytesEnd::new("cac:TaxScheme")))?;

    Ok(())
}

fn write_money(
    writer: &mut XmlWriter,
    element_name: &'static str,
    money: &Money,
    invoice_currency: &Currency,
    field: &'static str,
) -> Result<(), WriterError> {
    ensure_currency(money, invoice_currency, field)?;

    let mut element = BytesStart::new(element_name);

    element.push_attribute(("currencyID", money.currency.as_str()));

    writer.write_event(Event::Start(element))?;

    writer.write_event(Event::Text(BytesText::new(&money.amount.to_string())))?;

    writer.write_event(Event::End(BytesEnd::new(element_name)))?;

    Ok(())
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
