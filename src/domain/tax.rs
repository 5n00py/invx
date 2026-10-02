use super::Money;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaxInformation {
    pub category_code: Option<String>,
    pub rate: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VatBreakdown {
    pub category_code: Option<String>,
    pub rate: Decimal,
    pub taxable_amount: Money,
    pub tax_amount: Money,
}
