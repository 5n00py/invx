use super::{
    Currency, DocumentAdjustment, Money, Party, PaymentInformation, TaxInformation, VatBreakdown,
};
use crate::domain::adjustment::LineAdjustment;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InvoiceId(String);

impl InvoiceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PriceBaseQuantity {
    pub quantity: Decimal,
    pub unit_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InvoiceLine {
    pub id: String,

    pub description: String,

    pub quantity: Decimal,

    pub unit_code: Option<String>,

    pub unit_price: Money,

    /// Optional item price base quantity.
    ///
    /// None means the price applies to one unit.
    pub price_base_quantity: Option<PriceBaseQuantity>,

    /// Line-level allowances and charges.
    pub adjustments: Vec<LineAdjustment>,

    /// Net line amount after line-level
    /// allowances and charges, excluding VAT.
    pub net_amount: Money,

    pub tax: TaxInformation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InvoiceTotals {
    pub line_net_amount: Money,
    pub allowance_amount: Money,
    pub charge_amount: Money,

    pub net_amount: Money,
    pub tax_amount: Money,
    pub gross_amount: Money,
    pub payable_amount: Money,
}

impl InvoiceTotals {
    pub fn is_arithmetically_consistent(&self) -> bool {
        let same_currency = self.line_net_amount.currency == self.net_amount.currency
            && self.allowance_amount.currency == self.net_amount.currency
            && self.charge_amount.currency == self.net_amount.currency
            && self.tax_amount.currency == self.net_amount.currency
            && self.gross_amount.currency == self.net_amount.currency
            && self.payable_amount.currency == self.net_amount.currency;

        let adjusted_net =
            self.line_net_amount.amount - self.allowance_amount.amount + self.charge_amount.amount;

        same_currency
            && adjusted_net == self.net_amount.amount
            && self.net_amount.amount + self.tax_amount.amount == self.gross_amount.amount
            && self.gross_amount.amount == self.payable_amount.amount
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

    pub adjustments: Vec<DocumentAdjustment>,
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
            line_net_amount: eur(1_050_000, 2),
            allowance_amount: eur(47_500, 2),
            charge_amount: eur(10_000, 2),

            net_amount: eur(1_012_500, 2),
            tax_amount: eur(192_500, 2),
            gross_amount: eur(1_205_000, 2),
            payable_amount: eur(1_205_000, 2),
        };

        assert!(totals.is_arithmetically_consistent());
    }

    #[test]
    fn invoice_totals_detect_inconsistent_adjusted_net_amount() {
        let totals = InvoiceTotals {
            line_net_amount: eur(1_050_000, 2),
            allowance_amount: eur(47_500, 2),
            charge_amount: eur(10_000, 2),

            // Should be 10,125.00
            net_amount: eur(1_020_000, 2),
            tax_amount: eur(192_500, 2),
            gross_amount: eur(1_212_500, 2),
            payable_amount: eur(1_212_500, 2),
        };

        assert!(!totals.is_arithmetically_consistent());
    }

    #[test]
    fn invoice_totals_detect_inconsistent_gross_amount() {
        let totals = InvoiceTotals {
            line_net_amount: eur(1_050_000, 2),
            allowance_amount: eur(47_500, 2),
            charge_amount: eur(10_000, 2),

            net_amount: eur(1_012_500, 2),
            tax_amount: eur(192_500, 2),

            // Should be 12,050.00
            gross_amount: eur(1_195_000, 2),
            payable_amount: eur(1_195_000, 2),
        };

        assert!(!totals.is_arithmetically_consistent());
    }

    #[test]
    fn invoice_totals_detect_currency_mismatch() {
        let totals = InvoiceTotals {
            line_net_amount: eur(1_050_000, 2),
            allowance_amount: eur(47_500, 2),

            charge_amount: Money::new(Decimal::new(10_000, 2), Currency::new("USD")),

            net_amount: eur(1_012_500, 2),
            tax_amount: eur(192_500, 2),
            gross_amount: eur(1_205_000, 2),
            payable_amount: eur(1_205_000, 2),
        };

        assert!(!totals.is_arithmetically_consistent());
    }
}
