use super::{Money, TaxInformation};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentKind {
    Allowance,
    Charge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DocumentAdjustment {
    pub kind: AdjustmentKind,
    pub amount: Money,
    pub base_amount: Option<Money>,
    pub percentage: Option<Decimal>,
    pub reason_code: Option<String>,
    pub reasons: Vec<String>,
    pub tax: Option<TaxInformation>,
}
