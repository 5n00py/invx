use rust_decimal::Decimal;

use super::Money;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxInformation {
    pub category_code: Option<String>,
    pub rate: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VatBreakdown {
    pub category_code: Option<String>,
    pub rate: Decimal,
    pub taxable_amount: Money,
    pub tax_amount: Money,
}
