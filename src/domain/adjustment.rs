use rust_decimal::Decimal;

use super::{Money, TaxInformation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustmentKind {
    Allowance,
    Charge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentAdjustment {
    pub kind: AdjustmentKind,
    pub amount: Money,
    pub base_amount: Option<Money>,
    pub percentage: Option<Decimal>,
    pub reason_code: Option<String>,
    pub reasons: Vec<String>,
    pub tax: Option<TaxInformation>,
}
