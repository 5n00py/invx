use serde::Deserialize;

// -----------------------------------------------------------------------------
// Common value types
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceCountry {
    #[serde(rename = "@CountryCode")]
    pub country_code: Option<String>,

    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceQuantity {
    #[serde(rename = "@Unit")]
    pub unit: String,

    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceUnitPrice {
    #[serde(rename = "@BaseQuantity")]
    pub base_quantity: Option<String>,

    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceTaxPercent {
    #[serde(rename = "@TaxCategoryCode")]
    pub tax_category_code: String,

    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceClassification {
    #[serde(rename = "@ClassificationSchema")]
    pub classification_schema: Option<String>,

    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceCurrencyAmount {
    #[serde(rename = "@Currency")]
    pub currency: String,

    #[serde(rename = "$text")]
    pub value: String,
}

// -----------------------------------------------------------------------------
// References
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceOrderReference {
    #[serde(rename = "OrderID")]
    pub order_id: String,

    #[serde(rename = "ReferenceDate")]
    pub reference_date: Option<String>,

    #[serde(rename = "Description")]
    pub description: Option<String>,
}

// -----------------------------------------------------------------------------
// Party
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceAddress {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "TradingName")]
    pub trading_name: Option<String>,

    #[serde(rename = "Street")]
    pub street: Option<String>,

    #[serde(rename = "Town")]
    pub town: String,

    #[serde(rename = "ZIP")]
    pub zip: String,

    #[serde(rename = "Country")]
    pub country: EbInterfaceCountry,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceParty {
    #[serde(rename = "VATIdentificationNumber")]
    pub vat_identification_number: String,

    #[serde(rename = "OrderReference")]
    pub order_reference: Option<EbInterfaceOrderReference>,

    #[serde(rename = "Address")]
    pub address: Option<EbInterfaceAddress>,
}

// -----------------------------------------------------------------------------
// Tax
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceTaxItem {
    #[serde(rename = "TaxableAmount")]
    pub taxable_amount: String,

    #[serde(rename = "TaxPercent")]
    pub tax_percent: EbInterfaceTaxPercent,

    #[serde(rename = "TaxAmount")]
    pub tax_amount: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceTax {
    #[serde(rename = "TaxItem", default)]
    pub tax_items: Vec<EbInterfaceTaxItem>,
}

// -----------------------------------------------------------------------------
// Invoice line reductions / surcharges
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceLineAdjustment {
    #[serde(rename = "BaseAmount")]
    pub base_amount: String,

    #[serde(rename = "Percentage")]
    pub percentage: Option<String>,

    #[serde(rename = "Amount")]
    pub amount: Option<String>,

    #[serde(rename = "Comment")]
    pub comment: Option<String>,

    #[serde(rename = "Classification")]
    pub classification: Option<EbInterfaceClassification>,
}

/*
 * We do not yet have a canonical representation for
 * OtherVATableTaxListLineItem.
 *
 * Still parse it here so the mapping layer can detect
 * its presence and reject the conversion explicitly
 * instead of silently losing information.
 */
#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceOtherVatAbleTaxLineItem {
    #[serde(rename = "TaxableAmount")]
    pub taxable_amount: String,

    #[serde(rename = "TaxPercent")]
    pub tax_percent: EbInterfaceTaxPercent,

    #[serde(rename = "TaxAmount")]
    pub tax_amount: Option<String>,

    #[serde(rename = "AccountingCurrencyAmount")]
    pub accounting_currency_amount: Option<EbInterfaceCurrencyAmount>,

    #[serde(rename = "Comment")]
    pub comment: Option<String>,

    #[serde(rename = "TaxID")]
    pub tax_id: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum EbInterfaceLineAdjustmentEntry {
    #[serde(rename = "ReductionListLineItem")]
    Reduction(EbInterfaceLineAdjustment),

    #[serde(rename = "SurchargeListLineItem")]
    Surcharge(EbInterfaceLineAdjustment),

    #[serde(rename = "OtherVATableTaxListLineItem")]
    OtherVatAbleTax(EbInterfaceOtherVatAbleTaxLineItem),
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceReductionAndSurchargeListLineItemDetails {
    #[serde(rename = "$value", default)]
    pub items: Vec<EbInterfaceLineAdjustmentEntry>,
}

// -----------------------------------------------------------------------------
// Invoice lines
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceLineItem {
    #[serde(rename = "PositionNumber")]
    pub position_number: Option<String>,

    #[serde(rename = "Description", default)]
    pub descriptions: Vec<String>,

    #[serde(rename = "Quantity")]
    pub quantity: EbInterfaceQuantity,

    #[serde(rename = "UnitPrice")]
    pub unit_price: EbInterfaceUnitPrice,

    #[serde(rename = "ReductionAndSurchargeListLineItemDetails")]
    pub reduction_and_surcharge_details:
        Option<EbInterfaceReductionAndSurchargeListLineItemDetails>,

    #[serde(rename = "TaxItem")]
    pub tax_item: EbInterfaceTaxItem,

    #[serde(rename = "LineItemAmount")]
    pub line_item_amount: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceItemList {
    #[serde(rename = "ListLineItem", default)]
    pub line_items: Vec<EbInterfaceLineItem>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceDetails {
    #[serde(rename = "ItemList", default)]
    pub item_lists: Vec<EbInterfaceItemList>,
}

// -----------------------------------------------------------------------------
// Payment
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceBeneficiaryAccount {
    #[serde(rename = "BankAccountNr")]
    pub bank_account_nr: Option<String>,

    #[serde(rename = "IBAN")]
    pub iban: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfacePaymentReference {
    #[serde(rename = "@CheckSum")]
    pub check_sum: Option<String>,

    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceUniversalBankTransaction {
    #[serde(rename = "BeneficiaryAccount", default)]
    pub beneficiary_accounts: Vec<EbInterfaceBeneficiaryAccount>,

    #[serde(rename = "PaymentReference")]
    pub payment_reference: Option<EbInterfacePaymentReference>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfacePaymentMethod {
    #[serde(rename = "UniversalBankTransaction")]
    pub universal_bank_transaction: Option<EbInterfaceUniversalBankTransaction>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfacePaymentConditions {
    #[serde(rename = "DueDate")]
    pub due_date: Option<String>,
}

// -----------------------------------------------------------------------------
// Invoice
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize, PartialEq)]
pub struct EbInterfaceInvoice {
    #[serde(rename = "@GeneratingSystem")]
    pub generating_system: String,

    #[serde(rename = "@DocumentType")]
    pub document_type: String,

    #[serde(rename = "@InvoiceCurrency")]
    pub invoice_currency: String,

    #[serde(rename = "InvoiceNumber")]
    pub invoice_number: String,

    #[serde(rename = "InvoiceDate")]
    pub invoice_date: String,

    #[serde(rename = "Biller")]
    pub biller: EbInterfaceParty,

    #[serde(rename = "InvoiceRecipient")]
    pub invoice_recipient: EbInterfaceParty,

    #[serde(rename = "Details")]
    pub details: EbInterfaceDetails,

    #[serde(rename = "Tax")]
    pub tax: EbInterfaceTax,

    #[serde(rename = "TotalGrossAmount")]
    pub total_gross_amount: String,

    #[serde(rename = "PayableAmount")]
    pub payable_amount: String,

    #[serde(rename = "PaymentMethod")]
    pub payment_method: Option<EbInterfacePaymentMethod>,

    #[serde(rename = "PaymentConditions")]
    pub payment_conditions: Option<EbInterfacePaymentConditions>,
}
