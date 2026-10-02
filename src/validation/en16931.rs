use rust_decimal::Decimal;

use crate::domain::{Invoice, PaymentMethod};

use super::{Severity, ValidationResult, Violation, core::ValidationRule};

pub struct CreditTransferRequiresAccount;

impl ValidationRule for CreditTransferRequiresAccount {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let Some(payment) = &invoice.payment else {
            return Vec::new();
        };

        let requires_account = payment.method == PaymentMethod::BankTransfer;

        if requires_account && payment.payee_account.is_none() {
            return vec![Violation {
                code: "BR-61".to_string(),
                severity: Severity::Error,
                message: "Credit transfer requires a payment account identifier".to_string(),
                field: Some("payment.payee_account".to_string()),
            }];
        }

        Vec::new()
    }
}

pub struct InvoiceLineRequiresVatCategory;

impl ValidationRule for InvoiceLineRequiresVatCategory {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, line) in invoice.lines.iter().enumerate() {
            if line.tax.category_code.is_none() {
                violations.push(Violation {
                    code: "BR-CO-04".to_string(),
                    severity: Severity::Error,
                    message: "Invoice line must have a VAT category code".to_string(),
                    field: Some(format!("lines[{index}].tax.category_code")),
                });
            }
        }

        violations
    }
}

pub struct StandardRatedLineRequiresPositiveRate;

impl ValidationRule for StandardRatedLineRequiresPositiveRate {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, line) in invoice.lines.iter().enumerate() {
            if line.tax.category_code.as_deref() == Some("S") && line.tax.rate <= Decimal::ZERO {
                violations.push(Violation {
                    code: "BR-S-05".to_string(),
                    severity: Severity::Error,
                    message: "Standard-rated invoice line must have a VAT rate greater than zero"
                        .to_string(),
                    field: Some(format!("lines[{index}].tax.rate")),
                });
            }
        }

        violations
    }
}

pub struct StandardRatedLineRequiresVatBreakdown;

impl ValidationRule for StandardRatedLineRequiresVatBreakdown {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let has_standard_rated_line = invoice
            .lines
            .iter()
            .any(|line| line.tax.category_code.as_deref() == Some("S"));

        let has_standard_rated_breakdown = invoice
            .vat_breakdown
            .iter()
            .any(|vat| vat.category_code.as_deref() == Some("S"));

        if has_standard_rated_line && !has_standard_rated_breakdown {
            return vec![Violation {
                code: "BR-S-01".to_string(),
                severity: Severity::Error,
                message: "Standard-rated invoice lines require a standard-rated VAT breakdown"
                    .to_string(),
                field: Some("vat_breakdown".to_string()),
            }];
        }

        Vec::new()
    }
}

pub struct VatBreakdownTaxAmountMatchesRate;

impl ValidationRule for VatBreakdownTaxAmountMatchesRate {
    fn validate(&self, invoice: &Invoice) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (index, vat) in invoice.vat_breakdown.iter().enumerate() {
            let expected =
                (vat.taxable_amount.amount * vat.rate / Decimal::new(100, 0)).round_dp(2);

            if vat.tax_amount.amount != expected {
                violations.push(Violation {
                    code: "BR-CO-17".to_string(),
                    severity: Severity::Error,
                    message: format!(
                        "VAT breakdown tax amount must equal taxable amount multiplied by VAT rate; expected {expected}, found {}",
                        vat.tax_amount.amount
                    ),
                    field: Some(format!(
                        "vat_breakdown[{index}].tax_amount"
                    )),
                });
            }
        }

        violations
    }
}

pub(crate) fn validate_en16931_rules(invoice: &Invoice) -> ValidationResult {
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::new(CreditTransferRequiresAccount),
        Box::new(InvoiceLineRequiresVatCategory),
        Box::new(StandardRatedLineRequiresPositiveRate),
        Box::new(StandardRatedLineRequiresVatBreakdown),
        Box::new(VatBreakdownTaxAmountMatchesRate),
    ];

    let mut result = ValidationResult::default();

    for rule in rules {
        result.violations.extend(rule.validate(invoice));
    }

    result
}
