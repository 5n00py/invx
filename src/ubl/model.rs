use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblAmount {
    #[serde(rename = "@currencyID")]
    pub currency_id: String,

    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblQuantity {
    #[serde(rename = "@unitCode")]
    pub unit_code: Option<String>,

    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblPartyName {
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblParty {
    #[serde(rename = "PartyName")]
    pub party_name: UblPartyName,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblPartyContainer {
    #[serde(rename = "Party")]
    pub party: UblParty,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblTaxCategory {
    #[serde(rename = "Percent")]
    pub percent: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblTaxSubtotal {
    #[serde(rename = "TaxableAmount")]
    pub taxable_amount: UblAmount,

    #[serde(rename = "TaxAmount")]
    pub tax_amount: UblAmount,

    #[serde(rename = "TaxCategory")]
    pub tax_category: UblTaxCategory,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblTaxTotal {
    #[serde(rename = "TaxAmount")]
    pub tax_amount: UblAmount,

    #[serde(rename = "TaxSubtotal")]
    pub tax_subtotals: Vec<UblTaxSubtotal>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblLegalMonetaryTotal {
    #[serde(rename = "LineExtensionAmount")]
    pub line_extension_amount: UblAmount,

    #[serde(rename = "TaxExclusiveAmount")]
    pub tax_exclusive_amount: UblAmount,

    #[serde(rename = "TaxInclusiveAmount")]
    pub tax_inclusive_amount: UblAmount,

    #[serde(rename = "PayableAmount")]
    pub payable_amount: UblAmount,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblItem {
    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "ClassifiedTaxCategory")]
    pub classified_tax_category: UblTaxCategory,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblPrice {
    #[serde(rename = "PriceAmount")]
    pub price_amount: UblAmount,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblInvoiceLine {
    #[serde(rename = "ID")]
    pub id: String,

    #[serde(rename = "InvoicedQuantity")]
    pub invoiced_quantity: UblQuantity,

    #[serde(rename = "LineExtensionAmount")]
    pub line_extension_amount: UblAmount,

    #[serde(rename = "Item")]
    pub item: UblItem,

    #[serde(rename = "Price")]
    pub price: UblPrice,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblInvoice {
    #[serde(rename = "ID")]
    pub id: String,

    #[serde(rename = "IssueDate")]
    pub issue_date: String,

    #[serde(rename = "DocumentCurrencyCode")]
    pub document_currency_code: String,

    #[serde(rename = "AccountingSupplierParty")]
    pub accounting_supplier_party: UblPartyContainer,

    #[serde(rename = "AccountingCustomerParty")]
    pub accounting_customer_party: UblPartyContainer,

    #[serde(rename = "TaxTotal")]
    pub tax_total: UblTaxTotal,

    #[serde(rename = "LegalMonetaryTotal")]
    pub legal_monetary_total: UblLegalMonetaryTotal,

    #[serde(rename = "InvoiceLine")]
    pub invoice_lines: Vec<UblInvoiceLine>,
}
