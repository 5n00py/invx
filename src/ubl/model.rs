use serde::Deserialize;

// -----------------------------------------------------------------------------
// Common value types
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// References
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblOrderReference {
    #[serde(rename = "ID")]
    pub id: String,
}

// -----------------------------------------------------------------------------
// Tax
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblTaxScheme {
    #[serde(rename = "ID")]
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblTaxCategory {
    #[serde(rename = "ID")]
    pub id: Option<String>,

    #[serde(rename = "Percent")]
    pub percent: String,

    #[serde(rename = "TaxScheme")]
    pub tax_scheme: Option<UblTaxScheme>,
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

    #[serde(rename = "TaxSubtotal", default)]
    pub tax_subtotals: Vec<UblTaxSubtotal>,
}

// -----------------------------------------------------------------------------
// Party
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblCountry {
    #[serde(rename = "IdentificationCode")]
    pub identification_code: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblPostalAddress {
    #[serde(rename = "StreetName")]
    pub street_name: Option<String>,

    #[serde(rename = "CityName")]
    pub city_name: Option<String>,

    #[serde(rename = "PostalZone")]
    pub postal_zone: Option<String>,

    #[serde(rename = "Country")]
    pub country: Option<UblCountry>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblPartyTaxScheme {
    #[serde(rename = "CompanyID")]
    pub company_id: Option<String>,

    #[serde(rename = "TaxScheme")]
    pub tax_scheme: Option<UblTaxScheme>,
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

    #[serde(rename = "PostalAddress")]
    pub postal_address: Option<UblPostalAddress>,

    #[serde(rename = "PartyTaxScheme", default)]
    pub party_tax_schemes: Vec<UblPartyTaxScheme>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblPartyContainer {
    #[serde(rename = "Party")]
    pub party: UblParty,
}

// -----------------------------------------------------------------------------
// Payment
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblFinancialAccount {
    #[serde(rename = "ID")]
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblPaymentMeans {
    #[serde(rename = "PaymentMeansCode")]
    pub payment_means_code: String,

    #[serde(rename = "PaymentID")]
    pub payment_id: Option<String>,

    #[serde(rename = "PayeeFinancialAccount")]
    pub payee_financial_account: Option<UblFinancialAccount>,
}

// -----------------------------------------------------------------------------
// Allowances and charges
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblAllowanceCharge {
    #[serde(rename = "ChargeIndicator")]
    pub charge_indicator: bool,

    #[serde(rename = "AllowanceChargeReasonCode")]
    pub reason_code: Option<String>,

    #[serde(rename = "AllowanceChargeReason", default)]
    pub reasons: Vec<String>,

    #[serde(rename = "MultiplierFactorNumeric")]
    pub multiplier_factor_numeric: Option<String>,

    #[serde(rename = "Amount")]
    pub amount: UblAmount,

    #[serde(rename = "BaseAmount")]
    pub base_amount: Option<UblAmount>,

    #[serde(rename = "TaxCategory")]
    pub tax_category: Option<UblTaxCategory>,
}

// -----------------------------------------------------------------------------
// Invoice lines
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Monetary totals
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblLegalMonetaryTotal {
    #[serde(rename = "LineExtensionAmount")]
    pub line_extension_amount: UblAmount,

    #[serde(rename = "AllowanceTotalAmount")]
    pub allowance_total_amount: Option<UblAmount>,

    #[serde(rename = "ChargeTotalAmount")]
    pub charge_total_amount: Option<UblAmount>,

    #[serde(rename = "TaxExclusiveAmount")]
    pub tax_exclusive_amount: UblAmount,

    #[serde(rename = "TaxInclusiveAmount")]
    pub tax_inclusive_amount: UblAmount,

    #[serde(rename = "PayableAmount")]
    pub payable_amount: UblAmount,
}

// -----------------------------------------------------------------------------
// Invoice
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct UblInvoice {
    #[serde(rename = "ID")]
    pub id: String,

    #[serde(rename = "IssueDate")]
    pub issue_date: String,

    #[serde(rename = "DocumentCurrencyCode")]
    pub document_currency_code: String,

    #[serde(rename = "OrderReference")]
    pub order_reference: Option<UblOrderReference>,

    #[serde(rename = "AccountingSupplierParty")]
    pub accounting_supplier_party: UblPartyContainer,

    #[serde(rename = "AccountingCustomerParty")]
    pub accounting_customer_party: UblPartyContainer,

    #[serde(rename = "PaymentMeans", default)]
    pub payment_means: Vec<UblPaymentMeans>,

    #[serde(rename = "AllowanceCharge", default)]
    pub allowance_charges: Vec<UblAllowanceCharge>,

    #[serde(rename = "TaxTotal")]
    pub tax_total: UblTaxTotal,

    #[serde(rename = "LegalMonetaryTotal")]
    pub legal_monetary_total: UblLegalMonetaryTotal,

    #[serde(rename = "InvoiceLine", default)]
    pub invoice_lines: Vec<UblInvoiceLine>,
}
