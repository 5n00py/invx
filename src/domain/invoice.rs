use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::{Currency, Money, Party, PaymentInformation, TaxInformation, VatBreakdown};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceId(String);

impl InvoiceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceLine {
    pub id: String,
    pub description: String,
    pub quantity: Decimal,
    pub unit_code: Option<String>,
    pub unit_price: Money,
    pub net_amount: Money,
    pub tax: TaxInformation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceTotals {
    pub net_amount: Money,
    pub tax_amount: Money,
    pub gross_amount: Money,
    pub payable_amount: Money,
}

impl InvoiceTotals {
    pub fn is_arithmetically_consistent(&self) -> bool {
        let same_currency = self.net_amount.currency == self.tax_amount.currency
            && self.net_amount.currency == self.gross_amount.currency
            && self.net_amount.currency == self.payable_amount.currency;

        same_currency
            && self.net_amount.amount + self.tax_amount.amount == self.gross_amount.amount
            && self.gross_amount.amount == self.payable_amount.amount
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invoice {
    pub id: InvoiceId,
    pub issue_date: NaiveDate,
    pub currency: Currency,

    pub seller: Party,
    pub buyer: Party,

    pub order_reference: Option<String>,

    pub lines: Vec<InvoiceLine>,
    pub vat_breakdown: Vec<VatBreakdown>,

    pub payment: Option<PaymentInformation>,

    pub totals: InvoiceTotals,
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;
    use crate::domain::Currency;

    fn eur(amount: i64, scale: u32) -> Money {
        Money::new(Decimal::new(amount, scale), Currency::new("EUR"))
    }

    #[test]
    fn invoice_totals_are_consistent() {
        let totals = InvoiceTotals {
            net_amount: eur(950_000, 2),
            tax_amount: eur(190_000, 2),
            gross_amount: eur(1_140_000, 2),
            payable_amount: eur(1_140_000, 2),
        };

        assert!(totals.is_arithmetically_consistent());
    }

    #[test]
    fn invoice_totals_detect_inconsistent_gross_amount() {
        let totals = InvoiceTotals {
            net_amount: eur(950_000, 2),
            tax_amount: eur(190_000, 2),
            gross_amount: eur(1_130_000, 2),
            payable_amount: eur(1_130_000, 2),
        };

        assert!(!totals.is_arithmetically_consistent());
    }

    #[test]
    fn invoice_totals_detect_currency_mismatch() {
        let totals = InvoiceTotals {
            net_amount: eur(950_000, 2),
            tax_amount: eur(190_000, 2),
            gross_amount: Money::new(Decimal::new(1_140_000, 2), Currency::new("USD")),
            payable_amount: eur(1_140_000, 2),
        };

        assert!(!totals.is_arithmetically_consistent());
    }
}
