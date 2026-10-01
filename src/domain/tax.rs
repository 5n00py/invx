use rust_decimal::Decimal;

use super::Money;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxInformation {
    pub rate: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VatBreakdown {
    pub rate: Decimal,
    pub taxable_amount: Money,
    pub tax_amount: Money,
}
