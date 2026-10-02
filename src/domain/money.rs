use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Currency(String);

impl Currency {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into().to_uppercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }
}
